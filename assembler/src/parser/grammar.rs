use std::fmt::Display;

use isa::instruction::{InstructionType, Mnemonic};
use shared::EnumCount;
use thiserror::Error;

use crate::token::{IdentifierType, Token};

pub struct InstructionRule<'a> {
    ty: OperandRuleType,
    sequence: &'a [OperandTokenType],
}

impl<'a> InstructionRule<'a> {
    pub fn new(mnemonic: Mnemonic) -> InstructionRule<'a> {
        let ty = OperandRuleType::from(mnemonic);
        InstructionRule {
            ty,
            sequence: Self::generate_sequence(ty),
        }
    }

    pub fn get(&self, index: usize) -> OperandTokenType {
        self.sequence[index]
    }

    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &OperandTokenType> + ExactSizeIterator + use<'_> {
        self.sequence.iter()
    }

    fn generate_sequence(ty: OperandRuleType) -> &'a [OperandTokenType] {
        match ty {
            OperandRuleType::R3 => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Register,
            ]
            .as_slice(),
            OperandRuleType::R2I => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Immediate,
            ]
            .as_slice(),
            OperandRuleType::R2L => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Label,
            ]
            .as_slice(),
            OperandRuleType::RI => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Immediate,
            ]
            .as_slice(),
            OperandRuleType::RIR => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Immediate,
                OperandTokenType::ParenL,
                OperandTokenType::Register,
                OperandTokenType::ParenR,
            ]
            .as_slice(),
            OperandRuleType::RL => [
                OperandTokenType::Register,
                OperandTokenType::Comma,
                OperandTokenType::Label,
            ]
            .as_slice(),
        }
    }
}

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
    Immediate,
    ParenL,
    ParenR,
    // Symbol,
    Eol,
}

impl Display for OperandTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OperandTokenType::Register => "register",
                OperandTokenType::Comma => "comma",
                OperandTokenType::Label => "label",
                OperandTokenType::Immediate => "decimal|hex|binary",
                OperandTokenType::ParenL => "(",
                OperandTokenType::ParenR => ")",
                OperandTokenType::Eol => "eol", //"\\n|\\r"
                                                // OperandTokenType::Symbol => "symbol",
            }
        )
    }
}

impl PartialEq<OperandTokenType> for Token {
    fn eq(&self, other: &OperandTokenType) -> bool {
        match (self, other) {
            (Token::Identifier(IdentifierType::Register(_)), OperandTokenType::Register) => true,
            (Token::Identifier(IdentifierType::Symbol), OperandTokenType::Immediate) => true,
            (Token::Label, OperandTokenType::Label) => true,
            (Token::LiteralDecimal, OperandTokenType::Immediate) => true,
            (Token::LiteralHex, OperandTokenType::Immediate) => true,
            (Token::LiteralBinary, OperandTokenType::Immediate) => true,
            (Token::ParenR, OperandTokenType::ParenR) => true,
            (Token::ParenL, OperandTokenType::ParenL) => true,
            (Token::Comma, OperandTokenType::Comma) => true,
            (Token::Eol, OperandTokenType::Eol) => true,
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
        match value {
            InstructionType::Arithmetic => Self::R3,
            InstructionType::IA => Self::R2I,
            InstructionType::IJ => Self::R2I,
            InstructionType::IL => Self::RIR,
            InstructionType::S => Self::RIR,
            InstructionType::B => Self::R2L,
            InstructionType::J => Self::RL,
            InstructionType::U => Self::RI,
        }
    }
}

impl From<Mnemonic> for OperandRuleType {
    fn from(value: Mnemonic) -> Self {
        match value {
            Mnemonic::Add => Self::R3,
            Mnemonic::Sub => Self::R3,
            Mnemonic::Mul => Self::R3,
            Mnemonic::And => Self::R3,
            Mnemonic::Or => Self::R3,
            Mnemonic::Xor => Self::R3,
            Mnemonic::Shl => Self::R3,
            Mnemonic::Shr => Self::R3,
            Mnemonic::ShrA => Self::R3,
            Mnemonic::AddI => Self::R2I,
            Mnemonic::Lui => Self::RI,
            Mnemonic::Lw => Self::RIR,
            Mnemonic::Sw => Self::RIR,
            Mnemonic::Syscall => Self::R3,
        }
    }
}

pub enum DirectiveRule {
    D,
    L,
    S,
}
// pub trait RuleType {
//     type Error;
//     fn validate(&self, tokens: &[Token]) -> Result<(), Self::Error>;
// }
