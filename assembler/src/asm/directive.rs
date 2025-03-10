use std::fmt::Display;

use shared::{EnumCount, EnumVariants};

#[derive(EnumVariants, Debug, Clone, Copy, PartialEq, EnumCount)]
pub enum DirectiveType {
    // Symbol dir
    /// Create symbol/add symbol to the symbol table as local symbol\
    ///`.set name, expression`
    Set,
    ///`.equ name, expression` Create symbol/add symbol to the symbol table as local symbol. Same as `Set`
    Equ,
    ///`.globl name` Turn local symbols into global ones
    Globl,

    // Data dir
    ///`.byte expression [, expression]*` -> Emit one or more 8-bit comma separated words
    Byte,
    ///`.half expression [, expression]*` -> Emit one or more 16-bit comma separated words
    Half,
    ///`.word expression [, expression]*` -> Emit one or more 32-bit comma separated words\
    /// Ex: \
    /// `.word 10` -> assemble a 32-bit value (10) and add it to the active section
    Word,
    ///`.string string` -> Emit NULL terminated string
    String,
    ///`.asciz string` -> Emit NULL terminated string (alias for .string)
    Asciz,
    ///`.ascii string` -> Emit string without NULL character
    Ascii,
    // Incbin, //.incbin filename -> emit the included file as a binary sequence of octets
    // Zero, //.zero integer -> zero bytes

    // Alignment dir
    ///`.align N` -> To keep the memory align 4 bytes. Use this to aligned the location counter `Skip`.\
    /// It will checks if the location counter is a multiple of `2N`, if it is, it has no effect on the program, otherwise, it advances the location counter to the next value that is a multiple of `2N`
    Align,
    ///`b,[pad_val=0]` -> byte align
    Balign,
    /// `p2,[pad_val=0],max` -> align to power of 2
    P2align,

    // Section dir
    Section, //[{.text,.data,.rodata,.bss} or user defined name] -> e.g. .section .data -> turn the .data section in memory into the active section, hence,.. all the information processed by the assembler after this directive is added to the .data section
    Text,
    Data,
    Rodata,
    Bss,
    // CustomSection,

    // Allocation dir
    /// `symbol, size, align` -> emit common object to .bss section (local)
    Comm,
    /// `symbol, size, align` -> emit common object to .bss section (global)
    LComm,

    // Misc dir
    /// `.skip N` -> advances the location counter by N units and can be used to allocate space for variables on the .bss section which actually can't be added any data to it by the program.
    Skip,
    // Option, // {rvc,norvc,pic,nopic,push,pop} -> RISC-V options
    // File,   // filename -> emit filename FILE LOCAL symbol table
    // Ident,  //string,
    // Size,   //symbol, symbol
    // Type,   //symbol, @function
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".{}", Self::variants()[*self as usize])
    }
}

// pub enum DirectiveFolder {
//     Symbol(DirectiveType),
//     Data(DirectiveType),
//     Alignment(DirectiveType),
//     Section(DirectiveType),
//     Allocation(DirectiveType),
//     Misc(DirectiveType),
// }

// impl DirectiveFolder {
//     pub const fn unwrap(&self) -> i32 {
//         self.
//     }
// }

// impl From<DirectiveType> for DirectiveFolder {
//     fn from(value: DirectiveType) -> Self {
//         use DirectiveType::*;
//         match value {
//             Set | Equ | Globl => DirectiveFolder::Symbol(value),
//             Byte | Half | Word | Dword | String | Asciz | Ascii | Incbin | Zero => {
//                 DirectiveFolder::Data(value)
//             }
//             Align | Balign | P2align => DirectiveFolder::Alignment(value),
//             Section | Text | Data | Rodata | Bss | CustomSection => DirectiveFolder::Section(value),
//             Comm | LComm => DirectiveFolder::Allocation(value),
//             Skip | Option | File | Ident | Size | Type => DirectiveFolder::Misc(value),
//         }
//     }
// }
