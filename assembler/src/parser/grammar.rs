use std::fmt::Display;

use isa::instruction::{InstructionType, Mnemonic};
use shared::EnumCount;
// use thiserror::Error;

use crate::{
    asm::directive::DirectiveType,
    token::{self, IdentifierType, Token},
};

// #[derive(Error, Debug)]
// pub enum RuleError {
//     #[error("`{0}`")]
//     InvalidInstructionSequence(RuleToken),
//     #[error("directive|instruction|break")]
//     InvalidLabelSequence,
// }

#[derive(EnumCount, Copy, Clone, Debug)]
pub enum RuleToken {
    Register,
    Comma,
    Label,
    SymbolOrNumeric,
    ParenL,
    ParenR,
    Operator,
    //--- Hidden Token ---
    Symbol,
    LiteralString,
    SectionDir,
    InstructionOrDir,
    OperatorOrBreak,
    Break,
}

impl RuleToken {
    /// Token counts minus tokens that are "hidden"
    const fn token_count() -> usize {
        RuleToken::VARIANT_COUNT - Self::hidden_count()
    }

    /// Tokens that can be used for sequence matching
    const fn sequence() -> [Self; Self::token_count()] {
        [
            Self::Register,
            Self::Comma,
            Self::SymbolOrNumeric,
            Self::Label,
            Self::ParenL,
            Self::ParenR,
            Self::Operator,
        ]
    }

    /// "Hidden" tokens. Tokens that are only intended for single matching and not for sequence matching
    const fn hidden_count() -> usize {
        [
            // Self::Eol,
            Self::Symbol,
            Self::LiteralString,
            Self::SectionDir,
            Self::InstructionOrDir,
            Self::Break,
            Self::OperatorOrBreak,
        ]
        .len()
    }
}

// impl From<RuleToken> for Token {
//     fn from(value: RuleToken) -> Self {
//         match value {
//             RuleToken::Register => Token::register(),
//             RuleToken::Comma => Token::Comma,
//             RuleToken::Label => Token::Label,
//             RuleToken::SymbolOrNumeric => Token::symbol(),
//             RuleToken::ParenL => Token::ParenL,
//             RuleToken::ParenR => Token::ParenR,
//             RuleToken::Eol => Token::Eol,
//             RuleToken::Operator => todo!(),
//                     }
//     }
// }

impl PartialEq<RuleToken> for Token {
    fn eq(&self, other: &RuleToken) -> bool {
        use RuleToken::*;
        //IMPORTANT: Don't add hidden token here
        match (self, other) {
            (Token::Identifier(IdentifierType::Register(_)), Register)
            | (Token::Label, Label)
            | (
                Token::LiteralDecimal
                | Token::LiteralHex
                | Token::LiteralBinary
                | Token::Identifier(IdentifierType::Symbol),
                SymbolOrNumeric,
            )
            | (token::operator!(), Operator)
            | (Token::ParenR, ParenR)
            | (Token::ParenL, ParenL)
            | (Token::Comma, Comma)
            | (Token::Eol | Token::Eof, Break) => true,
            _ => false,
        }
    }
}

impl Display for RuleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RuleToken::*;
        match self {
            Register => Token::register().fmt(f),
            Comma => Token::Comma.fmt(f),
            Label => Token::Label.fmt(f),
            ParenL => Token::ParenL.fmt(f),
            ParenR => Token::ParenR.fmt(f),
            Break => write!(f, "{}|{}", Token::Eol, Token::Eof,),
            Symbol => Token::Identifier(IdentifierType::Symbol).fmt(f),
            LiteralString => Token::LiteralString.fmt(f),
            SectionDir => write!(
                f,
                "{}|{}|{}|{}",
                DirectiveType::Text,
                DirectiveType::Data,
                DirectiveType::Rodata,
                DirectiveType::Bss
            ),
            InstructionOrDir => write!(f, "{}|{}", Token::mnemonic(), Token::directive()),
            SymbolOrNumeric => write!(
                f,
                "{}|{}|{}|{}",
                Token::symbol(),
                Token::LiteralDecimal,
                Token::LiteralHex,
                Token::LiteralBinary
            ),
            Operator => write!(f, "{}|{}", Token::Positive, Token::Negative),
            OperatorOrBreak => write!(f, "{}|{}|{}", Token::Eol, Token::Eof, Operator.to_string()),
        }
    }
}

pub struct InstructionRule {
    sequence: [RuleToken; RuleToken::token_count()],
    ty: OperandRuleType,
}

impl InstructionRule {
    pub fn new(mnemonic: Mnemonic) -> InstructionRule {
        InstructionRule {
            sequence: RuleToken::sequence(),
            ty: OperandRuleType::from(mnemonic),
        }
    }

    pub fn ty(&self) -> OperandRuleType {
        self.ty
    }

    pub fn generate_sequence(&mut self) -> &[RuleToken] {
        use RuleToken::*;
        let last_id: usize = match self.ty {
            OperandRuleType::R3 => {
                // [Register, Comma, Register, Comma, Register]
                self.sequence[2] = Register;
                self.sequence[3] = Comma;
                self.sequence[4] = Register;
                4
            }
            OperandRuleType::R2I => {
                // [Register, Comma, Register, Comma, SymbolOrNumeric]
                self.sequence[2] = Register;
                self.sequence[3] = Comma;
                self.sequence[4] = SymbolOrNumeric;
                4
            }
            OperandRuleType::RI => {
                // [Register, Comma, SymbolOrNumeric]
                self.sequence[2] = SymbolOrNumeric;
                2
            }
            OperandRuleType::RIR => {
                // [Register, Comma, SymbolOrNumeric, ParenL, Register, ParenR]
                self.sequence[2] = SymbolOrNumeric;
                self.sequence[3] = ParenL;
                self.sequence[4] = Register;
                self.sequence[5] = ParenR;
                5
            }
            OperandRuleType::R2L => {
                // [Register, Comma, Register, Comma, Label]
                self.sequence[2] = Register;
                self.sequence[3] = Comma;
                self.sequence[4] = Label;
                4
            }
            OperandRuleType::RL => {
                // [Register, Comma, Label]
                self.sequence[2] = Label;
                2
            }
        };

        self.sequence.get(0..last_id + 1).unwrap()
    }
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
/// The operands rule
pub enum OperandRuleType {
    ///Register, Register, Register
    R3,
    #[default]
    ///Register, Register, Immediate
    R2I,
    ///Register, Register, Label
    R2L,
    ///Register, Immediate(Register)
    RIR,
    ///Register, Immediate
    RI,
    ///Register, Label
    RL,
}

impl OperandRuleType {
    /// Function to remind that there are noises e.g. `Comma, ParenL, ParenR` in every `X`index
    pub(crate) const fn noises_in_every() -> usize {
        2
    }
}

impl From<InstructionType> for OperandRuleType {
    fn from(value: InstructionType) -> Self {
        use InstructionType::*;
        match value {
            Arithmetic => Self::R3,
            IA => Self::R2I,
            IJ => Self::R2I,
            IL => Self::RIR,
            S => Self::RIR,
            B => Self::R2L,
            J => Self::RL,
            U => Self::RI,
        }
    }
}

impl From<Mnemonic> for OperandRuleType {
    fn from(value: Mnemonic) -> Self {
        use Mnemonic::*;
        match value {
            Add => Self::R3,
            Sub => Self::R3,
            Mul => Self::R3,
            And => Self::R3,
            Or => Self::R3,
            Xor => Self::R3,
            Shl => Self::R3,
            Shr => Self::R3,
            ShrA => Self::R3,
            AddI => Self::R2I,
            Lui => Self::RI,
            Lw => Self::RIR,
            Sw => Self::RIR,
            Syscall => Self::R3,
        }
    }
}
