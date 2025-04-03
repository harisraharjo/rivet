mod asm;
mod exprs;
mod helper;
mod instruction;
mod interner;
mod ir;
mod layout;
mod lexer;
mod parser;
mod symbol_table;
mod token;

use layout::Layout;
use lexer::Lexer;
use parser::{Parser, ParsingError};
// use parser::Parser;
use symbol_table::SymbolTable;
use thiserror::Error;
use token::LexingError;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexingError),
    #[error("Parser error: {0}")]
    ParserErrorError(#[from] ParsingError),
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
        let lexemes = Lexer::new().tokenize(source)?;
        let mut parsed_data = Parser::new(source, lexemes).parse()?;

        // let layout = Layout::new(parser.);

        Ok(())
    }
}
