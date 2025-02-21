mod asm;
mod instruction;
mod lexer;
mod parser;
mod symbol_table;
mod token;

use lexer::Lexer;
// use parser::Parser;
use symbol_table::{Symbol, SymbolTable};
use thiserror::Error;
use token::{LexingError, Tokens};

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] LexingError),
}

pub struct Assembler {
    symbol_table: SymbolTable,
    // lexer: Lexer,
    // parser: Parser<'a>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            symbol_table: SymbolTable::new(),
            // lexer: Lexer::new(),
            // parser: Parser::new([].as_slice(), Tokens::new(0)),
        }
    }

    pub fn assemble<'source>(&mut self, source: &'source [u8]) -> Result<(), AssemblerError> {
        let tokens = Lexer::new().tokenize(source)?;

        for (&token, span) in tokens.symbols() {
            self.symbol_table.insert(
                //safety: guaranteed to be safe because tokens derivate from the source
                span.to_owned(),
                Symbol::new(Default::default(), None, token.try_into().unwrap()),
            )
        }

        // let mut parser = Parser::new(source, tokens);
        // let data = parser.parse();

        Ok(())
    }
}
