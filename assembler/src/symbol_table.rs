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
    name_id: StrId,
    resolved: bool,
    exprs: Exprs,
}

impl ConstantSymbol {
    pub fn new(name_id: StrId, value: Exprs) -> ConstantSymbol {
        // TODO: finish constantsymbol
        ConstantSymbol {
            resolved: false,
            name_id,
            exprs: value,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConstantSymbols {
    constants: FxHashMap<StrId, ConstantSymbol>,
}

impl ConstantSymbols {
    fn insert(
        &mut self,
        ty: ConstantSymbolDir,
        constant: ConstantSymbol,
        name: &str,
    ) -> Result<(), SymbolError> {
        if ty == ConstantSymbolDir::Equ && self.constants.contains_key(&constant.name_id) {
            return Err(SymbolError::DuplicateSymbol(name.to_owned()));
        }

        self.constants.insert(constant.name_id, constant);
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

#[derive(Debug)]
struct GlobalHandle {
    section_idx: SectionId,
    local_idx: usize,
}

#[derive(Debug)]
pub struct GlobalSymbol {
    name: StrId,
    handle: Option<GlobalHandle>,
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

pub trait DupeCheck<'a, T> {
    type Output;
    /// Check duplicate
    fn dupe_check(&mut self, name: StrId, error_str: &str) -> Result<Self::Output, SymbolError>
    where
        T: 'a;
}

impl<'a, T> DupeCheck<'a, T> for std::slice::Iter<'a, T>
where
    T: SymbolKind,
{
    type Output = ();

    fn dupe_check(&mut self, name: StrId, error_str: &str) -> Result<Self::Output, SymbolError>
    where
        T: 'a,
    {
        if self.any(|s| s.name() == name) {
            return Err(SymbolError::DuplicateSymbol(error_str.to_owned()));
        }

        Ok(())
    }
}

impl<'a> DupeCheck<'a, GlobalSymbol> for std::slice::IterMut<'a, GlobalSymbol> {
    type Output = Option<&'a mut GlobalSymbol>;

    fn dupe_check(&mut self, name: StrId, error_str: &str) -> Result<Self::Output, SymbolError>
    where
        GlobalSymbol: 'a,
    {
        let dupe = self.find(|s| s.name() == name);
        let Some(sym) = dupe else {
            return Ok(None);
        };

        let Some(_) = &sym.handle else {
            return Ok(Some(sym));
        };

        Err(SymbolError::DuplicateSymbol(error_str.to_owned()))
    }
}

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
impl SymbolKind for SymbolName {
    fn name(&self) -> StrId {
        self.0
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    locals: FxHashMap<Key, Vec<Symbol>>,
    globals: Vec<GlobalSymbol>,
    constants: ConstantSymbols,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            locals: FxHashMap::default(),
            globals: Vec::new(),
            constants: ConstantSymbols::default(),
        }
    }

    pub fn insert(
        &mut self,
        section: Key,
        name: StrId,
        error_str: &str,
    ) -> Result<(), SymbolError> {
        let pending_global = self.globals.iter_mut().dupe_check(name, error_str)?;

        let locals = self.locals.entry(section).or_insert_with(Vec::new);
        locals.as_slice().iter().dupe_check(name, error_str)?;

        let mut symbol = Symbol::new(name, Default::default(), None, Default::default());
        if let Some(pg) = pending_global {
            symbol.vis = Visibility::Global;
            pg.handle = Some(GlobalHandle {
                section_idx: section,
                local_idx: locals.len(),
            });
        }

        locals.push(symbol);

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
        let mut global_symbol = GlobalSymbol { name, handle: None };

        if let Some(section_locals) = self.locals.get_mut(&section) {
            let local = section_locals
                .iter_mut()
                .enumerate()
                .rev()
                .find(|(_, sym)| sym.name == name);

            if let Some((_, symbol)) = local {
                symbol.vis = Visibility::Global;
                global_symbol.handle = Some(GlobalHandle {
                    section_idx: section,
                    local_idx: section_locals.len(),
                });
            };
        }

        self.globals.push(global_symbol);

        Ok(())
    }

    pub fn insert_constant(
        &mut self,
        ty: ConstantSymbolDir,
        str_id: StrId,
        value: Exprs,
        name: &str,
    ) -> Result<(), SymbolError> {
        self.constants
            .insert(ty, ConstantSymbol::new(str_id, value), name)
    }

    pub fn locals(&self) -> &FxHashMap<SectionId, Vec<Symbol>> {
        &self.locals
    }

    pub fn globals(&self) -> &[GlobalSymbol] {
        self.globals.as_slice()
    }

    pub fn pending_global(&self, name: StrId) -> Option<&GlobalSymbol> {
        self.globals
            .iter()
            .find(|global| global.handle.is_none() && global.name == name)
    }

    // pub fn get(&self, name: StrId) -> i32 {
    //     // TODO: Label take precedence over constant
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
}
