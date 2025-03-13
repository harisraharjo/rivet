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
        let mut lexemes = Lexemes::new(input.len());

        while let Some(sequence) = lex.next() {
            lex.extras.advance_row();
            let token = sequence.map_err(|e| match e {
                LexingError::Error => {
                    let slice = input.get(lex.span()).unwrap();
                    LexingError::UnknownSyntax(
                        String::from_utf8(slice.to_vec()).unwrap(),
                        lex.extras.cell().row(),
                    )
                }
                _ => e,
            })?;
            lex.extras.set_last_token(token);

            println!(
                "Lexeme: {:?} as {:?}",
                String::from_utf8(unsafe { input.get_unchecked(lex.span()).to_vec() }).unwrap(),
                token
            );
            lexemes.push(token, lex.span());
        }

        lexemes.seal();

        Ok(lexemes)
    }
}

#[derive(Debug)]
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

    pub fn slice(&self, index: Range<usize>) -> LexemesSlice<'_> {
        LexemesSlice::new(&self.tokens[index.clone()], &self.spans[index])
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
            // TODO: should be `end + 1` ??
            self.spans.push(end..end);
            self.tokens.push(Token::Eof);
            self.shrink_to_fit();
        }
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

    pub fn peek(&self) -> Option<Lexeme<'_>> {
        if self.index >= self.tokens.len() {
            return None;
        }

        Some(Lexeme {
            token: &self.tokens[self.index],
            span: &self.spans[self.index],
        })
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn contains(&self, token: &Token) -> bool {
        self.tokens.contains(token)
    }

    pub fn tokens(&self) -> &'a [Token] {
        self.tokens
    }

    pub fn get_token(&self, idx: usize) -> Token {
        self.tokens[idx]
    }

    pub fn reset(&mut self) {
        self.index = 0;
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

// TODO: add row number
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::symbol_table::{Symbol, SymbolTable};

    #[test]
    fn t_tokenize() {
        let lex = Lexer::new();

        let raw_source = r#"
        .section .data
        main: add x1, x2, x3
            0x1000MP # invalid literal bin
            99beto // invalid literal decimal
            0b11kl // invalid literal Binary
            lw x5, 0x1000(x0)
            
            addi x5, x6, 10
            # my_symbol x11, x22, 11 //this is a wrong instruction pattern
            # sw x7, 0x2000(x9) // invalid literal Hex
            # eds0110xFF //valid symbol
            # addi x6, 0x2000(x4)
            # lui x6, 0b111(x4)
            lw x1, 10(x5)
            sw x1, 111(x5)
            lui x1, 0x1212
        "#;

        let mut symbol_table = SymbolTable::new();
        let mut test_spans = Vec::new();

        let source = raw_source.to_string();
        let tokens = lex.tokenize(source.as_bytes()).unwrap();
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
