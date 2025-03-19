use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::asm::directive::DirectiveType;

use super::Token;

#[derive(Debug)]
pub struct Cell {
    row: usize,
    column: usize,
}

impl Cell {
    pub fn row(&self) -> usize {
        self.row
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self { row: 1, column: 1 }
    }
}

#[derive(Default, Debug)]
pub struct State {
    cell: Cell,
    last_token: Token,
    // in_block_comments: bool,
    // custom_section_count: usize
}

impl State {
    pub fn advance_row(&mut self) {
        self.cell.row += 1;
    }

    pub fn set_last_token(&mut self, token: Token) {
        self.last_token = token;
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    // #[error("Integer error: {0} at {1}")]
    // IntegerError(#[source] ParseIntError, usize),
    #[error("Unknown directive {0} at {1}")]
    UnknownDirective(String, usize),
    #[error("Invalid suffix {0} at {1}")]
    InvalidSuffix(String, usize),
    #[error("Invalid Ascii Character at {0}")]
    NonAsciiCharacter(usize),
    #[error("Unknown syntax {0} at row {0}")]
    UnknownSyntax(String, usize),
    #[default]
    #[error("")]
    Error,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IdentifierType {
    Mnemonic(isa::instruction::Mnemonic),
    Register(isa::Register),
    Symbol,
}

impl IdentifierType {
    #[inline(always)]
    fn mnemonics<'a>() -> [&'a str; isa::Instruction::VARIANT_COUNT] {
        isa::instruction::Mnemonic::variants()
    }

    // #[inline(always)]
    // fn pseudo_mnemonics<'a>() -> [&'a str; crate::instruction::PseudoMnemonic::VARIANT_COUNT] {
    //     crate::instruction::PseudoMnemonic::variants()
    // }

    #[inline(always)]
    fn registers<'a>() -> [&'a str; isa::Register::VARIANT_COUNT] {
        isa::Register::variants()
    }
}

impl From<&[u8]> for IdentifierType {
    fn from(value: &[u8]) -> Self {
        if let Some(i) = Self::mnemonics().iter().position(|v| v.as_bytes() == value) {
            return Self::Mnemonic(
                // Safety: guaranteed to be safe because `i` is an actual index from the selected variant.
                unsafe { std::mem::transmute::<u8, isa::instruction::Mnemonic>(i as u8) },
            );
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

pub(super) fn on_ident(lex: &mut logos::Lexer<Token>) -> IdentifierType {
    lex.slice().into()
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

#[derive(Debug, Clone, Copy)]
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

    /// The length of the prefix e.g. hex: `0x`, bin: `0b`
    pub fn prefix_len(first_byte: u8, id: u8) -> usize {
        let mut prefix = match id {
            0 => id as usize,
            _ => 2,
        };

        if Self::is_signed(first_byte) {
            prefix += 1;
        }

        prefix
    }

    pub const fn base(&self) -> u32 {
        match self {
            LiteralIntegerType::Decimal => 10,
            LiteralIntegerType::Hex => 16,
            LiteralIntegerType::Binary => 2,
            // LiteralIntegerType::Unknown => 0,
        }
    }

    pub fn is_signed(byte: u8) -> bool {
        byte == b'-'
    }

    pub const fn from_const(token: Token) -> Self {
        match token {
            Token::LiteralHex => LiteralIntegerType::Hex,
            Token::LiteralBinary => LiteralIntegerType::Binary,
            _ => LiteralIntegerType::Decimal,
        }
    }
}

impl From<Token> for LiteralIntegerType {
    fn from(value: Token) -> Self {
        match value {
            Token::LiteralHex => Self::Hex,
            Token::LiteralBinary => Self::Binary,
            _ => Self::Decimal,
        }
    }
}

pub(super) fn on_literal_integer<const TYPE: u8>(
    lex: &mut logos::Lexer<Token>,
) -> Result<(), LexingError> {
    // assert!(TYPE <= LiteralIntegerType::COUNT);

    let slice = lex.slice();
    let target = slice
        .get(LiteralIntegerType::prefix_len(slice[0], TYPE)..)
        .unwrap();

    let callback = LiteralIntegerType::cb(TYPE);
    if let Some(i) = target.iter().position(callback) {
        return Err(LexingError::InvalidSuffix(
            //safety: we read until the end so it's always safe
            String::from_utf8(target.get(i..target.len()).unwrap().to_vec()).unwrap(),
            lex.extras.cell.column,
        ));
    }

    Ok(())
}

pub(super) fn on_directive(lex: &mut logos::Lexer<Token>) -> Result<DirectiveType, LexingError> {
    let slice = lex.slice();
    let variants = DirectiveType::variants();

    // start from index 1 because the dot `.` is at index 0
    let target = slice.get(1..).unwrap();
    if let Some(i) = variants.iter().position(|v| v.as_bytes() == target) {
        return Ok(
            // Safety: guaranteed to be safe because `i` is an actual index from the selected variant and DirectiveTypes variants are all unit variant.
            unsafe { std::mem::transmute::<u8, DirectiveType>(i as u8) },
        );
    };

    // // approve user defined section if and only if the prev token is `.section`
    // if lex.extras.last_token == Token::Directive(DirectiveType::Section) {
    //     return Ok(DirectiveType::CustomSection);
    // }

    Err(LexingError::UnknownDirective(
        String::from_utf8(slice.to_vec()).unwrap(),
        lex.extras.cell.row,
    ))
}
