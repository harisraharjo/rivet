use std::ops::Range;

use isa::operand::{Immediate14, Immediate19};
use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::{
    lexer::Lexeme,
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

    // pub fn symbols(&self) -> i32 {
    //     1
    // }

    pub fn is_resolved(&self) -> bool {
        !self.operands.0.iter().any(|o| {
            let od = core::mem::discriminant(o);
            od == core::mem::discriminant(&OperandType::Symbol(0..0))
                || od == core::mem::discriminant(&OperandType::Label(0..0))
        })
    }
}

#[derive(Debug, Error)]
pub enum OperandError {
    #[error(transparent)]
    ImmediateError(#[from] isa::operand::ImmediateValueError),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

// pub(crate) enum OperandsIndex {
//     Dest,
//     Src1,
//     Src2,
// }

#[derive(Debug)]
/// `[dest, src1, src2]`
pub struct Operands([OperandType; 3]);

impl Operands {
    pub fn new() -> Operands {
        Operands(Default::default())
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &OperandType> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut OperandType> {
        self.0.iter_mut()
    }
}

impl FromIterator<OperandType> for Operands {
    fn from_iter<T: IntoIterator<Item = OperandType>>(iter: T) -> Self {
        let mut operands = Operands::new();
        for (l, r) in operands.iter_mut().zip(iter) {
            *l = r;
        }

        operands
    }
}

type Source<'a> = &'a [u8];

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

        match (*lexeme.token(), rule) {
            (Identifier(token::IdentifierType::Symbol), _) => {
                Ok(Self::Symbol(lexeme.span().to_owned()))
            }
            (Label, _) => Ok(Self::Label(lexeme.span().to_owned())),
            (Identifier(token::IdentifierType::Register(r)), _) => Ok(Self::Register(r)),
            (literal @ (LiteralDecimal | LiteralHex | LiteralBinary), R2I | RIR | RI) => {
                //safety unwrap: guaranteed safe
                let src = source.get(lexeme.span().to_owned()).unwrap();
                let signed_byte = src[0];
                let int_ty = LiteralIntegerType::from(literal);
                let base = int_ty.base();

                let imm = i32::from_str_radix(
                    std::str::from_utf8(
                        {
                            let bytes = src
                                .get(LiteralIntegerType::prefix_len(src[0], int_ty as u8)..)
                                .unwrap();

                            if LiteralIntegerType::is_signed(signed_byte) {
                                let mut vec = vec![0; bytes.len() + 1];
                                vec[0] = b'-';
                                vec[1..].copy_from_slice(bytes);
                                vec
                            } else {
                                bytes.to_owned()
                            }
                        }
                        .as_slice(),
                    )
                    .unwrap(),
                    base,
                )?;

                match rule {
                    R2I | RIR => Ok(Self::Literal14(Immediate14::try_from(imm)?)),
                    _ => Ok(Self::Literal19(Immediate19::try_from(imm)?)),
                }
            }
            _ => Ok(Self::None),
        }
    }
}
