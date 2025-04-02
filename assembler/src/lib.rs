mod asm;
mod exprs;
mod helper;
mod instruction;
mod interner;
mod ir;
mod lexer;
mod parser;
mod symbol_table;
mod token;

use lexer::Lexer;
use parser::Parser;
// use parser::Parser;
use symbol_table::SymbolTable;
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
        let mut symbol_table = SymbolTable::new(source);
        let tokens = Lexer::new().tokenize(source)?;

        let mut parser = Parser::new(source, tokens).parse();
        // let data = parser.parse();

        Ok(())
    }
}
