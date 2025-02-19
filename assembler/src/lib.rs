mod asm;
mod ast;
mod instruction;
mod lexer;
mod parser;
mod symbol_table;

use lexer::{Lexer, LexingError};
use parser::Parser;
use symbol_table::{Symbol, SymbolTable};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexingError),
}

pub struct Assembler<'a> {
    symbol_table: SymbolTable<'a>,
    lexer: Lexer,
    parser: Parser,
    // input: &'source [u8],
}

impl<'a> Assembler<'a> {
    pub fn new() -> Assembler<'a> {
        Assembler {
            symbol_table: SymbolTable::new(),
            lexer: Lexer::new(),
            parser: Parser::new(),
            // input: source,
        }
    }

    fn assembler<'source>(&mut self, source: &'source [u8]) -> Result<(), AssemblerError> {
        let tokens = self.lexer.tokenize(source)?;
        let mut symbol_table = SymbolTable::new();

        for (&token, span) in tokens.symbols() {
            symbol_table.insert(
                &source[span.start..span.end],
                Symbol::new(Default::default(), None, token.try_into().unwrap()),
            )
        }

        // let data = self.parser.parse(tokens);
        Ok(())
    }
}
