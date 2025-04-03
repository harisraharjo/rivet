use crate::interner::StrId;

use super::directive::DirectiveType;

// pub type SourceSpan = Range<usize>;
/// Represents a section in the assembler, mapping to an ELF section header.
#[derive(Debug)]
pub struct Section {
    /// Id in the Sections.
    id: SectionId,
    /// Type of the section (e.g., Progbits, Nobits).
    content_type: ContentType,
    /// Flags indicating section properties (e.g., ALLOC, EXECINSTR).
    flags: Flag,
    /// Alignment requirement (power of 2, e.g., 4 for .text).
    alignment: Alignment,
    /// Section type. `.text, .data, etc`
    tag: SectionTag,
    // /// Progbits or Nobits
    // content: T,
}

impl Section {
    pub fn new(
        tag: SectionTag,
        id: SectionId,
        // content: T,
    ) -> Self {
        let ty = tag.ty;
        let content_type = match ty {
            SectionType::Text => ContentType::Progbits,
            SectionType::Data => ContentType::Progbits,
            SectionType::Rodata => ContentType::Progbits,
            SectionType::Bss => ContentType::Nobits,
            SectionType::CustomSection => ContentType::Progbits,
        };

        let flags = match content_type {
            ContentType::Progbits => Flag::ALLOC,
            ContentType::Nobits => Flag::ALLOC | Flag::WRITE,
        };

        Self {
            id,
            flags,
            alignment: Alignment::new(ty),
            tag,
            content_type, // content,
        }
    }

    pub fn tag(&self) -> SectionTag {
        self.tag.clone()
    }
}
#[derive(Debug, Default, Clone, Copy, EnumCount, PartialEq, Eq, Hash)]
pub enum SectionType {
    #[default]
    Text,
    Data,
    Rodata,
    Bss,
    CustomSection,
    // None,
}

impl SectionType {
    pub const fn count() -> usize {
        Self::VARIANT_COUNT
    }
}

impl From<DirectiveType> for SectionType {
    fn from(value: DirectiveType) -> Self {
        match value {
            DirectiveType::Data => Self::Data,
            DirectiveType::Rodata => Self::Rodata,
            DirectiveType::Bss => Self::Bss,
            DirectiveType::CustomSection => Self::CustomSection,
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

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct SectionTag {
    str_id: StrId,
    ty: SectionType,
}

impl SectionTag {
    pub fn new(str_id: StrId, ty: SectionType) -> SectionTag {
        SectionTag { str_id, ty }
    }

    pub fn strid(&self) -> StrId {
        self.str_id
    }

    pub fn ty(&self) -> SectionType {
        self.ty
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct SectionId(u8);

impl SectionId {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn set(&mut self, value: u8) {
        self.0 = value;
    }
}

impl From<SectionId> for usize {
    fn from(value: SectionId) -> Self {
        value.0 as usize
    }
}

impl From<SectionId> for u8 {
    fn from(value: SectionId) -> Self {
        value.0
    }
}
