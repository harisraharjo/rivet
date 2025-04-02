mod helper;

use std::fmt::Display;

use logos::Logos;

pub use helper::LiteralIntegerType;
pub use helper::{IdentifierType, LexingError};
use helper::{State, on_directive, on_ident, on_literal_integer, on_newline};

use crate::asm::directive::DirectiveType;

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
    #[regex(r#"-?0b[01]+(?:\w+)?"#, on_literal_integer::<{LiteralIntegerType::Binary as u8}>)]
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
                IdentifierType::Mnemonic(_) => "instruction",
                IdentifierType::Register(_) => "register",
                IdentifierType::Symbol => "symbol",
            },
            Token::Label => "label",
            Token::Directive(_) => "directive",
            Token::LiteralString => "string",
            Token::LiteralDecimal => "decimal",
            Token::LiteralHex => "hex",
            Token::LiteralBinary => "binary",
            Token::Positive => "+",
            Token::Negative => "-",
            Token::ParenR => ")",
            Token::ParenL => "(",
            Token::QuoteSingle => "single quote",
            Token::Comma => "COMMA",
            Token::Colon => "COLON",
            Token::Eol => "EOL",
            Token::Eof => "EOF",
            Token::CommentSingleLine => "#|//",
        };
        write!(f, "{}", value)
    }
}

impl Token {
    pub const fn register() -> Self {
        Self::Identifier(IdentifierType::Register(isa::Register::X0))
    }

    pub const fn mnemonic() -> Self {
        Self::Identifier(IdentifierType::Mnemonic(isa::instruction::Mnemonic::Add))
    }

    pub const fn directive() -> Self {
        Self::Directive(DirectiveType::Text)
    }

    pub const fn symbol() -> Self {
        Self::Identifier(IdentifierType::Symbol)
    }
}

/// Expand to `Token::Identifier(IdentifierType::Symbol)`
macro_rules! symbol {
    () => {
        $crate::token::Token::Identifier($crate::token::IdentifierType::Symbol)
    };
}

pub(crate) use symbol;

/// Expand to `Token::LiteralDecimal | Token::LiteralHex | Token::LiteralBinary`
macro_rules! literal_integer {
    () => {
        $crate::token::Token::LiteralDecimal
            | $crate::token::Token::LiteralHex
            | $crate::token::Token::LiteralBinary
    };
}
pub(crate) use literal_integer;

/// Expand to `Token::LiteralDecimal | Token::LiteralHex | Token::LiteralBinary | Token::Identifier(IdentifierType::Symbol)`
macro_rules! symbol_or_numeric {
    () => {
        $crate::token::literal_integer!()
            | $crate::token::Token::Identifier($crate::token::IdentifierType::Symbol)
    };
}
pub(crate) use symbol_or_numeric;

/// Expand to `Token::Positive | Token::Negative`
macro_rules! operator {
    () => {
        $crate::token::Token::Positive | $crate::token::Token::Negative
    };
}
pub(crate) use operator;

/// Expand to `Token::Eol | Token::Eof`
macro_rules! break_kind {
    () => {
        $crate::token::Token::Eol | $crate::token::Token::Eof
    };
}
pub(crate) use break_kind;

/// Expand to `Token::Directive(DirectiveType::Text) | Token::Directive(DirectiveType::Data) | Token::Directive(DirectiveType::Rodata) | Token::Directive(DirectiveType::Bss)`
macro_rules! section_dir {
    () => {
        $crate::token::Token::Directive(DirectiveType::Text)
            | $crate::token::Token::Directive(DirectiveType::Data)
            | $crate::token::Token::Directive(DirectiveType::Rodata)
            | $crate::token::Token::Directive(DirectiveType::Bss)
            | $crate::token::Token::Directive(DirectiveType::CustomSection)
    };
}
pub(crate) use section_dir;

// impl TryFrom<&Token> for symbol_table::SymbolType {
//     type Error = LexingError;

//     fn try_from(value: &Token) -> Result<Self, Self::Error> {
//         match *value {
//             Token::Label => Ok(symbol_table::SymbolType::Label),
//             Token::Identifier(IdentifierType::Symbol) => Ok(symbol_table::SymbolType::Constant),
//             _ => Err(LexingError::Error),
//         }
//     }
// }
