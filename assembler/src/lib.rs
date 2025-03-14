mod asm;
mod instruction;
mod lexer;
mod parser;
mod symbol_table;
mod token;

use std::borrow::Cow;

use lexer::Lexer;
// use parser::Parser;
use symbol_table::{Symbol, SymbolTable};
use thiserror::Error;
use token::LexingError;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexingError),
}

pub struct Assembler {
    // lexer: Lexer,
    // parser: Parser<'a>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Self {
            // symbol_table: SymbolTable::new(),
            // lexer: Lexer::new(),
        }
    }

    pub fn assemble<'source>(&mut self, source: &'source [u8]) -> Result<(), AssemblerError> {
        let mut symbol_table = SymbolTable::new();
        let tokens = Lexer::new().tokenize(source)?;

        for (token, span) in tokens.symbols() {
            symbol_table.insert(
                Cow::Borrowed(source.get(span.to_owned()).unwrap()),
                Symbol::new(Default::default(), None, token.try_into().unwrap()),
            )
        }

        // let mut parser = Parser::new(source, tokens);
        // let data = parser.parse();

        Ok(())
    }
}
