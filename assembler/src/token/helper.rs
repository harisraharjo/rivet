use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::asm::directive::DirectiveType;

use super::Token;

#[derive(Debug)]
struct Cell {
    row: usize,
    column: usize,
}

impl Default for Cell {
    fn default() -> Self {
        Self { row: 1, column: 1 }
    }
}

#[derive(Default, Debug)]
pub struct State {
    cell: Cell,
    // in_block_comments: bool,
}

impl State {
    pub fn advance_row(&mut self) {
        self.cell.row += 1;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid Integer at {0}")]
    InvalidInteger(usize),
    #[error("Unknown directive {0} at {1}")]
    UnknownDirective(String, usize),
    #[error("Invalid suffix {0} at {1}")]
    InvalidSuffix(String, usize),
    #[error("Invalid Ascii Character at {0}")]
    NonAsciiCharacter(usize),
    #[default]
    #[error("Unknown Syntax")]
    UnknownSyntax,
}

// pub(super) fn on_block_comment(lex: &mut logos::Lexer<Token>) -> logos::Skip {
//     lex.extras.in_block_comments = !lex.extras.in_block_comments;
//     logos::Skip
// }

/// Update the line count and cell. //TODO: column still can't be used because logos doesn't allow us to index to the byte index but maybe i'm wrong
pub(super) fn on_newline(lex: &mut logos::Lexer<Token>) {
    lex.extras.cell.row += 1;
    lex.extras.cell.column = 1;
}

type Index = usize;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IdentifierType {
    Mnemonic(Index),
    Register(isa::Register),
    Symbol,
}

impl IdentifierType {
    #[inline(always)]
    fn mnemonics<'a>() -> [&'a str; isa::Instruction::VARIANT_COUNT] {
        isa::Instruction::mnemonics()
    }

    #[inline(always)]
    fn registers<'a>() -> [&'a str; isa::Register::VARIANT_COUNT] {
        isa::Register::variants()
    }
}

impl From<&[u8]> for IdentifierType {
    fn from(value: &[u8]) -> Self {
        if let Some(i) = Self::mnemonics().iter().position(|v| v.as_bytes() == value) {
            return Self::Mnemonic(i);
        };

        if let Some(i) = Self::registers().iter().position(|v| v.as_bytes() == value) {
            return Self::Register(
                // Safety: guaranteed to be safe because `i` is an actual index from the selected variant.
                unsafe { std::mem::transmute::<u8, isa::Register>(i as u8) },
            );
        };

        Self::Symbol
    }
}

pub(super) fn on_ident(lex: &mut logos::Lexer<Token>) -> Result<IdentifierType, LexingError> {
    let value = lex.slice();
    if !value.is_ascii() {
        // let cow = String::from_utf8_lossy(value);
        return Err(LexingError::NonAsciiCharacter(lex.extras.cell.row));
    }

    Ok(value.into())
}

fn on_decimal(b: &u8) -> bool {
    !b.is_ascii_digit()
}
fn on_bin(b: &u8) -> bool {
    *b != b'0' && *b != b'1'
}
fn on_hex(b: &u8) -> bool {
    !b.is_ascii_hexdigit()
}

pub enum LiteralIntegerType {
    Decimal,
    Hex,
    Binary,
}

impl LiteralIntegerType {
    const fn cb(id: u8) -> for<'a> fn(&'a u8) -> bool {
        match id {
            0 => on_decimal,
            1 => on_hex,
            _ => on_bin,
        }
    }

    const fn skip_by(id: u8) -> usize {
        match id {
            0 => 1,
            _ => 2,
        }
    }
}

pub(super) fn on_literal_integer<const TYPE: u8>(
    lex: &mut logos::Lexer<Token>,
) -> Result<(), LexingError> {
    let slice = lex.slice();

    //safety: we read the end of the slice so it's always safe
    let target = unsafe { slice.get_unchecked(LiteralIntegerType::skip_by(TYPE)..) };
    let callback = LiteralIntegerType::cb(TYPE);
    if let Some(i) = target.iter().position(callback) {
        return Err(LexingError::InvalidSuffix(
            //safety: we read until the end so it's always safe
            String::from_utf8(unsafe { target.get_unchecked(i..target.len()).to_vec() }).unwrap(),
            lex.extras.cell.column,
        ));
    }

    Ok(())
}

pub(super) fn on_directive(lex: &mut logos::Lexer<Token>) -> Result<DirectiveType, LexingError> {
    let slice = lex.slice();
    let variants = DirectiveType::variants();

    //safety: we read the end of the slice so it's always safe
    let target = unsafe { slice.get_unchecked(1..) };
    if let Some(i) = variants.iter().position(|v| v.as_bytes() == target) {
        return Ok(
            // Safety: guaranteed to be safe because `i` is an actual index from the selected variant and DirectiveTypes variants are all unit variant.
            unsafe { std::mem::transmute::<u8, DirectiveType>(i as u8) },
        );
    };

    Err(LexingError::UnknownDirective(
        String::from_utf8(slice.to_vec()).unwrap(),
        lex.extras.cell.row,
    ))
}
