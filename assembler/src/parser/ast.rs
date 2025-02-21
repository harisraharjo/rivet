// #[derive(Debug)]
// pub enum AstNode {
//     // Code
//     Instruction {
//         opcode: String,
//         operands: Vec<Operand>,
//         span: (usize, usize), // Source position for errors
//     },
//     LabelDef {
//         name: String,
//         span: (usize, usize),
//     },

//     // Data
//     DataDef {
//         label: Option<String>, // e.g., "my_array: .word ..."
//         directive: DataDirective,
//         span: (usize, usize),
//     },

//     // Symbols
//     SymbolDef {
//         name: String,
//         value: SymbolValue, // Constant, macro, or equ
//         span: (usize, usize),
//     },

//     // Sections
//     Section {
//         name: String, // ".text", ".data", etc.
//         body: Vec<AstNode>,
//         span: (usize, usize),
//     },
// }

use crate::lexer;

/// Deterministic State Machine
enum State {
    Initial,     // Default state, waiting for token start
    Label,       // Parsing a label definition (loop:)
    Directive,   // Parsing a directive (.text, .data)
    Mnemonic,    // Parsing a mnemonic (addi, lw, etc.)
    Register,    // Parsing a register (x0–x31)
    Immediate,   // Parsing an immediate value (-42)
    Punctuation, // Parsing punctuation (, ( ))
}

enum Grammar {
    Label,
    LabelInstruction,
    LabelDirective,
    Instruction,
    Directive,
}

// pub enum Ast {}

// impl Ast {
//     pub fn walk(&self, input: &[lexer::Token]) {
//         let len = input.len();
//         self.pos = 0;

//         while self.pos < len {
//             let c = input[self.pos];
//             match self.state {
//                 State::Initial => self.handle_initial(c, input),
//                 State::Mnemonic => self.handle_mnemonic(c),
//                 State::Register => self.handle_register(c),
//                 State::Label => self.handle_label_def(c),
//                 State::Directive => self.handle_directive(c),
//                 State::Immediate => self.handle_immediate(c),
//                 State::Punctuation => self.handle_punctuation(c),
//             }
//             self.pos += 1;
//         }

//         // Finalize any pending token
//         // self.finalize_buffer();
//     }
// }
