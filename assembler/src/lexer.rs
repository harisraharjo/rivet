use logos::Logos;

use crate::token::{Lexemes, LexingError, Token};

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

            println!(
                "Lexeme: {:?} as {:?}",
                String::from_utf8(unsafe { input.get_unchecked(span.clone()).to_vec() }).unwrap(),
                token
            );
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
        for (&token, span) in tokens.symbols() {
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
