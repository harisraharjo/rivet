use crate::interner::StrId;

use super::directive::DirectiveType;

// pub type SourceSpan = Range<usize>;
/// Represents a section in the assembler, mapping to an ELF section header.
#[derive(Debug)]
pub struct Section {
    /// Id in the Interner.
    str_id: StrId,
    /// Type of the section (e.g., Progbits, Nobits).
    content_type: ContentType,
    /// Flags indicating section properties (e.g., ALLOC, EXECINSTR).
    flags: Flag,
    /// Alignment requirement (power of 2, e.g., 4 for .text).
    alignment: Alignment,
    /// Section type. `.text, .data, etc`
    ty: SectionType,
    // /// Progbits or Nobits
    // content: T,
}

impl Section {
    pub fn new(
        ty: SectionType,
        str_id: StrId,
        // content: T,
    ) -> Self {
        let content_type = match ty {
            SectionType::Text => ContentType::Progbits,
            SectionType::Data => ContentType::Progbits,
            SectionType::Rodata => ContentType::Progbits,
            SectionType::Bss => ContentType::Nobits,
        };

        let flags = match content_type {
            ContentType::Progbits => Flag::ALLOC,
            ContentType::Nobits => Flag::ALLOC | Flag::WRITE,
        };

        Self {
            str_id,
            flags,
            alignment: Alignment::new(ty),
            ty,
            content_type, // content,
        }
    }

    // fn insert_name(&mut self, span: SourceSpan) {
    //     if self.name.end != 0 {
    //         self.name.start = span.start;
    //         self.name.end = span.end;
    //     };
    // }
}

// impl Default for Section<Progbits> {
//     fn default() -> Self {
//         Self {
//             name: StrId,
//             alignment: Alignment::new(DirectiveType::Text),
//             flags: Flag::ALLOC,
//             ty: DirectiveType::Text,
//             content_type: ContentType::Progbits,
//             // content: Progbits::default(),
//         }
//     }
// }

// impl Default for Section<Nobits> {
//     fn default() -> Self {
//         Self {
//             name: StrId,
//             alignment: Alignment::new(DirectiveType::Bss),
//             flags: Flag::ALLOC | Flag::WRITE,
//             ty: DirectiveType::Bss,
//             content: Nobits::default(),
//         }
//     }
// }

// impl Section<Progbits> {
//     pub fn insert(&mut self, element: Node) {
//         self.content.buffer.push(element);
//     }
// }

// impl Section<Nobits> {
//     pub fn insert(&mut self, element: Node) {
//         self.content.0 = match element {
//             Node::Word(d) => d,
//             Node::Byte(d) => d as u32,
//             Node::Half(d) => d as u32,
//             Node::Skip(d) => d,
//             Node::Align(d) => d,
//             _ => self.content.0,
//         }
//     }
// }

#[derive(Debug, Default, Clone, Copy, EnumCount, PartialEq, Eq, Hash)]
pub enum SectionType {
    #[default]
    Text,
    Data,
    Rodata,
    Bss,
    // CustomSection(usize),
    // None,
}

impl SectionType {
    const fn progbits_len() -> usize {
        Self::VARIANT_COUNT - 1
    }

    // pub const fn name(&self) -> &str {
    //     match self {
    //         SectionType::Text => "text",
    //         SectionType::Data => "data",
    //         SectionType::Rodata => "rodata",
    //         SectionType::Bss => "bss",
    //     }
    // }
}

impl From<DirectiveType> for SectionType {
    fn from(value: DirectiveType) -> Self {
        match value {
            // DirectiveType::Text => Self::Text,
            DirectiveType::Data => Self::Data,
            DirectiveType::Rodata => Self::Rodata,
            DirectiveType::Bss => Self::Bss,
            _ => Self::Text,
        }
    }
}

use bitflags::bitflags;
use shared::EnumCount;

bitflags! {
    /// Defines section attributes as per ELF section header flags (sh_flags).
    #[derive(Debug)]
    pub struct Flag: u32 {
        /// Section is writable (e.g., .data, .bss).
        const WRITE = 0x1;
        /// Section is allocated in memory during execution (e.g., .text, .data).
        const ALLOC = 0x2;
        /// Section contains executable instructions (e.g., .text).
        const EXECINSTR = 0x4;
        // Additional flags like SHF_MERGE, SHF_STRINGS can be added as needed.
    }
}

#[derive(Debug)]
pub struct Alignment {
    value: u32,
}

impl Alignment {
    pub const fn new(ty: SectionType) -> Alignment {
        Alignment {
            value: match ty {
                SectionType::Text => 4,
                _ => 1,
            },
        }
    }
}

// pub trait ContentType {}
// #[derive(Debug)]
// pub struct Progbits {
//     buffer: Vec<Node>,
// }
// impl ContentType for Progbits {}
// // impl Progbits {
// //     // fn new() -> Progbits {
// //     //     Progbits
// //     // }
// // }

// impl Default for Progbits {
//     fn default() -> Self {
//         Self {
//             buffer: Vec::with_capacity(10),
//         }
//     }
// }

// #[derive(Debug, Default)]
// pub struct Nobits(u32);
// impl ContentType for Nobits {}

/// Represents the type of a section defined by section control directives for RV32I.
#[derive(Debug, PartialEq, Eq, Default)]
pub enum ContentType {
    #[default]
    /// Program data (e.g., .text, .data, .rodata, user-defined .section).
    Progbits,
    /// Uninitialized data (e.g., .bss).
    Nobits,
    // /// String table (e.g., .shstrtab, indirectly related).
    // Strtab,
}

// pub struct Sections {
//     ///  \[Text, Data, Rodata, Bss]
//     progbits: Vec<Section<Progbits>>,
//     bss: Section<Nobits>,
//     //TODO: support user defined section
//     // custom: Vec<Section<Progbits>>,
//     active: SectionType,
//     offsets: Vec<u32>,
// }

// impl Sections {
//     pub fn switch(&mut self, ty: DirectiveType, source_span: SourceSpan) {
//         self.active = match ty {
//             DirectiveType::Data => {
//                 let active_section = SectionType::Data;
//                 self.progbits[active_section as usize].insert_name(source_span);
//                 active_section
//             }
//             DirectiveType::Rodata => {
//                 let active_section = SectionType::Rodata;
//                 self.progbits[active_section as usize].insert_name(source_span);
//                 active_section
//             }
//             DirectiveType::Bss => {
//                 self.bss.insert_name(source_span);
//                 SectionType::Bss
//             }
//             _ => {
//                 let active_section = SectionType::Text;
//                 self.progbits[active_section as usize].insert_name(source_span);
//                 active_section
//             } // DirectiveType::CustomSection => todo!(),
//         };
//     }

//     pub fn current(&mut self) -> CurrentSection<'_> {
//         CurrentSection { sections: self }
//     }

//     // pub fn insert(&mut self, element: Node) {
//     //     if let Some(s) = self.progbits.get_mut(self.active as usize) {
//     //         s.insert(element);
//     //         return;
//     //     };

//     //     self.bss.insert(element);
//     // }

//     // pub fn active_section(&self) -> SectionType {
//     //     self.active
//     // }

//     // pub fn current_offset(&self) -> u32 {
//     //     self.offsets[self.active as usize]
//     // }

//     // pub fn increase_offset_by(&mut self, v: u32) {
//     //     self.offsets[self.active as usize] += v;
//     // }
// }

// impl Default for Sections {
//     fn default() -> Self {
//         let mut text = Section::<Progbits>::default();
//         text.flags.insert(Flag::EXECINSTR);
//         let mut data = Section::<Progbits>::default();
//         data.flags.insert(Flag::WRITE);

//         Self {
//             progbits: vec![text, data, Default::default()],
//             bss: Default::default(),
//             active: SectionType::Text,
//             offsets: vec![0; SectionType::VARIANT_COUNT],
//         }
//         // let flags = match ty {
//         //     Text => (Flag::ALLOC | Flag::EXECINSTR),

//         //     Data => Flag::ALLOC | Flag::WRITE,
//         //     Bss => Flag::ALLOC | Flag::WRITE,
//         //     Rodata => Flag::ALLOC,
//         //     _ => Flag::ALLOC,
//         // };
//     }
// }

// pub struct CurrentSection<'a> {
//     sections: &'a mut Sections,
// }

// impl CurrentSection<'_> {
//     pub fn insert(&mut self, element: Node) {
//         if let Some(s) = self
//             .sections
//             .progbits
//             .get_mut(self.sections.active as usize)
//         {
//             s.insert(element);
//             return;
//         };

//         self.sections.bss.insert(element);
//     }

//     pub fn offset(&self) -> u32 {
//         self.sections.offsets[self.sections.active as usize]
//     }

//     pub fn increase_offset_by(&mut self, v: u32) {
//         self.sections.offsets[self.sections.active as usize] += v;
//     }

//     pub fn ty(&self) -> SectionType {
//         self.sections.active
//     }
// }
