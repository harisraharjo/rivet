use std::ops::Range;

use isa::operand::{Immediate14, Immediate19};
use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::{
    lexer::{Lexeme, LexemesSlice},
    parser::grammar::OperandRuleType,
    token::{self, LiteralIntegerType},
};

#[derive(Debug, EnumVariants, EnumCount)]
pub enum PseudoMnemonic {
    Nop, //No operation: noop converted into addi x0, x0, 0
    Mv,  //copies value between register: e.g. mv 15, 17 converted into addi a5, a7 0
    Li,  // Load immediate
}

#[derive(Debug)]
pub struct Instruction {
    mnemonic: isa::instruction::Mnemonic,
    operands: Operands,
}

impl Instruction {
    pub fn new(mnemonic: isa::instruction::Mnemonic, operands: Operands) -> Instruction {
        Instruction { mnemonic, operands }
    }
}

#[derive(Debug, Error)]
pub enum OperandError {
    #[error(transparent)]
    ImmediateError(#[from] isa::operand::ImmediateValueError),
}

#[derive(Debug)]
pub struct Operands {
    dest: OperandType,
    src1: OperandType,
    src2: OperandType,
}

pub type Source<'a> = &'a [u8];
impl<'a> TryFrom<(&mut LexemesSlice<'a>, OperandRuleType, Source<'a>)> for Operands {
    type Error = OperandError;

    fn try_from(
        (lexemes, rule, source): (&mut LexemesSlice<'a>, OperandRuleType, Source<'a>),
    ) -> Result<Self, Self::Error> {
        let mut iter = lexemes
            .step_by(OperandRuleType::noises_in_every())
            .map(|lexeme| -> Result<OperandType, _> { (lexeme, rule, source).try_into() });

        Ok(Self {
            dest: iter.next().unwrap().unwrap(),
            src1: iter.next().unwrap()?,
            src2: iter.next().unwrap_or(Ok(Default::default()))?,
        })
    }
}

#[derive(Debug, Default)]
pub enum OperandType {
    Symbol(Range<usize>),
    Label(Range<usize>),
    Register(isa::Register),
    Literal14(isa::operand::Immediate14),
    Literal19(isa::operand::Immediate19),
    #[default]
    None,
}

impl<'a> TryFrom<(Lexeme<'a>, OperandRuleType, Source<'a>)> for OperandType {
    type Error = OperandError;

    fn try_from(
        (lexeme, rule, source): (Lexeme<'a>, OperandRuleType, Source<'a>),
    ) -> Result<Self, Self::Error> {
        use OperandRuleType::*;
        use token::Token::*;

        let token = *lexeme.token();

        match (token, rule) {
            (Identifier(token::IdentifierType::Symbol), _) => {
                Ok(Self::Symbol(lexeme.span().to_owned()))
            }
            (Identifier(token::IdentifierType::Register(r)), _) => Ok(Self::Register(r)),
            (Label, _) => Ok(Self::Label(lexeme.span().to_owned())),
            (LiteralDecimal | LiteralHex | LiteralBinary, R2I | RIR | RI) => {
                let src = source.get(lexeme.span().to_owned()).unwrap();
                let signed_byte = src[0];
                let int_ty = LiteralIntegerType::from(token);
                let bytes =
                    LiteralIntegerType::filter(src, LiteralIntegerType::head_len(int_ty as u8));

                let target = if LiteralIntegerType::is_signed(signed_byte) {
                    let mut vec = vec![0; bytes.len() + 1];
                    vec[0] = b'-';
                    vec[1..].copy_from_slice(bytes);
                    vec
                } else {
                    bytes.to_owned()
                };

                let src_str = std::str::from_utf8(target.as_slice()).unwrap();
                let imm = i32::from_str_radix(src_str, int_ty.base()).unwrap();

                match rule {
                    R2I | RIR => Ok(Self::Literal14(Immediate14::try_from(imm)?)),
                    _ => Ok(Self::Literal19(Immediate19::try_from(imm)?)),
                }
            }
            _ => Ok(Self::None),
        }
    }
}
