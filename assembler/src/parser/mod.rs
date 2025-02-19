use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),
    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),
}

pub struct Parser;

impl Parser {
    // pub fn new<'a>(tokens: impl Iterator<Item = impl Debug + 'a>) -> Parser {
    //     Parser
    // }
    pub fn new() -> Parser {
        Parser
    }

    pub fn parse<'source>(&self, tokens: &'source [u8]) -> i32 {
        1
    }
}

enum Grammar {
    Label,
    LabelInstruction,
    LabelDirective,
    Instruction,
    Directive,
}

enum Type {
    Line,
    Block,
}

// Identifier could be: register | mnemonic | symbol
enum Identifier {
    Mnemonic,
    Register,
    Symbol,
}

// pub struct Dfa<T> {}
