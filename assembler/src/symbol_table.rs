use std::fmt::Debug;

use rustc_hash::FxHashMap;

use thiserror::Error;

use crate::{
    asm::{directive::DirectiveType, section::SectionId, symbol::SymbolType},
    exprs::Exprs,
    interner::StrId,
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
    name: StrId,
    resolved: bool,
}

impl ConstantSymbol {
    pub fn new(name: StrId, value: Exprs) -> ConstantSymbol {
        // TODO: finish constantsymbol
        ConstantSymbol {
            resolved: false,
            name,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConstantSymbols {
    data: Vec<ConstantSymbol>,
}

impl ConstantSymbols {
    fn try_push<'a>(
        &mut self,
        ty: ConstantSymbolDir,
        value: ConstantSymbol,
        name: &str,
    ) -> Result<(), SymbolError> {
        if ty == ConstantSymbolDir::Equ {
            self.data.iter().dupe_check(value.name, name)?;
        }

        self.data.push(value);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Symbol {
    name: StrId,
    vis: Visibility,
    value: Option<u32>,
    ty: SymbolType,
}

impl Symbol {
    pub fn new(
        name: StrId,
        vis: Visibility,
        value: Option<u32>,
        // offset: Option<u32>,
        ty: SymbolType,
    ) -> Self {
        Self {
            vis,
            // offset,
            name,
            value,
            ty,
            // section,
        }
    }

    pub fn name(&self) -> StrId {
        self.name
    }
}

pub type Key = SectionId;
pub type SymTabIdx = usize;
#[derive(Debug)]
pub struct GlobalSymbol {
    name: StrId,
    index: (Key, SymTabIdx),
}

#[derive(Debug)]
pub struct SymbolName(StrId);
impl From<StrId> for SymbolName {
    fn from(value: StrId) -> Self {
        Self(value)
    }
}

pub trait SymbolKind {
    fn name(&self) -> StrId;
}

pub trait DupeCheck<T> {
    /// Check duplicate
    fn dupe_check<'a>(mut self, name: StrId, error_str: &str) -> Result<(), SymbolError>
    where
        Self: Iterator<Item = &'a T> + Sized,
        T: SymbolKind + 'a,
    {
        if self.any(|s| s.name() == name) {
            return Err(SymbolError::DuplicateSymbol(error_str.to_owned()));
        }

        Ok(())
    }
}

impl<T> DupeCheck<T> for std::slice::Iter<'_, T> {}
impl SymbolKind for Symbol {
    fn name(&self) -> StrId {
        self.name
    }
}
impl SymbolKind for GlobalSymbol {
    fn name(&self) -> StrId {
        self.name
    }
}
impl SymbolKind for ConstantSymbol {
    fn name(&self) -> StrId {
        self.name
    }
}
impl SymbolKind for SymbolName {
    fn name(&self) -> StrId {
        self.0
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    locals: FxHashMap<Key, Vec<Symbol>>,
    globals: Vec<GlobalSymbol>,
    pending_globals: Vec<SymbolName>,
    constants: ConstantSymbols,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            locals: FxHashMap::default(),
            globals: Vec::new(),
            pending_globals: Vec::new(),
            constants: ConstantSymbols::default(),
        }
    }

    pub fn insert(
        &mut self,
        section: Key,
        name: StrId,
        error_str: &str,
    ) -> Result<(), SymbolError> {
        self.globals.iter().dupe_check(name, error_str)?;
        let mut value = Symbol::new(name, Default::default(), None, Default::default());

        let locals = self.locals.entry(section).or_insert_with(Vec::new);
        locals.as_slice().iter().dupe_check(name, error_str)?;

        let pending = self
            .pending_globals
            .iter()
            .dupe_check(name, error_str)
            .is_err();
        if pending {
            value.vis = Visibility::Global;
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
        name: StrId,
        error_str: &str,
    ) -> Result<(), SymbolError> {
        self.globals.iter().dupe_check(name, error_str)?;
        println!("GLOBALIZED: {:?}", name);

        let locals = self.locals.get_mut(&section);
        match locals {
            Some(locals) => {
                let local = locals
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .find(|(_, s)| s.name == name);

                if let Some((id, symbol)) = local {
                    symbol.vis = Visibility::Global;
                    self.globals.push(GlobalSymbol {
                        name,
                        index: (section, id),
                    });

                    return Ok(());
                }
            }
            None => {}
        }

        self.pending_globals.push(name.into());

        Ok(())
    }

    pub fn insert_constant(
        &mut self,
        ty: ConstantSymbolDir,
        value: ConstantSymbol,
        name: &str,
    ) -> Result<(), SymbolError> {
        self.constants.try_push(ty, value, name)
    }

    pub fn locals(&self) -> &FxHashMap<SectionId, Vec<Symbol>> {
        &self.locals
    }

    pub fn globals(&self) -> &[GlobalSymbol] {
        self.globals.as_slice()
    }

    pub fn pending_globals(&self) -> &[SymbolName] {
        self.pending_globals.as_slice()
    }

    pub fn get(&self, name: StrId) -> i32 {
        // TODO: Label take precedence over constant
        //   // Prefer local symbol in current section, then global
        //     if let Some(local) = symbols.iter().find(|s| s.name == *name && s.section == current_section && s.visibility == SymbolVisibility::Local) {
        //         Some(local.offset)
        //     } else if let Some(global) = symbols.iter().find(|s| s.name == *name && s.visibility == SymbolVisibility::Global) {
        //         Some(global.offset)
        //     } else {
        //         None // Unresolved, needs relocation
        //     }
        1
    }

    // #[cfg(test)]
    // pub fn contains_key(&self, name: &[u8], section: Key) -> bool {
    //     self.locals
    //         .get(&section)
    //         .and_then(|v| v.iter().any(|s| s.name == name).into())
    //         .is_some()
    // }
}
