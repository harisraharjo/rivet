use crate::instruction::Instruction;

use super::directive::DirectiveType;
use std::ops::Range;

pub struct Sections {
    // names: Vec<SourceSpan>,
    ///  \[Text, Data, Rodata, Bss]
    base: [Section<Progbits>; ActiveSection::progbits_len()],
    // base_span: [Range<usize>; ActiveSection::progbits_len()],
    bss: Section<Nobits>,
    //TODO: support user defined section
    // custom: Vec<Section<Progbits>>,
    active: ActiveSection,
}

impl Sections {
    pub fn switch(&mut self, ty: DirectiveType, source_span: &SourceSpan) {
        self.active = match ty {
            DirectiveType::Data => {
                let active_section = ActiveSection::Data;
                self.base[active_section as usize].insert_name(source_span);
                active_section
            }
            DirectiveType::Rodata => {
                let active_section = ActiveSection::Rodata;
                self.base[active_section as usize].insert_name(source_span);
                active_section
            }
            DirectiveType::Bss => {
                self.bss.insert_name(source_span);
                ActiveSection::Bss
            }
            _ => {
                let active_section = ActiveSection::Text;
                self.base[active_section as usize].insert_name(source_span);
                active_section
            } // DirectiveType::CustomSection => todo!(),
        };
    }

    pub fn insert(&mut self, element: Element) {
        if let Some(s) = self.base.get_mut(self.active as usize) {
            s.insert(element);
            return;
        };

        self.bss.insert(element);
    }
}

impl Default for Sections {
    fn default() -> Self {
        let mut text = Section::<Progbits>::default();
        text.flags.insert(Flag::EXECINSTR);
        let mut data = Section::<Progbits>::default();
        data.flags.insert(Flag::WRITE);
        Self {
            base: [text, data, Default::default()],
            bss: Default::default(),
            active: ActiveSection::Text,
        }
        // let flags = match ty {
        //     Text => (Flag::ALLOC | Flag::EXECINSTR),

        //     Data => Flag::ALLOC | Flag::WRITE,
        //     Bss => Flag::ALLOC | Flag::WRITE,
        //     Rodata => Flag::ALLOC,
        //     _ => Flag::ALLOC,
        // };
    }
}

/// Range in the source `&[u8]` where the section name resides.
pub type SourceSpan = Range<usize>;
/// Represents a section in the assembler, mapping to an ELF section header.
#[derive(Debug)]
pub struct Section<T>
where
    T: ContentType,
{
    /// Range in the source &[u8] where the section name resides.
    name: SourceSpan,
    // /// Type of the section (e.g., Progbits, Nobits).
    // attr: SectionAttribute,
    /// Flags indicating section properties (e.g., ALLOC, EXECINSTR).
    flags: Flag,
    /// Alignment requirement (power of 2, e.g., 4 for .text).
    alignment: Alignment,
    // /// Content or size, depending on section type.
    ty: DirectiveType,
    /// Progbits or Nobits
    content: T,
}

impl<T> Section<T>
where
    T: ContentType,
{
    pub const fn new(
        ty: DirectiveType,
        name: SourceSpan,
        flags: Flag,
        // alignment: Alignment,
        content: T,
    ) -> Self {
        Self {
            name,
            flags,
            alignment: Alignment::new(ty),
            ty,
            content,
        }
    }

    fn insert_name(&mut self, span: &SourceSpan) {
        if self.name.end != 0 {
            self.name.start = span.start;
            self.name.end = span.end;
        };
    }
}

impl Default for Section<Progbits> {
    fn default() -> Self {
        Self {
            name: 0..0,
            alignment: Alignment::new(DirectiveType::Text),
            flags: Flag::ALLOC,
            ty: DirectiveType::Text,
            content: Progbits::default(),
        }
    }
}

impl Default for Section<Nobits> {
    fn default() -> Self {
        Self {
            name: 0..0,
            alignment: Alignment::new(DirectiveType::Bss),
            flags: Flag::ALLOC | Flag::WRITE,
            ty: DirectiveType::Bss,
            content: Nobits::default(),
        }
    }
}

impl Section<Progbits> {
    pub fn insert(&mut self, element: Element) {
        self.content.buffer.push(element);
    }
}

impl Section<Nobits> {
    pub fn insert(&mut self, element: Element) {
        self.content.0 = match element {
            Element::Word(d) => d,
            Element::Byte(d) => d as u32,
            Element::Half(d) => d as u32,
            Element::Skip(d) => d,
            Element::Align(d) => d,
            _ => self.content.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, EnumCount)]
pub enum ActiveSection {
    #[default]
    Text,
    Data,
    Rodata,
    Bss,
    // CustomSection(usize),
}

impl ActiveSection {
    const fn progbits_len() -> usize {
        Self::VARIANT_COUNT - 1
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
    pub const fn new(ty: DirectiveType) -> Alignment {
        Alignment {
            value: match ty {
                DirectiveType::Text => 4,
                _ => 1,
            },
        }
    }
}

/// Represents an expression parsed from lexemes.
#[derive(Debug)]
enum Expr {
    Literal(i32),                     // Resolved numeric literal, e.g., 42
    Symbol(Range<usize>),             // Unresolved symbol, e.g., "foo" at 5..8
    BinaryOp(ExprSide, Op, ExprSide), // e.g., foo + 4
}

#[derive(Debug)]
enum ExprSide {
    Symbol,
    Literal,
}

/// Supported operators in expressions.
#[derive(Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// Represents data parsed into a section, using spans for strings.
#[derive(Debug)]
pub enum Element {
    Word(u32),                // .word 0x12345678
    Byte(u8),                 // .byte 0xFF
    Half(u16),                // .half 0x1234
    String(Range<usize>),     // .asciz "hello" (span into source)
    Instruction(Instruction), // e.g., "lw x5, 0(x6)" (span into source)
    Align(u32),               // New for .align, .p2align, .balign
    Skip(u32),                // For .skip size
}

pub trait ContentType {}
#[derive(Debug)]
pub struct Progbits {
    buffer: Vec<Element>,
}
impl ContentType for Progbits {}
// impl Progbits {
//     // fn new() -> Progbits {
//     //     Progbits
//     // }
// }

impl Default for Progbits {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(10),
        }
    }
}

#[derive(Debug, Default)]
pub struct Nobits(u32);
impl ContentType for Nobits {}

// /// Represents the type of a section defined by section control directives for RV32I.
// #[derive(Debug, PartialEq, Eq, Default)]
// enum SectionAttribute {
//     #[default]
//     /// Program data (e.g., .text, .data, .rodata, user-defined .section).
//     Progbits,
//     /// Uninitialized data (e.g., .bss).
//     Nobits,
//     /// String table (e.g., .shstrtab, indirectly related).
//     Strtab,
// }
