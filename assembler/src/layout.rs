use crate::ir::IR;

pub struct Layout;

impl Layout {
    pub fn new(ir: IR) -> Layout {
        Layout
    }

    // pub fn resolve(&mut self) -> Result<(), ParsingError> {
    // self.ir.push(Node::Label(Symbol::new(
    //     str_id,
    //     Visibility::Local,
    //     None,
    //     SymbolType::Label,
    // )));
    //     while let Some(token) = self.eat() {
    //         self.walk(*token)?;
    //     }

    //     Ok(())
    // }
}

// struct AssembledSection {
//     bytes: Vec<u8>,              // Encoded machine code or data
//     offsets: Vec<(u32, usize)>,  // Offset and index into original nodes
//     symbols: Vec<Symbol>,        // Labels defined in this section
//     relocations: Vec<(u32, String)>, // Relocation info for unresolved symbols
// }
