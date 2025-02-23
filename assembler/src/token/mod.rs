mod helper;

use logos::Logos;
use std::ops::Range;

pub use helper::{IdentifierType, LexingError};
use helper::{LiteralIntegerType, State, on_directive, on_ident, on_literal_integer, on_newline};

use crate::{asm::directive::DirectiveType, symbol_table};

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
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

impl TryFrom<Token> for symbol_table::SymbolType {
    type Error = LexingError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Label => Ok(symbol_table::SymbolType::Label),
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
    pub fn new(capacity: usize) -> Tokens {
        Tokens {
            tokens: Vec::with_capacity(capacity),
            spans: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    pub fn get_unchecked(&self, index: usize) -> Token {
        self.tokens[index]
    }

    pub fn push(&mut self, token: Token, span: Range<usize>) {
        self.tokens.push(token);
        self.spans.push(span);
    }

    pub fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
        self.spans.shrink_to_fit();
    }

    pub fn buffer(&self) -> &[Token] {
        &self.tokens
    }

    pub fn span(&self, index: usize) -> &Range<usize> {
        &self.spans[index]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.tokens.iter().zip(&self.spans)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Range<usize>, &Token)> {
        self.spans.iter_mut().zip(&self.tokens)
    }

    pub fn symbols(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.iter().filter(|&(&token, ..)| {
            token == Token::Label || token == Token::Identifier(IdentifierType::Symbol)
        })
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn seal(&mut self) {
        self.tokens.push(Token::Eof);
        self.spans.push(0..0);
        self.shrink_to_fit();
    }
}
