// use std::collections::HashMap;

// use nom::{bytes::tag, error::ParseError, IResult, Parser};

// // enum ParserError {
// //     UnknownSection
// // }

// // impl ParseError<&[u8]> for ParserError {
// //     fn from_error_kind(input: &[u8], kind: nom::error::ErrorKind) -> Self {
// //         todo!()
// //     }

// //     fn append(input: &[u8], kind: nom::error::ErrorKind, other: Self) -> Self {
// //         todo!()
// //     }
// // }

// pub enum Section {
//     Text,
//     Data
// }

// // impl<'a> Parser<&'a [u8]> for Section {
// //     type Output = bool;

// //     type Error = ParserError;

// //     fn process<OM: nom::OutputMode>(
// //         &mut self,
// //         input: &'a [u8],
// //       ) -> nom::PResult<OM, &'a [u8], Self::Output, Self::Error> {
// //         todo!()
// //     }
// // }

// pub struct Sections([Section; 4]);

// // #[wasm_bindgen]
// pub struct Assembler {
//     instructions: HashMap,
// }

// #[wasm_bindgen]
// impl Assembler {
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Self {
//         let mut instructions = HashMap::new();
//         instructions.insert("add".to_string(), 0b0110011); // Example opcode for ADD

//         Assembler {
//             instructions,
//         }
//     }

//     pub fn assemble(&self, asm_code: &str) -> Vec {
//         let mut binary = Vec::new();
//         for line in asm_code.lines() {
//             if let Some(binary_instr) = self.parse_line(line) {
//                 binary.extend_from_slice(&binary_instr.to_le_bytes());
//             } else {
//                 // Handle errors or unsupported instructions
//                 console_log(&format!("Error parsing line: {}", line));
//             }
//         }
//         binary
//     }

//     // /// Return value will be: let (remaining_input, output) = do_nothing_parser("my_input")?;
//     // pub fn do_nothing_parser(&self, input: &str) -> IResult<&str, &str> {
//     //     Ok((input, ""))
//     // }

//     pub fn parse_line(&self, line: &str) -> Option {
//         let f = [0u8; 12];
//         let ff = b"addi";
//         let ggg = &ff[..];
//         let tg = tag(&ff[..]);
//         let mut fgf = Section::Data;
//         let ff = fgf.parse(ggg);

//         let tokens: Vec<&str> = line.split_whitespace().collect();
//         if tokens.len() < 4 || tokens[0] != "add" { // For simplicity, only handling 'add' instruction
//             return None;
//         }

//         let opcode = self.instructions.get(tokens[0])?;
//         let rd = self.parse_register(tokens[1])?;
//         let rs1 = self.parse_register(tokens[2])?;
//         let rs2 = self.parse_register(tokens[3])?;

//         Some(self.encode_r_type(*opcode, rd, rs1, rs2))
//     }

//     fn parse_register(&self, reg: &str) -> Option {
//         if let Some(num) = reg.strip_prefix('x') {
//             num.parse::().ok()
//         } else {
//             None
//         }
//     }

//     fn encode_r_type(&self, opcode: u32, rd: u8, rs1: u8, rs2: u8) -> u32 {
//         // R-type instruction format for RV32I
//         // funct7 | rs2 | rs1 | funct3 | rd | opcode
//         // Here we're simplifying; in reality, you'd need to handle funct7 and funct3 for different R-type instructions
//         let funct7 = 0; // For basic ADD
//         let funct3 = 0b000; // For ADD

//         (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode
//     }
// }

// // Helper function to log to browser console
// fn console_log(s: &str) {
//     web_sys::console::log_1(&s.into());
// }
