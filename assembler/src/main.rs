fn main() {
    println!("Hello, world!");
}

// #[derive(Debug)]
// pub enum Instruction {
//     Add { rd: u8, rs1: u8, rs2: u8 },
//     Addi { rd: u8, rs1: u8, imm: i32 },
//     // ... all RV32I instructions
// }

// #[derive(Debug)]
// pub enum AstNode {
//     Instruction(Instruction),
//     LabelDef(String),
//     Directive(String, Vec<String>),
// }

// pub struct Parser {
//     tokens: Vec<Token>,
//     pos: usize,
// }

// impl Parser {
//     pub fn parse(&mut self) -> Result<Vec<AstNode>, AssemblerError> {
//         let mut ast = Vec::new();
//         while self.pos < self.tokens.len() {
//             match self.tokens[self.pos].typ {
//                 TokenType::LabelDef(ref name) => {
//                     ast.push(AstNode::LabelDef(name.clone()));
//                     self.pos += 1;
//                 }
//                 TokenType::Mnemonic(ref m) => {
//                     let instr = self.parse_instruction(m)?;
//                     ast.push(AstNode::Instruction(instr));
//                 }
//                 // ... handle directives
//             }
//         }
//         Ok(ast)
//     }

//     fn parse_instruction(&mut self, mnemonic: &str) -> Result<Instruction, AssemblerError> {
//         match mnemonic {
//             "add" => {
//                 let rd = self.parse_register()?;
//                 self.consume(TokenType::Comma)?;
//                 let rs1 = self.parse_register()?;
//                 self.consume(TokenType::Comma)?;
//                 let rs2 = self.parse_register()?;
//                 Ok(Instruction::Add { rd, rs1, rs2 })
//             }
//             "addi" => {
//                 let rd = self.parse_register()?;
//                 self.consume(TokenType::Comma)?;
//                 let rs1 = self.parse_register()?;
//                 self.consume(TokenType::Comma)?;
//                 let imm = self.parse_immediate()?;
//                 Ok(Instruction::Addi { rd, rs1, imm })
//             }
//             // ... other instructions
//         }
//     }
// }
