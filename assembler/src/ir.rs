use std::{ops::Range, slice::Windows};

use thiserror::Error;

use crate::{
    asm::section::Section,
    instruction::Instruction,
    interner::StrId,
    token::{self, LiteralIntegerType},
};

#[derive(Debug, Error)]
pub enum IRError {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug)]
pub struct Exprs {
    vars: Vec<Expr>,
    ops: Vec<Op>,
}

impl Exprs {
    pub fn with_capacity(cap: usize) -> Exprs {
        Exprs {
            vars: Vec::with_capacity(cap),
            ops: Vec::with_capacity(cap - 1),
        }
    }

    pub fn push(&mut self, value: Expr) {
        self.vars.push(value);
    }
}

// #[derive(Debug)]
// pub enum ExprTy {
//     Symbol,
//     U32,
//     I32,
// }

// #[derive(Debug, Clone, Copy)]
// pub enum ExprLayer<A> {
//     Add { a: A, b: A },
//     Sub { a: A, b: A },
//     Mul { a: A, b: A },
//     IntI32(i32),
//     IntU32(u32),
// }

// #[derive(Eq, Hash, PartialEq)]
// pub struct ExprIdx(usize);
// impl ExprIdx {
//     fn head() -> Self {
//         ExprIdx(0)
//     }
// }

// pub struct ExprTopo {
//     // nonempty, in topological-sorted order. guaranteed via construction.
//     elems: Vec<ExprLayer<ExprIdx>>,
// }

/// Represents an expression parsed from lexemes.
//16 bytes
#[derive(Debug)]
pub enum Expr {
    Symbol(StrId),
    U32(u32),
    I32(i32),
    Operator(Op),
}

type Slice<'a> = &'a [u8];
impl<'a> TryFrom<(token::Token, Slice<'a>)> for Expr {
    type Error = IRError;

    fn try_from((token, slice): (token::Token, Slice<'a>)) -> Result<Self, Self::Error> {
        match token {
            token::symbol!() => Ok(Self::Symbol(StrId::default())),
            token::operator!() => Ok(Self::Operator(token.into())),
            literal @ _ => {
                //safety unwrap: guaranteed safe
                let frst_byte = slice[0];
                let ty = LiteralIntegerType::from(literal);
                let signed = LiteralIntegerType::is_signed(frst_byte);

                let mut buffer = Vec::with_capacity(0);
                let radix = std::str::from_utf8({
                    let bytes = slice
                        .get(LiteralIntegerType::prefix_len(frst_byte, ty as u8)..)
                        .unwrap();

                    if signed {
                        buffer.reserve_exact(bytes.len() + 1);
                        buffer.push(b'-');
                        buffer.fill(Default::default());
                        buffer[1..].copy_from_slice(bytes);
                        buffer.as_slice()
                    } else {
                        bytes
                    }
                })
                .unwrap();

                let base = ty.base();
                if signed {
                    Ok(Self::I32(i32::from_str_radix(radix, base)?))
                } else {
                    Ok(Self::U32(u32::from_str_radix(radix, base)?))
                }
            }
        }
    }
}

/// Supported operators in expressions.
#[derive(Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Noop,
}

impl From<token::Token> for Op {
    fn from(value: token::Token) -> Self {
        match value {
            token::Token::Negative => Self::Sub,
            token::Token::Positive => Self::Add,
            _ => Self::Noop,
        }
    }
}

/// Represents data parsed into a section, using spans for strings.
#[derive(Debug)]
pub enum Node {
    Word(u32),
    Byte(u8),
    Half(u16),
    String(Box<str>),
    Section(Section),
    Instruction(Instruction),
    // Label(Range<usize>),
    Label(StrId),
    Global(StrId),
    // Expr(Exprs),
    Align(u32), // New for .align, .p2align, .balign
    Skip(u32),
}
