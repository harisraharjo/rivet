use std::{collections::HashMap, fmt::Debug};

// use bumpalo::Bump;
use thiserror::Error;

use crate::{
    asm::{directive::DirectiveType, section::SectionType, symbol::SymbolType},
    exprs::Exprs,
    interner::{Interner, StrId},
};

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("{0} is already defined")]
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
pub type SymbolName = StrId;
// pub type SymbolName = &'a [u8];

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Visibility {
    #[default]
    Local,
    Global,
}
// .equ GREETING, msg where msg is defined as .ascii "Hello, World!"
//      would make GREETING a label/symbol pointing to the start of the string's memory location.

#[derive(Debug, PartialEq)]
pub(crate) enum ConstantSymbolDir {
    Set,
    Equ,
}

impl From<DirectiveType> for ConstantSymbolDir {
    fn from(value: DirectiveType) -> Self {
        match value {
            DirectiveType::Set => Self::Set,
            _ => Self::Equ,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConstantSymbol {
    resolved: bool,
}

impl ConstantSymbol {
    pub fn new(value: Exprs) -> ConstantSymbol {
        // TODO: finish constantsymbol
        ConstantSymbol { resolved: false }
    }
}

pub type ConstantsKey = StrId;
#[derive(Debug, Default)]
pub struct ConstantSymbols {
    data: HashMap<ConstantsKey, ConstantSymbol>,
}

impl ConstantSymbols {
    pub fn insert<'a>(
        &mut self,
        name_id: ConstantsKey,
        ty: ConstantSymbolDir,
        value: ConstantSymbol,
        name: &str,
    ) -> Result<(), SymbolError> {
        if ty == ConstantSymbolDir::Equ && self.data.contains_key(&name_id) {
            return Err(SymbolError::DuplicateSymbol(name.to_owned()));
        }
        self.data.entry(name_id).insert_entry(value);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Symbol {
    name: SymbolName,
    visibility: Visibility,
    value: Option<u32>,
    ty: SymbolType,
    // section: SectionType,
    // active_section: SectionType,
}

impl Symbol {
    pub fn new(
        name: SymbolName,
        visibility: Visibility,
        value: Option<u32>,
        // offset: Option<u32>,
        ty: SymbolType,
    ) -> Self {
        Self {
            visibility,
            // offset,
            name,
            value,
            ty,
            // section,
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }
}

#[derive(Debug)]
pub struct GlobalSymbol {
    name: SymbolName,
    index: (Key, usize),
}

#[derive(Debug)]
pub struct SymbolTable<'a> {
    // arena: Bump,
    locals: HashMap<Key, Vec<Symbol>>,
    globals: Vec<GlobalSymbol>,
    pending_globals: Vec<SymbolName>,
    source: &'a [u8],
}

impl<'a> SymbolTable<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            // arena: Bump::new(),
            locals: HashMap::new(),
            globals: Vec::new(),
            pending_globals: Vec::new(),
            source,
        }
    }

    pub fn insert(
        &mut self,
        section: Key,
        mut value: Symbol,
        intern: &Interner,
    ) -> Result<(), SymbolError> {
        let name = value.name().to_owned();
        let global_dupe = self.globals.len() > 0 && self.globals.iter().any(|s| s.name == name);
        if global_dupe {
            return Err(SymbolError::DuplicateSymbol(
                intern.lookup(value.name).to_owned(),
            ));
        }

        let locals = self.locals.entry(section).or_insert_with(Vec::new);

        let local_dupe = locals.iter().any(|s| s.name == name);
        if local_dupe {
            return Err(SymbolError::DuplicateSymbol(
                intern.lookup(value.name).to_owned(),
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

    // pub fn get(&self, name: SymbolName) -> i32 {
    //     //   // Prefer local symbol in current section, then global
    //     //     if let Some(local) = symbols.iter().find(|s| s.name == *name && s.section == current_section && s.visibility == SymbolVisibility::Local) {
    //     //         Some(local.offset)
    //     //     } else if let Some(global) = symbols.iter().find(|s| s.name == *name && s.visibility == SymbolVisibility::Global) {
    //     //         Some(global.offset)
    //     //     } else {
    //     //         None // Unresolved, needs relocation
    //     //     }
    //     1
    // }

    /// Change symbol visibility from local to global
    pub fn declare_global(
        &mut self,
        section: Key,
        name: SymbolName,
        intern: &Interner,
    ) -> Result<(), SymbolError> {
        let global_dupe = self.globals.len() > 0 && self.globals.iter().any(|s| s.name == name);
        if global_dupe {
            return Err(SymbolError::DuplicateSymbol(intern.lookup(name).to_owned()));
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

    pub fn locals(&self) -> &HashMap<SectionType, Vec<Symbol>> {
        &self.locals
    }

    pub fn globals(&self) -> &[GlobalSymbol] {
        self.globals.as_slice()
    }

    pub fn pending_globals(&self) -> &[SymbolName] {
        self.pending_globals.as_slice()
    }

    // #[cfg(test)]
    // pub fn contains_key(&self, name: &[u8], section: Key) -> bool {
    //     self.locals
    //         .get(&section)
    //         .and_then(|v| v.iter().any(|s| s.name == name).into())
    //         .is_some()
    // }
}
