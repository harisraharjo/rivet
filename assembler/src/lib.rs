mod asm;
mod ast;
mod instruction;
mod lexer;
mod parser;

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

use isa::instruction::Instruction;

pub enum Section {
    Text,
    Data,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Mnemonic(String),
    Register(String),
    Immediate(i32),
    LabelDef(String),
    Directive(String),
    Comma,
    LParen,
    RParen,
    // ... others
}

// #[token_parse]
// #[derive(PartialEq, Clone, Copy, Debug)]
// pub enum Token<'a> {
//     #[token(parse_fn = "")]
//     Mnemonic,
//     #[token(regex = "%[0-9]+")]
//     PercentInt(&'a str),
//     #[token(parse_fn = "compiler_tools::util::parse_str::<'\\''>")]
//     String(&'a str),
//     #[token(regex = "[0-9]+")]
//     Int(i32),
//     #[token(regex = "awa[a-z]+")]
//     AwaIdent(&'a str),
//     #[token(regex = "[a-z][a-zA-Z0-9_]*")]
//     Ident(&'a str),
//     // #[token(regex = "//[^\n]*")]
//     // Comment(&'a str),
//     #[token(regex = "#[^\n]*")]
//     Comment(&'a str),
//     // #[token(regex = "/\\*.*\\*/")]
//     // CommentBlock(&'a str),
//     #[token(regex = "[ \n]+")]
//     Whitespace,
//     #[token(illegal)]
//     Illegal(char),
// }

pub struct Parser<'a> {
    input: &'a [u8],
    instruction: Instruction,
}

// // #[wasm_bindgen]
pub struct Assembler {
    // instructions: HashMap,
    lc: usize, //Location Counter
}

// #[wasm_bindgen]
impl Assembler {
    // #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // let fgf: [&[u8]] = [b"aaa", b"ggg"];
        // let mut instructions = HashMap::new();
        // instructions.insert("add".to_string(), 0b0110011); // Example opcode for ADD

        Assembler { lc: todo!() }
    }

    // pub fn assemble(&self, asm_code: &str) -> Vec {
    //     let mut binary = Vec::new();
    //     for buffer in asm_code.lines() {
    //         if let Some(binary_instr) = self.parse_line(buffer) {
    //             binary.extend_from_slice(&binary_instr.to_le_bytes());
    //         } else {
    //             // Handle errors or unsupported instructions
    //             console_log(&format!("Error parsing buffer: {}", buffer));
    //         }
    //     }
    //     binary
    // }

    // /// Return value will be: let (remaining_input, output) = do_nothing_parser("my_input")?;
    pub fn fun(&self, input: &str) -> [&str; 2] {
        ["a", "v"]
    }

    pub fn parse_line(&self, mut buffer: &[u8]) {
        println!("Parsing...");
        println!("Len: {}", buffer.len());
        println!("{:?}", buffer);
        // let a = buffer[0..3];
        // a.trim_ascii()
        //  let mut bytes = self;
        // // Note: A pattern matching based approach (instead of indexing) allows
        // // making the function const.
        // while let [first, rest @ ..] = bytes {
        //     if first.is_ascii_whitespace() {
        //         bytes = rest;
        //     } else {
        //         break;
        //     }
        // }
        // bytes

        // let ff = buffer.split_mut(|a| b"{a}" == b"\n");

        let buffer = buffer.trim_ascii();
        println!("Len: {}", buffer.len());
        println!("{:?}", buffer);
        // enum Pattern {
        //     WithImm,
        // }
        //mnemonic field1, field2, fieldImm(field3) | fieldImm
        // menmonic -> field1 -> , -> field2 | imm -> , -> imm(field3) | imm
        // buffer.trim
        // let ga= nom::bytes::complete::tag(buffer);
        // let gaf= alt([b"a"]);

        //  let mut parts = buffer.split(|&b| b == b' ' || b == b'\t');
        // for chunk in gfg {

        // }

        // let gg = Utf8Pattern::StringPattern(buffer);
        // let ff = "aaa".as_utf8_pattern().unwrap();

        // let f = [0u8; 12];
        // let ff = b"addi";
        // let ggg = &ff[..];
        // let tg = tag(&ff[..]);
        // let mut fgf = Section::Data;
        // let ff = fgf.parse(ggg);

        // let buffer = "111";

        // let tokens: Vec<&str> = buffer.split_whitespace().collect();
        // if tokens.len() < 4 || tokens[0] != "add" {
        //     // For simplicity, only handling 'add' instruction
        //     return None;
        // }

        // let opcode = self.instructions.get(tokens[0])?;
        // let rd = self.parse_register(tokens[1])?;
        // let rs1 = self.parse_register(tokens[2])?;
        // let rs2 = self.parse_register(tokens[3])?;

        // Some(self.encode_r_type(*opcode, rd, rs1, rs2))
    }

    // fn parse_register(&self, reg: &str) -> Option {
    //     if let Some(num) = reg.strip_prefix('x') {
    //         num.parse::().ok()
    //     } else {
    //         None
    //     }
    // }

    // fn encode_r_type(&self, opcode: u32, rd: u8, rs1: u8, rs2: u8) -> u32 {
    //     // R-type instruction format for RV32I
    //     // funct7 | rs2 | rs1 | funct3 | rd | opcode
    //     // Here we're simplifying; in reality, you'd need to handle funct7 and funct3 for different R-type instructions
    //     let funct7 = 0; // For basic ADD
    //     let funct3 = 0b000; // For ADD

    //     (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode
    // }
}

// // Helper function to log to browser console
// fn console_log(s: &str) {
//     web_sys::console::log_1(&s.into());
// }

#[cfg(test)]
mod test_super {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn t_parse() {
        let asmblr = Assembler::new();

        match File::open("test.asm") {
            Ok(mut file) => {
                let mut buffer = Vec::new();

                // read the whole file
                file.read_to_end(&mut buffer).unwrap();
                asmblr.parse_line(&buffer);
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
