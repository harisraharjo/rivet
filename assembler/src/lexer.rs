use std::ops::Range;

use logos::Logos;

use crate::token::{IdentifierType, LexingError, Token};

pub struct Lexer;

impl Lexer {
    pub fn new() -> Lexer {
        Lexer
    }

    pub fn tokenize<'a>(&self, input: &'a [u8]) -> Result<Lexemes, LexingError> {
        let mut lex = Token::lexer(input);
        let mut tokens = Lexemes::new(input.len());

        while let Some(sequence) = lex.next() {
            lex.extras.advance_row();
            let token = sequence?;
            let span = lex.span();

            // println!(
            //     "Lexeme: {:?} as {:?}",
            //     String::from_utf8(unsafe { input.get_unchecked(span.clone()).to_vec() }).unwrap(),
            //     token
            // );
            tokens.push(token, span);
        }

        tokens.seal();

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
        for (token, span) in tokens.symbols() {
            symbol_table.insert(
                span.to_owned(),
                Symbol::new(Default::default(), None, token.try_into().unwrap()),
            );
            test_spans.push(span);
        }

        println!("Symbol Table: {:?}", symbol_table);

        for span in test_spans {
            // let key_slice = &buffer[span.start..span.end];
            assert!(symbol_table.contains_key(&span));
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

    pub fn get_token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    pub fn get_span(&self, index: usize) -> &Range<usize> {
        &self.spans[index]
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
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn spans(&self) -> &[Range<usize>] {
        &self.spans
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.tokens.iter().zip(&self.spans)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Range<usize>, &Token)> {
        self.spans.iter_mut().zip(&self.tokens)
    }

    pub fn symbols(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.iter().filter(|&(token, ..)| {
            *token == Token::Label || *token == Token::Identifier(IdentifierType::Symbol)
        })
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn seal(&mut self) {
        if let Some(last_span) = self.spans.last() {
            let end = last_span.end;
            self.spans.push(end..end);
            self.tokens.push(Token::Eof);
            self.shrink_to_fit();
        }
    }

    pub fn slice(&self, index: Range<usize>) -> LexemesSlice<'_> {
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

    /// find a token within a slice. short-circuiting
    pub fn find(&self, predicate: fn(&Token) -> bool) -> Option<Lexeme<'a>> {
        self.tokens
            .iter()
            .position(predicate)
            .and_then(|pos| Some(Lexeme::new(&self.tokens[pos], &self.spans[pos])))
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn contains(&self, token: &Token) -> bool {
        self.tokens.contains(token)
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

    // Override size_hint for clarity (optional, since default works)
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.tokens.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for LexemesSlice<'_> {
    fn len(&self) -> usize {
        self.tokens.len() - self.index
    }
}

#[derive(Debug)]
pub struct Lexeme<'a> {
    token: &'a Token,
    span: &'a Range<usize>,
}

impl<'a> Lexeme<'a> {
    fn new(token: &'a Token, span: &'a Range<usize>) -> Self {
        Self { token, span }
    }

    pub fn token(&self) -> &Token {
        self.token
    }

    pub fn span(&self) -> &Range<usize> {
        self.span
    }
}
