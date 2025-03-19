use std::{collections::HashMap, fmt::Debug};

// use bumpalo::Bump;
use thiserror::Error;

use crate::asm::section::SectionType;

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("Symbol {0} is already defined")]
    DuplicateSymbol(String),
}

// #[derive(Debug)]
// pub enum Symboloffset {
//     Constant(i32),      // .equ MAX, 42
//     Macro(Vec<AstNode>),// Reusable code snippets
//     Address(u32),       // Labels resolved to addresses
// }

// #[derive(Debug)]
pub type Key = SectionType;
pub type NameSource<'a> = &'a [u8];

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Visibility {
    #[default]
    Local,
    Global,
}
// .equ GREETING, msg where msg is defined as .ascii "Hello, World!"
//      would make GREETING a label/symbol pointing to the start of the string's memory location.

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Symbol<'a> {
    name: NameSource<'a>,
    visibility: Visibility,
    offset: Option<u32>,
    // section: SectionType,
    // ty: SymbolType,
    // active_section: SectionType,
}

impl<'a> Symbol<'a> {
    pub fn new(name: NameSource<'a>, visibility: Visibility, offset: Option<u32>) -> Self {
        Self {
            visibility,
            offset,
            name,
            // section,
        }
    }

    pub fn name(&self) -> NameSource<'_> {
        self.name
    }
}

#[derive(Debug)]
pub struct GlobalSymbol<'a> {
    name: NameSource<'a>,
    index: (Key, usize),
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    // arena: Bump,
    locals: HashMap<Key, Vec<Symbol<'a>>>,
    globals: Vec<GlobalSymbol<'a>>,
    pending_globals: Vec<NameSource<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> Self {
        Self {
            // arena: Bump::new(),
            locals: HashMap::new(),
            globals: Vec::new(),
            pending_globals: Vec::new(),
        }
    }

    pub fn insert(&mut self, section: Key, mut value: Symbol<'a>) -> Result<(), SymbolError> {
        let name = value.name;
        let global_dupe = self.globals.len() > 0 && self.globals.iter().any(|s| s.name == name);
        if global_dupe {
            return Err(SymbolError::DuplicateSymbol(
                String::from_utf8(name.to_vec()).unwrap(),
            ));
        }

        let locals = self.locals.entry(section).or_insert_with(Vec::new);

        let local_dupe = locals.iter().any(|s| s.name == name);
        if local_dupe {
            return Err(SymbolError::DuplicateSymbol(
                String::from_utf8(name.to_vec()).unwrap(),
            ));
        }

        let destined_tobe_global = self.pending_globals.iter().any(|p| *p == name);
        if destined_tobe_global {
            value.visibility = Visibility::Global;
            self.globals.push(GlobalSymbol {
                name,
                index: (section, locals.len()),
            });
        }

        locals.push(value);

        Ok(())
    }

    /// Change symbol visibility from local to global
    pub fn declare_global(
        &mut self,
        section: Key,
        name: NameSource<'a>,
    ) -> Result<(), SymbolError> {
        let global_dupe = self.globals.len() > 0 && self.globals.iter().any(|s| s.name == name);
        if global_dupe {
            return Err(SymbolError::DuplicateSymbol(
                String::from_utf8(name.to_vec()).unwrap(),
            ));
        }
        let locals = self.locals.get_mut(&section);
        match locals {
            Some(locals) => {
                let mut locals_iter_mut = locals
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .filter(|(_, s)| s.name == name);

                if let Some((id, symbol)) = locals_iter_mut.next() {
                    symbol.visibility = Visibility::Global;
                    self.globals.push(GlobalSymbol {
                        name,
                        index: (section, id),
                    });

                    return Ok(());
                }
            }
            None => {}
        }

        self.pending_globals.push(name);

        Ok(())
    }

    pub fn locals(&self) -> &HashMap<SectionType, Vec<Symbol<'a>>> {
        &self.locals
    }

    pub fn globals(&self) -> &[GlobalSymbol<'a>] {
        self.globals.as_slice()
    }

    pub fn pending_globals(&self) -> &[NameSource<'a>] {
        self.pending_globals.as_slice()
    }

    // #[cfg(test)]
    pub fn contains_key(&self, name: &[u8], section: Key) -> bool {
        self.locals
            .get(&section)
            .and_then(|v| v.iter().any(|s| s.name == name).into())
            .is_some()
    }
}
