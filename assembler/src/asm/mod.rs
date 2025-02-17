mod directive;
mod label;
mod memory;

pub struct SymbolTable; //maps each program symbol to its value. Labels converted to symbol automatically by assmbler

pub enum Symbol {
    Local,
    Global,
}

#[derive(Debug)]
enum Comment {
    Single, // #, //
    Multi,  // /**/
}

//? Note: The operands of assembly instructions may contain: Register, immediate, symbol name.

//? Note: Symbol names are defined by a sequence of alphanumeric characters and the underscore
//?-    character (_). However, the first character may not be a numeric character. Their value are also encoded into the machine instruction as
//?-    a sequence of bits.
