mod helper;

use std::fmt::Display;

use logos::Logos;

pub use helper::{IdentifierType, LexingError};
use helper::{LiteralIntegerType, State, on_directive, on_ident, on_literal_integer, on_newline};

use crate::{asm::directive::DirectiveType, symbol_table};

#[derive(Logos, Debug, PartialEq, Copy, Clone, Default)]
#[logos(source = [u8])]
#[logos(skip r"[ \t\f]+")] // Ignore this regex pattern between tokens
#[logos(extras = State)]
#[logos(error = LexingError)]
pub enum Token {
    #[regex(r#"[a-zA-Z_]\w+"#, on_ident)]
    Identifier(IdentifierType),

    #[regex(r#"[a-zA-Z]\w+:"#)]
    // Label(LabelType), //TODO: add numeric label?
    Label,
    #[regex(r#"\.[a-zA-Z]\w+"#, on_directive)]
    Directive(DirectiveType),

    #[regex(r#"\"(\\.|[^\\"])*\""#)] // https://www.lysator.liu.se/c/ANSI-C-grammar-l.html
    LiteralString,
    #[regex(r#"-?\d+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Decimal as u8}>)]
    LiteralDecimal,
    #[regex(r#"-?0x[0-9a-fA-F]+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Hex as u8}>)]
    LiteralHex,
    #[regex(r#"0b[01]+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Binary as u8}>)]
    LiteralBinary,

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
    #[default]
    #[token(b"\n", on_newline, priority = 2)]
    Eol,
    Eof,

    #[regex(r#"(?:;|#|//)[^\n]*"#, logos::skip)]
    CommentSingleLine,
    // #[regex(r#"/*(?:\*|[^*])*\*/"#, logos::skip)]
    // CommentBlock,
    // #[token(b"/*", |lex| { lex.extras.in_block_comments = true; logos::Skip }, priority = 2)]
    // CommentBlockStart,
    // #[token(b"*/", |lex| { lex.extras.in_block_comments = false; logos::Skip }, priority = 3)]
    // CommentBlockEnd,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Token::Identifier(identifier_type) => match identifier_type {
                IdentifierType::Mnemonic(_) => "INSTRUCTION",
                IdentifierType::Register(_) => "REGISTER",
                IdentifierType::Symbol => "SYMBOL",
            },
            Token::Label => "LABEL",
            Token::Directive(_) => "DIRECTIVE",
            Token::LiteralString => todo!(),
            Token::LiteralDecimal => "DECIMAL",
            Token::LiteralHex => "HEX",
            Token::LiteralBinary => "BINARY",
            Token::Positive => "POSITIVE",
            Token::ParenR => "PARENTHESES",
            Token::ParenL => "PARENTHESES",
            Token::QuoteSingle => "SINGLE QUOTE",
            Token::Comma => "COMMA",
            Token::Colon => "COLON",
            Token::Eol => "EOL",
            Token::Eof => "EOF",
            Token::CommentSingleLine => "",
        };
        write!(f, "{}", value)
    }
}

impl Token {
    pub const fn register() -> Token {
        Token::Identifier(IdentifierType::Register(isa::Register::X0))
    }

    pub fn symbol() -> Token {
        Token::Identifier(IdentifierType::Symbol)
    }
}

impl TryFrom<&Token> for symbol_table::SymbolType {
    type Error = LexingError;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match *value {
            Token::Label => Ok(symbol_table::SymbolType::Label),
            Token::Identifier(IdentifierType::Symbol) => Ok(symbol_table::SymbolType::Constant),
            _ => Err(LexingError::UnknownSyntax),
        }
    }
}
