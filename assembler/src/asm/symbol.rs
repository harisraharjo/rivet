//? Symbol: is an absoulte, means that the value is not changed during the linking process
//? Label: (is basically a symbol). may have its value (which is an address) changed during the relocation process.

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SymbolType {
    #[default]
    Label,
    Constant,
}

#[derive(Debug, Default)]
pub enum LabelType {
    #[default]
    Symbolic, // Stored as symbols in the symbol table and are often used to identify global variables and routines. e.g. `get_age:` `age:`
    Numeric, // Single decimal digit followed by a colon. e.g. `1:` used for local reference and are not included in the symbol table of executable
             //      files. Also, they can be redefined repeatedly in the same assembly program.
             //  References to numeric labels contain a suffix that indicates whether the reference is to a numeric label positioned
             //      before (‘b’ suffix) or after (‘f’ suffix) the reference
}
