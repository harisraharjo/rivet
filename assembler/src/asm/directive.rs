use shared::{EnumCount, EnumVariants};

#[derive(EnumVariants, Debug, Clone, Copy, PartialEq, EnumCount)]
pub enum DirectiveType {
    // Symbol dir
    Set, //.set name, expression -> for creating symbol/add symbol to the symbol table as local symbol
    Equ, //.equ name, expression -> for creating symbol/add symbol to the symbol table as local symbol
    Globl, //.globl name -> turn local symbols into global ones

    // Data dir
    Byte,   //.byte expression [, expression]* -> Emit one or more 8-bit comma separated words
    Half,   //.half expression [, expression]* -> Emit one or more 16-bit comma separated words
    Word, //.word expression [, expression]* -> Emit one or more 32-bit comma separated words e.g. .word 10 -> assemble a 32-bit value (10) and add it to the active section
    Dword, //.dword expression [, expression]* -> Emit one or more 64-bit comma separated words
    String, //.string string -> Emit NULL terminated string
    Asciz, //.asciz string -> Emit NULL terminated string (alias for .string)
    Ascii, //.ascii string -> Emit string without NULL character
    Incbin, //.incbin filename -> emit the included file as a binary sequence of octets
    Zero, //.zero integer -> zero bytes

    // Alignment dir
    Align, //.align N -> To keep the memory align 4 bytes. This is the proper way of ensuring the location counter is aligned
    // and not by doing .skip N where N is the number to make it aligned. It will checks if the location counter is a multiple of 2N,
    // if it is, it has no effect on the program, otherwise, it advances the location counter to the next value that is a multiple of 2N
    Balign,  //b,[pad_val=0] -> byte align
    P2align, //p2,[pad_val=0],max -> align to power of 2

    // Section dir
    Section, //[{.text,.data,.rodata,.bss} or user defined name] -> e.g. .section .data -> turn the .data section in memory into the active section, hence,.. all the information processed by the assembler after this directive is added to the .data section
    Text,
    Data,
    Rodata,
    Bss,
    CustomSection,

    // Allocation dir
    Comm,  //symbol_name,size,align -> emit common object to .bss section
    LComm, //symbol_name,size,align -> emit common object to .bss section //for global

    // Misc dir
    Skip, // .skip N -> advances the location counter by N units and can be used to allocate space for variables on the .bss section which actually can't be added any data to it by the program.
    Option, // {rvc,norvc,pic,nopic,push,pop} -> RISC-V options
    File, // filename -> emit filename FILE LOCAL symbol table
    Ident, //string,
    Size, //symbol, symbol
    Type, //symbol, @function
}

// pub enum SectionType {
//     Custom,
//     Text,
//     Data,
//     Rodata,
//     Bss,
// }

pub enum DirectiveFolder {
    Symbol(DirectiveType),
    Data(DirectiveType),
    Alignment(DirectiveType),
    Section(DirectiveType),
    Allocation(DirectiveType),
    Misc(DirectiveType),
}

impl DirectiveFolder {
    pub fn rule(&self) -> i32 {
        1
    }
}

impl From<DirectiveType> for DirectiveFolder {
    fn from(value: DirectiveType) -> Self {
        use DirectiveType::*;
        match value {
            Set | Equ | Globl => DirectiveFolder::Symbol(value),
            Byte | Half | Word | Dword | String | Asciz | Ascii | Incbin | Zero => {
                DirectiveFolder::Data(value)
            }
            Align | Balign | P2align => DirectiveFolder::Alignment(value),
            Section | Text | Data | Rodata | Bss | CustomSection => DirectiveFolder::Section(value),
            Comm | LComm => DirectiveFolder::Allocation(value),
            Skip | Option | File | Ident | Size | Type => DirectiveFolder::Misc(value),
        }
    }
}
