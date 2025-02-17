pub trait Directive {}

pub enum SymbolDir {
    Set, //.set name, expression -> for creating symbol/add symbol to the symbol table as local symbol
    Equ, //.equ name, expression -> for creating symbol/add symbol to the symbol table as local symbol
    Globl, //.globl name -> turn local symbols into global ones
}

pub enum DataDir {
    Byte,   //.byte expression [, expression]* -> Emit one or more 8-bit comma separated words
    Half,   //.half expression [, expression]* -> Emit one or more 16-bit comma separated words
    Word, //.word expression [, expression]* -> Emit one or more 32-bit comma separated words e.g. .word 10 -> assemble a 32-bit value (10) and add it to the active section
    Dword, //.dword expression [, expression]* -> Emit one or more 64-bit comma separated words
    String, //.string string -> Emit NULL terminated string
    Asciz, //.asciz string -> Emit NULL terminated string (alias for .string)
    Ascii, //.ascii string -> Emit string without NULL character
    Incbin, //.incbin filename -> emit the included file as a binary sequence of octets
    Zero, //.zero integer -> zero bytes
}

pub enum AlignmentDir {
    Align, //.align N -> To keep the memory align 4 bytes. This is the proper way of ensuring the location counter is aligned
    // and not by doing .skip N where N is the number to make it aligned. It will checks if the location counter is a multiple of 2N,
    // if it is, it has no effect on the program, otherwise, it advances the location counter to the next value that is a multiple of 2N
    Balign,  //b,[pad_val=0] -> byte align
    P2align, //p2,[pad_val=0],max -> align to power of 2
}

pub enum SectionDir {
    Section, //[{.text,.data,.rodata,.bss}] -> e.g. .section .data -> turn the .data section in memory into the active section, hence, all the information processed
    // by the assembler after this directive is added to the .data section
    Comm,   //symbol_name,size,align -> emit common object to .bss section
    Common, //symbol_name,size,align -> emit common object to .bss section
}

pub enum MiscDir {
    Skip, // .skip N -> advances the location counter by N units and can be used to allocate space for variables on the .bss section
    // which actually can't be added any data to it by the program.
    Option, // {rvc,norvc,pic,nopic,push,pop} -> RISC-V options
    Macro,  //name arg1 [, argn] -> begin macro
    Endm,   // end macro
    File,   // filename -> emit filename FILE LOCAL symbol table
    Ident,  //string,
    Size,   //symbol, symbol
    Type,   //symbol, @function
}

impl Directive for SymbolDir {}
impl Directive for DataDir {}
impl Directive for AlignmentDir {}
impl Directive for SectionDir {}
impl Directive for MiscDir {}
