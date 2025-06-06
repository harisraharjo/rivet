use thiserror::Error;

use crate::{
    asm::section::{Section, SectionId, SectionTag, SectionType},
    instruction::Instruction,
    interner::{Interner, StrId},
};

#[derive(Debug, Error)]
pub enum IRError {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Unknown value")]
    UnknownValue,
}

#[derive(Debug)]
pub struct IR {
    nodes: Vec<Node>,
    str_tab: Interner,
    sections: Sections,
    instructions: Instructions,
    last_section_id: SectionId,
}

impl IR {
    pub fn new(cap: usize) -> Self {
        Self {
            nodes: Vec::new(),
            str_tab: Interner::with_capacity(cap),
            sections: Sections::new(),
            instructions: Instructions::new(),
            last_section_id: SectionId::default(),
        }
    }
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn alloc_str(&mut self, name: &str) -> StrId {
        self.str_tab.intern(name)
    }

    pub fn add_instruction(&mut self, ins: Instruction) {
        let id = self.instructions.add(ins);
        self.nodes.push(Node::Instruction(id));
    }

    pub fn add_section(&mut self, name: &str, ty: SectionType) {
        let str_id = self.str_tab.intern(name);
        let id = self.sections.switch(str_id, ty);
        self.nodes.push(Node::Section(id));
        self.last_section_id = id;
    }

    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn str_tab(&self) -> &Interner {
        &self.str_tab
    }

    pub fn str_tab_mut(&mut self) -> &mut Interner {
        &mut self.str_tab
    }

    pub fn sections_mut(&mut self) -> &mut Sections {
        &mut self.sections
    }

    pub fn active_section(&self) -> SectionId {
        self.last_section_id
    }

    pub(crate) fn print_ins(&self) {
        println!("Instructions: {:?}", self.instructions);
    }

    pub(crate) fn print_nodes(&self) {
        println!("Nodes: {:?}", self.nodes);
    }

    pub(crate) fn print_sections(&self) {
        println!("Sect: {:?}", self.sections);
    }
}

/// Represents data parsed into a section, using spans for strings.
#[derive(Debug)]
pub enum Node {
    String(Box<str>),
    Section(SectionId),
    Instruction(InstructionId),
    Label(StrId),
    // Global(StrId),
    // Word(u32),
    // Byte(u8),
    // Half(u16),
    // Align(u32), // New for .align, .p2align, .balign
    // Skip(u32),
}

use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct Sections {
    map: FxHashMap<SectionTag, SectionId>,
    vec: Vec<Section>,
}

impl Sections {
    pub fn new() -> Sections {
        Sections {
            map: FxHashMap::default(),
            vec: Vec::with_capacity(SectionType::count()),
        }
    }

    pub fn switch(&mut self, str_id: StrId, ty: SectionType) -> SectionId {
        let tag = SectionTag::new(str_id, ty);
        if let Some(id) = self.map.get(&tag) {
            return *id;
        }
        let id = self.generate_id();
        self.insert(tag, id);
        id
    }

    pub fn insert(&mut self, tag: SectionTag, id: SectionId) {
        let section = Section::new(tag.clone(), id);
        self.map.insert(tag, id);
        self.vec.push(section);
    }

    pub fn get(&self, id: SectionId) -> &Section {
        &self.vec[usize::from(id)]
    }

    ///Generate the next `id`
    pub fn generate_id(&self) -> SectionId {
        SectionId::new(self.map.len() as u8)
    }
}

#[derive(Debug)]
pub struct InstructionId(u32);

#[derive(Debug)]
pub struct Instructions {
    vec: Vec<Instruction>,
}

impl Instructions {
    fn new() -> Instructions {
        Instructions { vec: Vec::new() }
    }

    pub fn add(&mut self, value: Instruction) -> InstructionId {
        self.vec.push(value);
        InstructionId((self.vec.len() - 1) as u32)
    }
}
