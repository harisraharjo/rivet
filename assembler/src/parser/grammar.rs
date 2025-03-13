use std::{fmt::Display, marker::PhantomData};

use isa::instruction::{InstructionType, Mnemonic};
use shared::EnumCount;
use thiserror::Error;

use crate::token::{IdentifierType, Token};

pub struct InstructionRule {
    sequence: [OperandTokenType; OperandTokenType::token_count()],
    ty: OperandRuleType,
    // sequence: &'a [OperandTokenType],
}

impl InstructionRule {
    pub fn new(mnemonic: Mnemonic) -> InstructionRule {
        use OperandTokenType::*;
        let ty = OperandRuleType::from(mnemonic);
        InstructionRule {
            sequence: [Register, Comma, SymbolOrLiteral, Label, ParenL, ParenR],
            ty,
            // sequence: Self::generate_sequence(ty),
        }
    }

    // pub fn get(&self, index: usize) -> OperandTokenType {
    //     self.sequence[index]
    // }

    // pub fn len(&self) -> usize {
    //     self.sequence.len()
    // }

    // pub fn iter(&self) -> impl Iterator<Item = &OperandTokenType> + ExactSizeIterator + use<'_> {
    //     self.sequence.iter()
    // }

    // pub fn iter_real(&self) -> OperandTokenIter {
    //     OperandTokenIter::new(self.ty)
    // }

    pub fn ty(&self) -> OperandRuleType {
        self.ty
    }

    pub fn sequence(&mut self) -> &[OperandTokenType] {
        use OperandTokenType::*;
        let last_id: usize = match self.ty {
            OperandRuleType::R3 => {
                // [Register, Comma, Register, Comma, Register]
                self.sequence[2] = Register;
                self.sequence[3] = Comma;
                self.sequence[4] = Register;
                4
            }
            OperandRuleType::R2I => {
                // [Register, Comma, Register, Comma, SymbolOrLiteral]
                self.sequence[2] = Register;
                self.sequence[3] = Comma;
                self.sequence[4] = SymbolOrLiteral;
                4
            }
            OperandRuleType::RI => {
                // [Register, Comma, SymbolOrLiteral]
                self.sequence[2] = SymbolOrLiteral;
                2
            }
            OperandRuleType::RIR => {
                // [Register, Comma, SymbolOrLiteral, ParenL, Register, ParenR]
                self.sequence[2] = SymbolOrLiteral;
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
        // match ty {
        //     OperandRuleType::R3 => [Register, Comma, Register, Comma, Register].as_slice(),
        //     OperandRuleType::R2I => [Register, Comma, Register, Comma, SymbolOrLiteral].as_slice(),
        //     OperandRuleType::RI => [Register, Comma, SymbolOrLiteral].as_slice(),
        //     OperandRuleType::RIR => {
        //         [Register, Comma, SymbolOrLiteral, ParenL, Register, ParenR].as_slice()
        //     }
        //     OperandRuleType::R2L => [Register, Comma, Register, Comma, Label].as_slice(),
        //     OperandRuleType::RL => [Register, Comma, Label].as_slice(),
        // }
    }
}

// struct OperandTokenIter<'a> {
//     sequence: &'a [OperandTokenType],
//     pointer: usize,
//     index: usize,
// }

// impl<'a> OperandTokenIter<'a> {
//     fn new(ty: OperandRuleType, sequence: &'a mut [OperandTokenType]) -> OperandTokenIter<'a> {
//         match ty {
//             OperandRuleType::R3 => {
//                 // [Register, Comma, Register, Comma, Register]
//                 sequence[2] = OperandTokenType::Register;
//                 sequence[3] = OperandTokenType::Comma;
//                 sequence[4] = OperandTokenType::Register;
//             }
//             OperandRuleType::R2I => {
//                 // [Register, Comma, Register, Comma, SymbolOrLiteral]
//                 sequence[2] = OperandTokenType::Register;
//                 sequence[3] = OperandTokenType::Comma;
//                 sequence[4] = OperandTokenType::SymbolOrLiteral;
//             }
//             OperandRuleType::RI => {
//                 // [Register, Comma, SymbolOrLiteral]
//                 sequence[2] = OperandTokenType::SymbolOrLiteral;
//             }
//             OperandRuleType::RIR => {
//                 // [Register, Comma, SymbolOrLiteral, ParenL, Register, ParenR]
//                 sequence[2] = OperandTokenType::SymbolOrLiteral;
//                 sequence[3] = OperandTokenType::ParenL;
//                 sequence[4] = OperandTokenType::Register;
//                 sequence[5] = OperandTokenType::ParenR;
//             }
//             OperandRuleType::R2L => {
//                 // [Register, Comma, Register, Comma, Label]
//                 sequence[2] = OperandTokenType::Register;
//                 sequence[3] = OperandTokenType::Comma;
//                 sequence[4] = OperandTokenType::Label;
//             }
//             OperandRuleType::RL => {
//                 // [Register, Comma, Label]
//                 sequence[2] = OperandTokenType::Label;
//             }
//         };

//         Self {
//             sequence,
//             pointer: 0,
//             index: 0,
//         }
//     }
// }

struct PointerIter {}

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("`{0}`")]
    InvalidInstructionSequence(OperandTokenType),
    #[error("directive|instruction|break")]
    InvalidLabelSequence,
}

#[derive(EnumCount, Copy, Clone, Debug)]
pub enum OperandTokenType {
    Register,
    Comma,
    Label,
    SymbolOrLiteral,
    ParenL,
    ParenR,
    // Negative,
    // Symbol,
    Eol,
}

impl OperandTokenType {
    const fn token_count() -> usize {
        // 1 to remove Eol
        OperandTokenType::VARIANT_COUNT - 1
    }
}

impl Display for OperandTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OperandTokenType::*;
        match self {
            Register => Token::register().fmt(f),
            Comma => Token::Comma.fmt(f),
            Label => Token::Label.fmt(f),
            ParenL => Token::ParenL.fmt(f),
            ParenR => Token::ParenR.fmt(f),
            Eol => Token::Eol.fmt(f),
            SymbolOrLiteral => write!(
                f,
                "{}|{}|{}|{}",
                Token::symbol(),
                Token::LiteralDecimal,
                Token::LiteralHex,
                Token::LiteralBinary
            ),
            // Negative => Token::Negative.fmt(f),
        }
    }
}

impl PartialEq<OperandTokenType> for Token {
    fn eq(&self, other: &OperandTokenType) -> bool {
        use OperandTokenType::*;
        match (self, other) {
            (Token::Identifier(IdentifierType::Register(_)), Register)
            | (Token::Identifier(IdentifierType::Symbol), SymbolOrLiteral)
            | (Token::Label, Label)
            | (Token::LiteralDecimal | Token::LiteralHex | Token::LiteralBinary, SymbolOrLiteral)
            | (Token::ParenR, ParenR)
            | (Token::ParenL, ParenL)
            | (Token::Comma, Comma)
            | (Token::Eol, Eol) => true,
            _ => false,
        }
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
