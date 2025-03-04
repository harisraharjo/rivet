use std::{fmt::Display, ops::Range};

use isa::instruction::{InstructionType, Mnemonic};
use shared::EnumCount;
use thiserror::Error;

use crate::{
    asm::directive::DirectiveFolder,
    token::{IdentifierType, Token},
};

pub struct InstructionRule<'a> {
    ty: OperandRuleType,
    sequence: &'a [OperandTokenType],
}

impl<'a> InstructionRule<'a> {
    pub fn new(mnemonic: Mnemonic) -> InstructionRule<'a> {
        let ty = OperandRuleType::from(mnemonic);
        InstructionRule {
            ty,
            sequence: Self::generate_sequence(ty),
        }
    }

    pub fn get(&self, index: usize) -> OperandTokenType {
        self.sequence[index]
    }

    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &OperandTokenType> + ExactSizeIterator + use<'_> {
        self.sequence.iter()
    }

    fn generate_sequence(ty: OperandRuleType) -> &'a [OperandTokenType] {
        use OperandTokenType::*;
        match ty {
            OperandRuleType::R3 => [Register, Comma, Register, Comma, Register].as_slice(),
            OperandRuleType::R2I => [Register, Comma, Register, Comma, Immediate].as_slice(),
            OperandRuleType::R2L => [Register, Comma, Register, Comma, Label].as_slice(),
            OperandRuleType::RI => [Register, Comma, Immediate].as_slice(),
            OperandRuleType::RIR => {
                [Register, Comma, Immediate, ParenL, Register, ParenR].as_slice()
            }
            OperandRuleType::RL => [Register, Comma, Label].as_slice(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("`{0}`")]
    InvalidInstructionSequence(OperandTokenType),
    #[error("directive|instruction|break")]
    InvalidLabelSequence,
}

#[derive(EnumCount, Copy, Clone, Debug)]
pub enum OperandTokenType {
    Register,
    Comma,
    Label,
    Immediate,
    ParenL,
    ParenR,
    // Symbol,
    Eol,
}

impl Display for OperandTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OperandTokenType::*;
        write!(
            f,
            "{}",
            match self {
                Register => "register",
                Comma => "comma",
                Label => "label",
                Immediate => "decimal|hex|binary",
                ParenL => "(",
                ParenR => ")",
                Eol => "eol", //"\\n|\\r"
                              // Symbol => "symbol",
            }
        )
    }
}

impl PartialEq<OperandTokenType> for Token {
    fn eq(&self, other: &OperandTokenType) -> bool {
        use OperandTokenType::*;
        match (self, other) {
            (Token::Identifier(IdentifierType::Register(_)), Register) => true,
            (Token::Identifier(IdentifierType::Symbol), Immediate) => true,
            (Token::Label, Label) => true,
            (Token::LiteralDecimal, Immediate) => true,
            (Token::LiteralHex, Immediate) => true,
            (Token::LiteralBinary, Immediate) => true,
            (Token::ParenR, ParenR) => true,
            (Token::ParenL, ParenL) => true,
            (Token::Comma, Comma) => true,
            (Token::Eol, Eol) => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
/// The operands rule
pub enum OperandRuleType {
    ///Register, Register, Register
    R3,
    #[default]
    ///Register, Register, Immediate
    R2I,
    ///Register, Register, Label
    R2L,
    ///Register, Immediate(Register)
    RIR,
    ///Register, Immediate
    RI,
    ///Register, Label
    RL,
}

impl From<InstructionType> for OperandRuleType {
    fn from(value: InstructionType) -> Self {
        use InstructionType::*;
        match value {
            Arithmetic => Self::R3,
            IA => Self::R2I,
            IJ => Self::R2I,
            IL => Self::RIR,
            S => Self::RIR,
            B => Self::R2L,
            J => Self::RL,
            U => Self::RI,
        }
    }
}

impl From<Mnemonic> for OperandRuleType {
    fn from(value: Mnemonic) -> Self {
        use Mnemonic::*;
        match value {
            Add => Self::R3,
            Sub => Self::R3,
            Mul => Self::R3,
            And => Self::R3,
            Or => Self::R3,
            Xor => Self::R3,
            Shl => Self::R3,
            Shr => Self::R3,
            ShrA => Self::R3,
            AddI => Self::R2I,
            Lui => Self::RI,
            Lw => Self::RIR,
            Sw => Self::RIR,
            Syscall => Self::R3,
        }
    }
}

pub struct Sections {
    section: Vec<Section>,
    progbits: Vec<u8>,
    no_bits: Vec<u32>,
}

impl Default for Sections {
    fn default() -> Self {
        Self {
            section: Vec::new(),
            progbits: Default::default(),
            no_bits: Default::default(),
        }
    }
}

/// Range in the source `&[u8]` where the section name resides.
pub type SourceSpan = Range<usize>;
/// Represents a section in the assembler, mapping to an ELF section header.
#[derive(Debug)]
struct Section {
    /// Offset into .shstrtab where the section name resides.
    name: SourceSpan,
    /// Type of the section (e.g., Progbits, Nobits).
    section_type: SectionType,
    // /// Flags indicating section properties (e.g., ALLOC, EXECINSTR).
    // flags: SectionFlags,
    // /// Alignment requirement (power of 2, e.g., 4 for .text).
    // alignment: u32,
    /// span for the real content in `Sections`
    span: Range<usize>, // /// Content or size, depending on section type.
                        // content: ContentType,
}

impl Section {
    // pub fn new(ty: ,name: SourceSpan) -> Self {
    //     let (section_type, flags) = match name {
    //         ".text" => (SectionType::Progbits, SectionFlags::ALLOC | SectionFlags::EXECINSTR),
    //         ".data" => (SectionType::Progbits, SectionFlags::ALLOC | SectionFlags::WRITE),
    //         ".bss" => (SectionType::Nobits, SectionFlags::ALLOC | SectionFlags::WRITE),
    //         ".rodata" => (SectionType::Progbits, SectionFlags::ALLOC),
    //         _ => (SectionType::Progbits, SectionFlags::ALLOC),
    //     };

    //     Section {
    //         name,
    //         section_type,
    //         flags,
    //         alignment: if name == ".text" { 4 } else { 1 },
    //         content: match section_type {
    //             SectionType::Nobits => SectionContent::Nobits(0),
    //             _ => SectionContent::Progbits(Vec::new()),
    //         },
    //     }
    // }
}

/// Represents the type of a section in an ELF object file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SectionType {
    Progbits,
    /// Uninitialized data (e.g., .bss), occupies no space in the file.
    Nobits,
    /// Symbol table (e.g., .symtab).
    Symtab,
}

impl From<DirectiveFolder> for Section {
    fn from(value: DirectiveFolder) -> Self {
        use DirectiveFolder::*;
        match value {
            Section(ty) => todo!(),
            Symbol(ty) => todo!(),
            Data(ty) => todo!(),
            Alignment(ty) => todo!(),
            Allocation(ty) => todo!(),
            Misc(ty) => todo!(),
        }
    }
}
