use std::ops::Range;

use isa::{instruction::Instruction, register::Register};
use logos::{Logos, Source};
use shared::{EnumCount, EnumVariants};
use thiserror::Error;

use crate::symbol_table::{self};

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
    in_block_comments: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid Integer at {0}")]
    InvalidInteger(usize),
    #[error("Invalid suffix {0} at {1}")]
    InvalidSuffix(String, usize),
    #[error("Non Ascii Character at {0}")]
    NonAsciiCharacter(usize),
    #[default]
    #[error("Unknown Syntax")]
    UnknownSyntax,
}

fn on_block_comment(lex: &mut logos::Lexer<Token>) -> logos::Skip {
    lex.extras.in_block_comments = !lex.extras.in_block_comments;
    logos::Skip
}

/// Update the line count and cell. //TODO: column still wrong because can't index to the byte index but maybe i'm wrong
fn on_newline(lex: &mut logos::Lexer<Token>) {
    lex.extras.cell.row += 1;
    lex.extras.cell.column = 1;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IdentifierType {
    Mnemonic,
    Register,
    Symbol,
}

impl IdentifierType {
    fn mnemonics<'a>() -> [&'a str; Instruction::VARIANT_COUNT] {
        Instruction::mnemonics()
    }

    fn registers<'a>() -> [&'a str; Register::VARIANT_COUNT] {
        Register::variants()
    }
}

impl From<&[u8]> for IdentifierType {
    fn from(value: &[u8]) -> Self {
        if let Some(_) = Self::mnemonics().iter().find(|v| v.as_bytes() == value) {
            return Self::Mnemonic;
        };

        if let Some(_) = Self::registers().iter().find(|v| v.as_bytes() == value) {
            return Self::Register;
        };

        Self::Symbol
    }
}

fn extract_ident(lex: &mut logos::Lexer<Token>) -> Result<IdentifierType, LexingError> {
    let value = lex.slice();
    if !value.is_ascii() {
        return Err(LexingError::NonAsciiCharacter(lex.span().end));
    }
    // [a-zA-Z_]\w+
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

enum LiteralIntegerType {
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

fn on_literal_integer<const TYPE: u8>(lex: &mut logos::Lexer<Token>) -> Result<(), LexingError> {
    let slice = lex.slice();

    let callback = LiteralIntegerType::cb(TYPE);
    let len = slice.len();

    //safety: we read the end of the slice so it's always safe
    if callback(unsafe { slice.get_unchecked(len - 1) }) {
        let skip = LiteralIntegerType::skip_by(TYPE);
        //safety: we read until the end so it's always safe
        let target = unsafe { slice.get_unchecked(skip..len) };
        if let Some(i) = target.iter().position(callback) {
            return Err(LexingError::InvalidSuffix(
                //safety: we read until the end so it's always safe
                String::from_utf8(unsafe { target.get_unchecked(i..target.len()) }.to_vec())
                    .unwrap(),
                lex.extras.cell.column,
            ));
        };
    };

    Ok(())
}

#[derive(Logos, Debug, PartialEq, EnumCount, Copy, Clone)]
#[logos(source = [u8])]
#[logos(skip r"[ \t\f]+")] // Ignore this regex pattern between tokens
#[logos(extras = State)]
#[logos(error = LexingError)]
pub enum Token {
    #[regex(r#"[a-zA-Z_]\w+"#, extract_ident)]
    Identifier(IdentifierType),

    #[regex(r#"[a-zA-Z]\w+:"#)]
    // LabelDef(LabelType), //TODO: add numeric label?
    LabelDef,
    #[regex(r#"\.[a-zA-Z]\w+"#)]
    Directive,

    #[regex(r#"\"(\\.|[^\\"])*\""#)] // https://www.lysator.liu.se/c/ANSI-C-grammar-l.html
    LiteralString,
    #[regex(r#"\d+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Decimal as u8}>)]
    LiteralDecimal,
    #[regex(r#"0x[0-9a-fA-F]+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Hex as u8}>)]
    LiteralHex,
    #[regex(r#"0b[01]+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Binary as u8}>)]
    LiteralBinary,

    #[token(b"-")]
    Negative,
    #[token(b"+")]
    Positive,
    #[token(b")")]
    ParenR,
    #[token(b"(")]
    ParenL,
    #[token(b"\'")]
    QuoteSingle,
    // #[token(b".")]
    // Dot,
    #[token(b",")]
    Comma,
    // #[token(b"\"")]
    // QuoteDouble,
    #[token(b":")]
    Colon,
    #[token(b"\n", on_newline, priority = 2)]
    Eol,

    #[regex(r#"(?:;|#|//)[^\n]*"#, logos::skip)]
    CommentSingleLine,
    // #[regex(r#"/*(?:\*|[^*])*\*/"#, logos::skip)]
    // CommentBlock,
    // #[token(b"/*", |lex| { lex.extras.in_block_comments = true; logos::Skip }, priority = 2)]
    // CommentBlockStart,
    // #[token(b"*/", |lex| { lex.extras.in_block_comments = false; logos::Skip }, priority = 3)]
    // CommentBlockEnd,
}

impl TryFrom<Token> for symbol_table::SymbolType {
    type Error = LexingError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::LabelDef => Ok(symbol_table::SymbolType::Label),
            Token::Identifier(IdentifierType::Symbol) => Ok(symbol_table::SymbolType::Constant),
            _ => Err(LexingError::UnknownSyntax),
        }
    }
}

/// Structure of Arrays
pub struct Tokens {
    tokens: Vec<Token>,
    spans: Vec<Range<usize>>,
}

impl Tokens {
    fn new(capacity: usize) -> Tokens {
        Tokens {
            tokens: Vec::with_capacity(capacity),
            spans: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, token: Token, span: Range<usize>) {
        self.tokens.push(token);
        self.spans.push(span);
    }

    fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
        self.spans.shrink_to_fit();
    }

    pub fn buffer(&self) -> &[Token] {
        &self.tokens
    }

    pub fn span(&self, index: usize) -> &Range<usize> {
        &self.spans[index]
    }

    fn iter(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.tokens.iter().zip(&self.spans)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Range<usize>, &Token)> {
        self.spans.iter_mut().zip(&self.tokens)
    }

    pub fn symbols(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.iter().filter(|(&token, ..)| {
            token == Token::LabelDef || token == Token::Identifier(IdentifierType::Symbol)
        })
    }
}

pub struct Lexer;

impl Lexer {
    pub fn new() -> Lexer {
        Lexer
    }

    pub fn tokenize<'a>(&self, input: &'a [u8]) -> Result<Tokens, LexingError> {
        let mut lex = Token::lexer(input);
        let mut tokens = Tokens::new(input.len());

        while let Some(sequence) = lex.next() {
            lex.extras.cell.column += 1;
            let token = sequence?;
            let mut span = lex.span();

            // token.sanitize(&mut span);
            println!(
                "Lexeme: {:?} as {:?}",
                String::from_utf8(unsafe { input.slice_unchecked(span.clone()) }.to_vec()).unwrap(),
                token
            );
            tokens.push(token, span);
        }

        tokens.shrink_to_fit();

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::symbol_table::{Symbol, SymbolTable};
    use std::{fs::File, io::Read};

    #[test]
    fn t_tokenize() {
        let lex = Lexer::new();

        let buffer = match File::open("test.asm") {
            Ok(mut file) => {
                let mut _buffer = Vec::new();
                file.read_to_end(&mut _buffer).unwrap();
                Ok(_buffer)
            }
            Err(e) => {
                println!("File Error: {:?}", e);
                Err(e)
            }
        }
        .unwrap();

        let mut symbol_table = SymbolTable::new();
        let mut test_spans = Vec::new();

        let tokens = lex.tokenize(&buffer).unwrap();
        for (&token, span) in tokens.symbols() {
            symbol_table.insert(
                &buffer[span.start..span.end],
                Symbol::new(Default::default(), None, token.try_into().unwrap()),
            );
            test_spans.push(span);
        }

        println!("Symbol Table: {:?}", symbol_table);

        for span in test_spans {
            let key_slice = &buffer[span.start..span.end];
            assert!(symbol_table.contains_key(key_slice));
        }
    }
}
