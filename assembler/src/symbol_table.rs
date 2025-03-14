use std::{borrow::Cow, collections::HashMap, fmt::Debug, ops::Range};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SymbolType {
    #[default]
    Label,
    Constant,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Local,
    Global,
}
// .equ GREETING, msg where msg is defined as .ascii "Hello, World!"
//      would make GREETING a label/symbol pointing to the start of the string's memory location.

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Symbol {
    scope: Scope,
    value: Option<u32>,
    ty: SymbolType,
    // active_section: SectionType,
}

impl Symbol {
    pub fn new(scope: Scope, value: Option<u32>, ty: SymbolType) -> Symbol {
        Symbol { scope, value, ty }
    }
}

// #[derive(Debug)]
// pub enum SymbolValue {
//     Constant(i32),      // .equ MAX, 42
//     Macro(Vec<AstNode>),// Reusable code snippets
//     Address(u32),       // Labels resolved to addresses
// }

pub type RawSymbolName<'a> = Cow<'a, [u8]>;
#[derive(Debug)]
pub struct SymbolTable<'a> {
    entries: HashMap<RawSymbolName<'a>, Symbol>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: RawSymbolName<'a>, v: Symbol) {
        self.entries.insert(k, v);
    }

    pub fn contains_key(&self, key: &RawSymbolName<'a>) -> bool {
        self.entries.contains_key(key)
    }
}
