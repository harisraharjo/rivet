use std::{
    num::NonZeroUsize,
    ops::{Add, Range},
    slice::Windows,
};

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
                LexingError::Error => LexingError::UnknownSyntax(
                    String::from_utf8(input.get(lex.span()).unwrap().to_vec()).unwrap(),
                    lex.extras.cell().row(),
                ),
                _ => e,
            })?;
            lex.extras.set_last_token(token);
            lexemes.push(token, self.filter_span(token, lex.span()));
        }

        lexemes.seal();

        Ok(lexemes)
    }

    fn filter_span(&self, token: Token, mut span: Range<usize>) -> Range<usize> {
        match token {
            Token::Label => {
                span.end -= 1;
            }
            Token::Directive(_) => {
                span.start += 1;
            }
            Token::LiteralString => {
                span.start += 1;
                span.end -= 1;
            }
            // t @ Token::LiteralBinary | Token::LiteralHex =>{
            //     let len = LiteralIntegerType::prefix_len(source[0],LiteralIntegerType::from_const( t));

            // },
            // Token::Negative => todo!(),
            // Token::Positive => todo!(),
            _ => {}
        };

        span
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

    #[inline(always)]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn spans(&self) -> &[Range<usize>] {
        &self.spans
    }

    pub fn get(&self, index: usize) -> Option<Lexeme<'_>> {
        self.tokens.get(index).and_then(|token| {
            Some(Lexeme {
                token,
                span: self.spans.get(index).unwrap(),
            })
        })
    }

    pub fn get_unchecked(&self, index: usize) -> Lexeme<'_> {
        Lexeme::new(&self.tokens[index], &self.spans[index])
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

    pub fn slice(&self, range: Range<usize>) -> Option<LexemesSlice<'_>> {
        self.tokens
            .get(range.clone())
            .and_then(|slice| Some(LexemesSlice::new(slice, &self.spans[range.clone()], range)))
    }

    // pub fn iter(&self) -> LexemesIter<'_> {
    //     LexemesIter::new(self.tokens.as_slice(), self.spans.as_slice())
    // }

    pub fn symbols(&self) -> impl Iterator<Item = (&Token, &Range<usize>)> {
        self.tokens.iter().zip(&self.spans).filter(|&(token, ..)| {
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

    pub fn shrink_to_fit(&mut self) {
        self.tokens.shrink_to_fit();
        self.spans.shrink_to_fit();
    }
}

pub struct LexemesSlice<'a> {
    tokens: &'a [Token],
    spans: &'a [Range<usize>],
    index: usize,
    range_index: Range<usize>,
}

impl<'a> LexemesSlice<'a> {
    fn new(
        tokens: &'a [Token],
        spans: &'a [Range<usize>],
        range_index: Range<usize>,
    ) -> LexemesSlice<'a> {
        LexemesSlice {
            tokens,
            spans,
            index: 0,
            range_index,
        }
    }

    // pub fn iter(&self, ) -> LexemesIter<'_> {
    //     LexemesIter::new(self.tokens, self.spans)
    // }

    /// find a token within a slice. short-circuiting
    pub fn find_token(&self, predicate: fn(&Token) -> bool) -> Option<Lexeme<'_>> {
        self.tokens
            .iter()
            .position(predicate)
            .and_then(|pos| Some(Lexeme::new(&self.tokens[pos], &self.spans[pos])))
    }

    pub fn peek(&self) -> Option<Lexeme<'_>> {
        if self.index >= self.tokens.len() {
            return None;
        }

        let next_id = self.index + 1;
        Some(self.get(next_id))
    }

    /// Peek into the next `N` index
    pub fn peek_n(&self, n: usize) -> Option<Lexeme<'_>> {
        let next_id = self.index + n;
        if next_id >= self.tokens.len() {
            return None;
        }

        Some(self.get(next_id))
    }

    pub fn token_len(&self) -> usize {
        self.tokens.len()
    }
    pub fn tokens(&self) -> &'a [Token] {
        self.tokens
    }

    pub fn get_token(&self, idx: usize) -> Token {
        self.tokens[idx]
    }

    pub fn get(&self, index: usize) -> Lexeme<'_> {
        Lexeme::new(&self.tokens[index], &self.spans[index])
    }

    /// reset the iterator index
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// reset the iterator index to given index. It will reset to the last index if the specified index is larger than the length or less than 0
    pub fn reset_to(&mut self, index: usize) {
        self.index = std::cmp::min(std::cmp::max(0, index), self.tokens.len());
    }

    /// reset the iterator index to given index
    pub fn reset_to_unchecked(&mut self, index: usize) {
        self.index = index;
    }

    /// Range index of the slice in the original `Lexemes`
    pub fn range_index(&self) -> &Range<usize> {
        &self.range_index
    }

    /// Range index for the remainder of the iterator
    pub fn remainder_range(&self) -> Range<usize> {
        // self.range_index.clone().skip(self.index)
        let start = self.range_index.start + self.index;
        start..self.range_index.end + 1
    }

    // pub fn peek_remainder(&self) -> i32 {
    //     &self.lexemes.tokens()[self.index..]
    // }
}

impl<'a> Iterator for LexemesSlice<'a> {
    type Item = Lexeme<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tokens.len() {
            return None;
        }

        let lex = Lexeme {
            token: &self.tokens[self.index],
            span: &self.spans[self.index],
        };

        self.index += 1;

        Some(lex)
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

        // let mut symbol_table = SymbolTable::new();
        // let mut test_spans = Vec::new();

        let source = raw_source.as_bytes();
        let tokens_result = lex.tokenize(source);
        assert!(tokens_result.is_err());
    }
}
