use shared::RangeChunks;

use crate::{
    interner::StrId,
    ir::IRError,
    lexer::{Lexeme, Lexemes},
    parser::{ParsingError, expect_token, grammar::RuleToken},
    token::{self},
};

#[derive(Debug, Default)]
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

    pub fn build(
        &mut self,
        chunked_range: RangeChunks<usize>,
        lexemes: &Lexemes,
        source: &[u8],
        mut interner: impl FnMut(&str) -> StrId,
        // interner: fn(&str) -> StrId,
    ) -> Result<(), ParsingError> {
        let vars_count = self.buffer.capacity().div_ceil(2);
        let mut ops = Vec::<Expr>::with_capacity(vars_count - 1);
        let mut buffer = Vec::<Op>::with_capacity(ops.capacity());

        for r in Self::check(chunked_range, lexemes) {
            let (var, op) = r?;
            let slice = source.get(var.span().to_owned()).unwrap();
            let mut variable = Variable::new(*var.token(), slice)?;
            if let Variable::Symbol(ref mut str_id) = variable {
                *str_id = interner(std::str::from_utf8(slice).unwrap());
            };
            self.buffer.push(Expr::Var(variable));

            let op = Op::from(*op.token());
            while let Some(top) = buffer.last() {
                if !top.ge(&op) {
                    break;
                }

                let len = self.buffer.len();
                ops.push(Expr::Operator {
                    op: buffer.pop().unwrap(),
                    left: len - 1,
                    right: len - 2,
                });
            }

            buffer.push(op);
        }

        self.buffer.append(&mut ops);

        Ok(())
    }

    fn check(
        chunked_range: RangeChunks<usize>,
        lexemes: &Lexemes,
    ) -> impl Iterator<Item = Result<(Lexeme<'_>, Lexeme<'_>), ParsingError>> {
        chunked_range.map(|chunk| -> Result<(_, _), _> {
            Ok((
                expect_token!(
                    lexemes.get(*chunk.start()),
                    token::symbol_or_numeric!(),
                    RuleToken::SymbolOrNumeric
                )?,
                expect_token!(
                    lexemes.get(*chunk.end()),
                    token::operator!() | token::break_kind!(),
                    RuleToken::OperatorOrBreak
                )?,
            ))
        })
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
                let ty = token::LiteralIntegerType::from(literal);
                let signed = token::LiteralIntegerType::is_signed(frst_byte);

                let mut buffer = Vec::with_capacity(0);
                let radix = std::str::from_utf8({
                    let bytes = slice
                        .get(token::LiteralIntegerType::prefix_len(frst_byte, ty as u8)..)
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
