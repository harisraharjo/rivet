use super::directive::DirectiveType;
use std::ops::Range;

pub struct Sections {
    ///  \[Text, Data, Rodata, Bss]
    //1 to exclude bss
    base: [Section<Progbits>; ActiveSection::VARIANT_COUNT - 1],
    bss: Section<Nobits>,
    //TODO: support user defined section
    // custom: Vec<Section<Progbits>>,
    active: ActiveSection,
}

impl Sections {
    pub fn switch(&mut self, ty: DirectiveType, source_span: SourceSpan) {
        self.active = match ty {
            DirectiveType::Data => {
                let active_section = ActiveSection::Data;
                self.base[active_section as usize]
                    .name
                    .get_or_insert(source_span);
                active_section
            }
            DirectiveType::Rodata => {
                let active_section = ActiveSection::Rodata;
                self.base[active_section as usize]
                    .name
                    .get_or_insert(source_span);
                active_section
            }
            DirectiveType::Bss => {
                self.bss.name.get_or_insert(source_span);
                ActiveSection::Bss
            }
            _ => {
                let active_section = ActiveSection::Text;
                self.base[active_section as usize]
                    .name
                    .get_or_insert(source_span);
                active_section
            } // DirectiveType::CustomSection => todo!(),
        };
    }

    pub fn insert(&self, data: u8) -> i32 {
        if let Some(s) = self.base.get(self.active as usize) {
            return 1;
        };

        return 1;
        // self.bss
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
    name: Option<SourceSpan>,
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
        name: Option<SourceSpan>,
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
}

impl Default for Section<Progbits> {
    fn default() -> Self {
        Self {
            name: Default::default(),
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
            name: Default::default(),
            alignment: Alignment::new(DirectiveType::Bss),
            flags: Flag::ALLOC | Flag::WRITE,
            ty: DirectiveType::Bss,
            content: Nobits::default(),
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

// pub struct Elementary<T> where T {
//     ty: DirectiveType,
//     data: T
// }

/// Represents data parsed into a section, using spans for strings.
#[derive(Debug)]
enum Element {
    Word(u32),                 // .word 0x12345678
    Byte(u8),                  // .byte 0xFF
    Half(u16),                 // .half 0x1234
    String(Range<usize>),      // .asciz "hello" (span into source)
    Instruction(Range<usize>), // e.g., "lw x5, 0(x6)" (span into source)
    Align(u32),                // New for .align, .p2align, .balign
    Skip(u32),                 // For .skip size
}

// impl Element {
//     pub fn new() -> Element {
//         // match value {
//         //     DirectiveType::Byte => Self::Byte(Default::default()),
//         //     DirectiveType::Half => Self::Half(Default::default()),
//         //     DirectiveType::Word => Self::Word(Default::default()),
//         //     DirectiveType::String => Self::String(Default::default()),
//         //     DirectiveType::Asciz => Self::String(Default::default()),
//         //     DirectiveType::Ascii => Self::String(Default::default()),
//         //     DirectiveType::Align => Self::Align(Default::default()),
//         //     DirectiveType::Balign => Self::Align(Default::default()),
//         //     DirectiveType::P2align => Self::Align(Default::default()),
//         //     DirectiveType::Skip => Self::Skip(Default::default()),

//         // }
//         Element {}
//     }
// }

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
