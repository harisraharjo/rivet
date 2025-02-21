use std::{collections::HashMap, fmt::Debug, ops::Range};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SymbolType {
    #[default]
    Label,
    Constant,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Scope {
    Local,
    #[default]
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

#[derive(Debug)]
pub struct SymbolTable {
    entries: HashMap<Range<usize>, Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: Range<usize>, v: Symbol) {
        self.entries.insert(k, v);
    }

    pub fn contains_key(&self, key: &Range<usize>) -> bool {
        self.entries.contains_key(key)
    }
}
