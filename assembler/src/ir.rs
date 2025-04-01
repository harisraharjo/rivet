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
    #[error("Unknown value")]
    UnknownValue,
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

#[derive(Debug)]
pub struct Exprs {
    buffer: Vec<Expr>,
}

impl Exprs {
    pub fn with_capacity(cap: usize) -> Exprs {
        Exprs {
            buffer: Vec::with_capacity(cap),
            // ops: Vec::with_capacity(cap - 1),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn push(&mut self, value: Expr) {
        self.buffer.push(value);
    }

    pub fn append(&mut self, value: &mut Vec<Expr>) {
        self.buffer.append(value);
    }
}

#[derive(Debug)]
pub enum Variable {
    Symbol(StrId),
    U32(u32),
    I32(i32),
}

impl Variable {
    pub fn new(token: token::Token, slice: &[u8]) -> Result<Self, IRError> {
        match token {
            token::symbol!() => Ok(Self::Symbol(StrId::default())),
            literal @ token::literal_integer!() => {
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
            _ => Err(IRError::UnknownValue),
        }
    }
}

/// Represents an expression parsed from lexemes.
//16 bytes
#[derive(Debug)]
pub enum Expr {
    Var(Variable),
    Operator { op: Op, left: usize, right: usize },
}

/// Supported operators in expressions.
#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    None,
}

impl Op {
    const fn precedence(&self) -> u8 {
        match self {
            Op::Add => 1,
            Op::Sub => 1,
            Op::Mul => 2,
            Op::Div => 2,
            Op::None => 0,
        }
    }
}

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other)) // Delegate to Ord::cmp
    }
}

impl Ord for Op {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.precedence().cmp(&other.precedence())
    }
}

impl From<token::Token> for Op {
    fn from(value: token::Token) -> Self {
        match value {
            token::Token::Negative => Self::Sub,
            token::Token::Positive => Self::Add,
            _ => Self::None,
        }
    }
}

// impl PartialEq for Op {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Op::Add, Op::Add) => true,
//             (Op::Mul, Op::Mul) => true,
//             // (Op::Number(n1), Op::Number(n2)) => n1 == n2,
//             _ => false,
//         }
//     }
// }

// impl Eq for Op {}
