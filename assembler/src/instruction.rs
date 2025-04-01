use std::ops::{Index, IndexMut};

use isa::operand::{Immediate14, Immediate19};
use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::{
    interner::StrId,
    // lexer::Lexeme,
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
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, EnumCount)]
pub(crate) enum OperandsIndex {
    Dest,
    Src1,
    Src2,
}

#[derive(Debug)]
/// `[dest, src1, src2]`
pub struct Operands([Operand; OperandsIndex::VARIANT_COUNT]);

impl Operands {
    pub fn new() -> Operands {
        Operands(Default::default())
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &Operand> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Operand> {
        self.0.iter_mut()
    }

    /// Copies all elements from `src` into `self`, using a memcpy. Partial copy is allowed as long as the length of `self` is longer than the `src`
    pub fn copy_from_slice(&mut self, src: &[Operand]) {
        assert!(src.len() <= self.0.len(), "Source slice length is too long",);
        // SAFETY: Basically using the underlying implementation of `copy_from_slice`
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), self.0.as_mut_ptr(), src.len());
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<OperandsIndex> for Operands {
    type Output = Operand;

    fn index(&self, index: OperandsIndex) -> &Self::Output {
        match index {
            OperandsIndex::Dest => &self.0[0],
            OperandsIndex::Src1 => &self.0[1],
            OperandsIndex::Src2 => &self.0[2],
        }
    }
}

impl IndexMut<OperandsIndex> for Operands {
    fn index_mut(&mut self, index: OperandsIndex) -> &mut Self::Output {
        match index {
            OperandsIndex::Dest => &mut self.0[0],
            OperandsIndex::Src1 => &mut self.0[1],
            OperandsIndex::Src2 => &mut self.0[2],
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
//4 bytes
pub enum Operand {
    Symbol(StrId),
    Register(isa::Register),
    Imm14(isa::operand::Immediate14),
    Imm19(isa::operand::Immediate19),
    #[default]
    None,
}

type SourceSlice<'a> = &'a [u8];
impl<'a> TryFrom<(token::Token, OperandRuleType, SourceSlice<'a>)> for Operand {
    type Error = OperandError;

    fn try_from(
        (token, rule, slice): (token::Token, OperandRuleType, SourceSlice<'a>),
    ) -> Result<Self, Self::Error> {
        use OperandRuleType::*;
        use token::Token::*;

        match (token, rule) {
            (Identifier(token::IdentifierType::Symbol), _) => Ok(Self::Symbol(StrId::default())),
            // (Label, _) => Ok(Self::Label(lexeme.span().to_owned())),
            (Identifier(token::IdentifierType::Register(r)), _) => Ok(Self::Register(r)),
            (literal @ (LiteralDecimal | LiteralHex | LiteralBinary), R2I | RIR | RI) => {
                //safety unwrap: guaranteed safe
                let frst_byte = slice[0];
                let int_ty = LiteralIntegerType::from(literal);

                let bytes = slice
                    .get(LiteralIntegerType::prefix_len(frst_byte, int_ty as u8)..)
                    .unwrap();
                let mut buffer = Vec::with_capacity(bytes.len() + 1);
                let radix = std::str::from_utf8({
                    if LiteralIntegerType::is_signed(frst_byte) {
                        buffer.push(b'-');
                        buffer.fill(Default::default());
                        buffer[1..].copy_from_slice(bytes);
                        buffer.as_slice()
                    } else {
                        bytes
                    }
                })
                .unwrap();

                let imm = i32::from_str_radix(radix, int_ty.base())?;
                match rule {
                    R2I | RIR => Ok(Self::Imm14(Immediate14::try_from(imm)?)),
                    _ => Ok(Self::Imm19(Immediate19::try_from(imm)?)),
                }
            }
            _ => Ok(Self::None),
        }
    }
}
