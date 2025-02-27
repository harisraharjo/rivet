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

impl Token {
    pub const fn register() -> Token {
        Token::Identifier(IdentifierType::Register(isa::Register::X0))
    }

    pub fn symbol() -> Token {
        Token::Identifier(IdentifierType::Symbol)
    }

    // pub fn symbol() -> Token {
    //     Token::Identifier(IdentifierType::Mn)
    // }
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
pub struct Lexemes {
    tokens: Vec<Token>,
    spans: Vec<Range<usize>>,
}

impl Lexemes {
    pub fn new(capacity: usize) -> Lexemes {
        Lexemes {
            tokens: Vec::with_capacity(capacity),
            spans: Vec::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    pub fn push(&mut self, token: Token, span: Range<usize>) {
        self.tokens.push(token);
        self.spans.push(span);
    }

    pub fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
        self.spans.shrink_to_fit();
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[Token] {
        &self.tokens
    }

    pub fn spans(&self) -> &[Range<usize>] {
        &self.spans
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

    pub fn slice<I>(&self, index: I) -> LexemesSlice<'_>
    where
        I: Clone
            + std::slice::SliceIndex<[Token], Output = [Token]>
            + std::slice::SliceIndex<[Range<usize>], Output = [Range<usize>]>,
    {
        // let add = &self.tokens[index.clone()].iter().zip(&self.spans[index]);
        LexemesSlice::new(&self.tokens[index.clone()], &self.spans[index])
    }
}

pub struct LexemesSlice<'a> {
    tokens: &'a [Token],
    spans: &'a [Range<usize>],
    index: usize,
}

impl<'a> LexemesSlice<'a> {
    fn new(tokens: &'a [Token], spans: &'a [Range<usize>]) -> LexemesSlice<'a> {
        LexemesSlice {
            tokens,
            spans,
            index: 0,
        }
    }
}

impl<'a> Iterator for LexemesSlice<'a> {
    type Item = Lexeme<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tokens.len() {
            return None;
        }

        let lexeme = Lexeme {
            token: &self.tokens[self.index],
            span: &self.spans[self.index],
        };

        self.index += 1;
        Some(lexeme)
    }
}

pub struct Lexeme<'a> {
    token: &'a Token,
    span: &'a Range<usize>,
}

impl Lexeme<'_> {
    pub fn token(&self) -> &Token {
        self.token
    }

    pub fn span(&self) -> &Range<usize> {
        self.span
    }
}
