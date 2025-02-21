// mod ast;
// mod token;

// use std::fmt::Debug;
// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum ParseError {
//     #[error("Undefined symbol: {0}")]
//     UndefinedSymbol(String),
//     #[error("Duplicate label: {0}")]
//     DuplicateLabel(String),
// }

// use std::collections::HashMap;

// use crate::token::{IdentifierType, Token, Tokens};

// // Token type from lexer (assume this is provided)
// // #[derive(Debug, Clone, PartialEq)]
// // enum Token {
// //     Mnemonic(String),
// //     Register(u8),
// //     Immediate(i32),
// //     Label(String),
// //     Directive(String),
// //     Symbol(String),
// //     Comma,
// //     Newline,
// //     EOF,
// // }

// // impl Token {
// //     fn to_string(&self) -> String {
// //         match self {
// //             Token::Mnemonic(s) => s.clone(),
// //             Token::Register(r) => format!("x{}", r),
// //             Token::Immediate(i) => i.to_string(),
// //             Token::Label(l) => l.clone(),
// //             Token::Directive(d) => d.clone(),
// //             Token::Symbol(s) => s.clone(),
// //             Token::Comma => ",".to_string(),
// //             Token::Eol => "\n".to_string(),
// //             Token::EOF => "".to_string(),
// //         }
// //     }
// // }

// // Flat AST representation
// #[derive(Debug)]
// struct Ast {
//     nodes: Vec<AstNode>,
//     symbols: HashMap<String, SymbolInfo>,
// }

// #[derive(Debug)]
// enum AstNode {
//     Instruction {
//         mnemonic: String,
//         operands: Vec<Operand>,
//     },
//     LabelDefinition {
//         name: String,
//         position: usize,
//     },
//     Directive {
//         name: String,
//         args: Vec<String>,
//     },
//     Data {
//         value: DataValue,
//     },
// }

// #[derive(Debug)]
// enum Operand {
//     Register(u8),
//     Immediate(i32),
//     LabelRef(String),
// }

// #[derive(Debug)]
// enum DataValue {
//     Word(u32),
//     // Add more data types as needed
// }

// #[derive(Debug)]
// struct SymbolInfo {
//     position: usize,
//     is_global: bool,
// }

// // Parser with grammar checking
// pub struct Parser<'a> {
//     tokens: Tokens,
//     current: usize,
//     source: &'a [u8],
// }

// impl<'a> Parser<'a> {
//     pub fn new(source: &'a [u8], tokens: Tokens) -> Self {
//         Parser {
//             tokens,
//             current: 0,
//             source,
//         }
//     }

//     pub fn new_source<'source: 'a>(&mut self, input: &'source [u8]) {
//         self.source = input;
//     }

//     fn peek(&self) -> Option<&Token> {
//         self.tokens.get(self.current + 1)
//     }

//     fn fetch(&self) -> Token {
//         self.tokens.get_unchecked(self.current)
//     }

//     fn data(&self) -> &[u8] {
//         let span = self.tokens.span(self.current).to_owned();
//         // safety: safe because the span is guaranteed to be inside the bounds
//         unsafe { self.source.get_unchecked(span) }
//     }

//     fn advance(&mut self) -> Token {
//         // let token = self.peek().unwrap().clone();
//         self.current += 1;
//         self.fetch()
//         // token
//     }

//     pub fn parse(&mut self) -> Result<Ast, String> {
//         let mut ast = Ast {
//             nodes: Vec::with_capacity(1024),
//             symbols: HashMap::new(),
//         };

//         while self.current <= self.tokens.len() {
//             self.parse_line(&mut ast)?;
//         }

//         Ok(ast)
//     }

//     fn parse_line(&mut self, ast: &mut Ast) -> Result<(), String> {
//         match self.fetch() {
//             Token::Directive => {
//                 // self.advance(); // Consume directive
//                 let name = self.data();
//                 let mut args = Vec::new();

//                 while let Some(&token) = self.peek() {
//                     // Token::Eol Token::Eof
//                     match self.advance() {
//                         Token::Symbol(s) => args.push(s),
//                         Token::Immediate(i) => args.push(i.to_string()),
//                         Token::Comma => continue,
//                         t => {
//                             return Err(format!("Invalid argument for directive {}: {:?}", name, t))
//                         }
//                     }
//                 }
//                 self.expect(Token::Eol, "Expected newline after directive")?;

//                 match name.as_str() {
//                     ".globl" => {
//                         if args.len() != 1 {
//                             return Err(".globl expects exactly one symbol".to_string());
//                         }
//                         ast.symbols.insert(
//                             args[0].clone(),
//                             SymbolInfo {
//                                 position: ast.nodes.len(),
//                                 is_global: true,
//                             },
//                         );
//                     }
//                     ".lcomm" => {
//                         if args.len() != 2 {
//                             return Err(".lcomm expects symbol and size".to_string());
//                         }
//                         args[1]
//                             .parse::<usize>()
//                             .map_err(|_| "Invalid size in .lcomm".to_string())?;
//                         ast.nodes.push(AstNode::Directive { name, args });
//                     }
//                     _ => ast.nodes.push(AstNode::Directive { name, args }),
//                 }
//             }
//             Token::Label => {
//                 let name = self.data();
//                 // self.advance(); // Consume label
//                 self.expect(Token::Eol, "Expected newline after label")?;
//                 let position = ast.nodes.len();
//                 ast.symbols.insert(
//                     name.clone(),
//                     SymbolInfo {
//                         position,
//                         is_global: false,
//                     },
//                 );
//                 ast.nodes.push(AstNode::LabelDefinition { name, position });
//             }
//             Token::Identifier(IdentifierType::Mnemonic) => {
//                 let mnemonic = self.data();
//                 // self.advance(); // Consume mnemonic
//                 let operands = self.parse_operands(&mnemonic)?;
//                 self.check_operand_count(&mnemonic, &operands)?;
//                 ast.nodes.push(AstNode::Instruction { mnemonic, operands });
//                 self.expect(Token::Eol, "Expected newline after instruction")?;
//             }
//             Token::Eol => {
//                 // self.advance(); // Skip empty lines
//             }
//             t @ _ => return Err(format!("Unexpected token: {:?}", t)),
//         }
//         Ok(())
//     }

//     fn parse_operands(&mut self, mnemonic: &str) -> Result<Vec<Operand>, String> {
//         let mut operands = Vec::new();

//         while self.peek() != Token::Eol && self.peek() != Token::EOF {
//             match self.advance() {
//                 Token::Register(r) => operands.push(Operand::Register(r)),
//                 Token::Immediate(i) => operands.push(Operand::Immediate(i)),
//                 Token::Symbol(s) => operands.push(Operand::LabelRef(s)),
//                 Token::Comma => {
//                     if operands.is_empty() {
//                         return Err("Unexpected comma before first operand".to_string());
//                     }
//                     continue;
//                 }
//                 t => return Err(format!("Invalid operand for {}: {:?}", mnemonic, t)),
//             }

//             // Expect a comma after each operand except the last one
//             if self.peek() != Token::Eol && self.peek() != Token::EOF {
//                 self.expect(Token::Comma, "Expected comma between operands")?;
//             }
//         }

//         Ok(operands)
//     }

//     fn check_operand_count(&self, mnemonic: &str, operands: &[Operand]) -> Result<(), String> {
//         match mnemonic {
//             "add" | "sub" => {
//                 if operands.len() != 3 {
//                     return Err(format!(
//                         "{} expects 3 operands, found {}",
//                         mnemonic,
//                         operands.len()
//                     ));
//                 }
//                 if !matches!(operands[0], Operand::Register(_))
//                     || !matches!(operands[1], Operand::Register(_))
//                     || !matches!(operands[2], Operand::Register(_))
//                 {
//                     return Err(format!("{} expects register operands", mnemonic));
//                 }
//             }
//             "addi" => {
//                 if operands.len() != 3 {
//                     return Err(format!("addi expects 3 operands, found {}", operands.len()));
//                 }
//                 if !matches!(operands[0], Operand::Register(_))
//                     || !matches!(operands[1], Operand::Register(_))
//                     || !matches!(operands[2], Operand::Immediate(_))
//                 {
//                     return Err("addi expects rd, rs1, immediate".to_string());
//                 }
//             }
//             // Add more instruction checks here
//             _ => {} // Unknown mnemonics can be handled later in codegen
//         }
//         Ok(())
//     }

//     fn expect(&mut self, expected: Token, msg: &str) -> Result<(), String> {
//         if *self.peek() == expected {
//             self.advance();
//             Ok(())
//         } else {
//             Err(format!("{}: found {:?}", msg, self.peek()))
//         }
//     }
// }

// #[cfg(test)]
// mod test_super {
//     // use super::*;

//     // #[test]
//     // fn t() -> Result<(), Box<dyn std::error::Error>> {
//     // let tokens = vec![
//     //     Token::Directive(".text".to_string()),
//     //     Token::Eol,
//     //     Token::Mnemonic("add".to_string()),
//     //     Token::Register(1),
//     //     Token::Comma,
//     //     Token::Register(2),
//     //     Token::Comma,
//     //     Token::Register(3),
//     //     Token::Eol,
//     //     Token::Label("loop".to_string()),
//     //     Token::Eol,
//     //     Token::Mnemonic("addi".to_string()),
//     //     Token::Register(4),
//     //     Token::Comma,
//     //     Token::Register(0),
//     //     Token::Comma,
//     //     Token::Immediate(10),
//     //     Token::Eol,
//     //     // Invalid example (missing comma)
//     //     Token::Mnemonic("add".to_string()),
//     //     Token::Register(1),
//     //     Token::Register(2),
//     //     Token::Eol,
//     //     Token::EOF,
//     // ];

//     // let mut parser = Parser::new(tokens);
//     // match parser.parse() {
//     //     Ok(ast) => println!("AST: {:?}", ast),
//     //     Err(e) => eprintln!("Parse error: {}", e),
//     // }
//     // Ok(())
//     // }
// }
