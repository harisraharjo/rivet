pub mod directive;
pub mod section;
pub mod symbol;

//? Note: The operands of assembly instructions may contain: Register, immediate, symbol name.

//? Note: Symbol names are defined by a sequence of alphanumeric characters and the underscore
//?-    character (_). However, the first character may not be a numeric character. Their value are also encoded into the machine instruction as
//?-    a sequence of bits.
