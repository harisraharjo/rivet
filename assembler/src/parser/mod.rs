pub mod grammar;
mod token;

use grammar::{OperandTokenType, RuleError};
use std::{fmt::Debug, ops::Range};
use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum ParseError {
//     #[error("Undefined symbol: {0}")]
//     UndefinedSymbol(String),
//     #[error("Duplicate label: {0}")]
//     DuplicateLabel(String),
// }

use crate::token::{IdentifierType, Lexemes, LexemesSlice, Token};

#[derive(Debug)]
enum Operand {
    Register(u8),
    Immediate(i32),
    LabelRef(String),
}

#[derive(Debug)]
enum DataValue {
    Word(u32),
    // Add more current_source types as needed
}

#[derive(Debug)]
struct SymbolInfo {
    position: usize,
    is_global: bool,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Invalid grammar. expected {expected} found `{found}`")]
    InvalidGrammar {
        #[source]
        expected: RuleError,
        found: String,
    },
    #[error("Syntax Error. expected LABEL|DIRECTIVE|MNEMONIC")]
    SyntaxError,
}

// Parser with grammar checking
pub struct Parser<'a> {
    lexemes: Lexemes,
    current: usize,
    source: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes) -> Self {
        Parser {
            lexemes,
            current: 0,
            source,
        }
    }

    pub fn new_source<'source: 'a>(&mut self, input: &'source [u8]) {
        self.source = input;
    }

    fn peek(&self) -> Option<&Token> {
        self.lexemes.get_token(self.current + 1)
    }

    pub fn remainder(&self) -> &[Token] {
        // safety: Read until the end. guaranteed to be safe
        &self.lexemes.buffer()[self.current..]
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        //safety: unwrap safe because guaranteed (Token::Eol || Token::Eof) is always present
        let pos = self.nearest_break().unwrap();
        // upper bound is the index of Eol||Eof because we don't take them in
        self.lexemes.slice(self.current..self.current + pos)
    }

    fn eat(&mut self) -> Option<Token> {
        // Token::Eof
        let token = self.lexemes.buffer()[self.current];

        if token == Token::Eof {
            return None;
        }

        self.advance();
        Some(token)
    }

    fn nearest_break(&self) -> Option<usize> {
        self.remainder()
            .iter()
            .position(|t| *t == Token::Eol || *t == Token::Eof)
    }

    fn advance_line(&mut self) {
        let pos = self.nearest_break().unwrap();
        self.advance_by(pos);
    }

    fn advance_by(&mut self, n: usize) {
        self.current += n;
    }

    fn advance(&mut self) {
        self.advance_by(1);
    }

    #[inline(always)]
    fn get_span(&self, index: usize) -> &Range<usize> {
        self.lexemes.get_span(index)
    }

    #[inline(always)]
    fn get_source(&self, span: Range<usize>) -> &[u8] {
        &self.source[span]
    }

    fn current_span(&self) -> &Range<usize> {
        self.get_span(self.current)
    }

    fn current_source(&self) -> &[u8] {
        let span = self.current_span().to_owned();
        // safety: safe because the span is guaranteed to be inside the bounds
        self.get_source(span)
    }

    pub fn parse(&mut self) -> Result<bool, ParserError> {
        while let Some(token) = self.eat() {
            self.walk(token)?;
        }

        Ok(true)
    }

    fn walk(&mut self, token: Token) -> Result<(), ParserError> {
        println!("Parsing...");
        match token {
            Token::Directive(dir_type) => {
                println!("Directive : {:?}", dir_type);
                self.advance_line();
                // self.advance(); // Consume directive
                // let name = self.current_source();
                // let mut args = Vec::new();

                // while let Some(&token) = self.peek() {
                //     // Token::Eol Token::Eof
                //     match self.advance() {
                //         Token::Symbol(s) => args.push(s),
                //         Token::Immediate(i) => args.push(i.to_string()),
                //         Token::Comma => continue,
                //         t => {
                //             return Err(format!(
                //                 "Invalid argument for directive {}: {:?}",
                //                 name, t
                //             ));
                //         }
                //     }
                // }
                // self.expect(Token::Eol, "Expected newline after directive")?;

                // match name.as_str() {
                //     ".globl" => {
                //         if args.len() != 1 {
                //             return Err(".globl expects exactly one symbol".to_string());
                //         }
                //         ast.symbols.insert(
                //             args[0].clone(),
                //             SymbolInfo {
                //                 position: ast.nodes.len(),
                //                 is_global: true,
                //             },
                //         );
                //     }
                //     ".lcomm" => {
                //         if args.len() != 2 {
                //             return Err(".lcomm expects symbol and size".to_string());
                //         }
                //         args[1]
                //             .parse::<usize>()
                //             .map_err(|_| "Invalid size in .lcomm".to_string())?;
                //         // ast.nodes.push(AstNode::Directive { name, args });
                //     }
                //     _ => {
                //         // ast.nodes.push(AstNode::Directive { name, args })
                //     }
                // }
            }
            Token::Identifier(IdentifierType::Symbol) => {
                self.advance_line();
            }
            Token::Label => {
                println!("Label");
                self.advance_line();

                // let name = self.current_source();
                // self.advance(); // Consume label
                // self.expect(Token::Eol, "Expected newline after label")?;
                // let position = ast.nodes.len();
                // ast.symbols.insert(
                //     name.clone(),
                //     SymbolInfo {
                //         position,
                //         is_global: false,
                //     },
                // );
                // ast.nodes.push(AstNode::LabelDefinition { name, position });
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic_type)) => {
                let rule = grammar::InstructionRule::new(mnemonic_type);
                let mut lexemes = self.peek_line();
                let rule_iter = rule.iter();

                if let Some(mismatch) = rule_iter
                    .zip(&mut lexemes)
                    .find(|(ty, lex)| lex.token().to_owned() != **ty)
                {
                    let (ty, lexeme) = mismatch;

                    return Err(ParserError::InvalidGrammar {
                        expected: RuleError::InvalidInstruction(*ty).into(),
                        found: String::from_utf8(
                            self.get_source(lexeme.span().to_owned()).to_vec(),
                        )
                        .unwrap(),
                    });
                };

                if let Some(remainder) = &lexemes.next() {
                    let span = remainder.span();
                    println!("leftover: {:?}", remainder.token());
                    return Err(ParserError::InvalidGrammar {
                        expected: RuleError::InvalidInstruction(OperandTokenType::Eol).into(),
                        found: String::from_utf8(self.get_source(span.to_owned()).to_vec())
                            .unwrap(),
                    });
                }
                self.advance_by(rule.len());

                // ast.nodes.push(AstNode::Instruction { mnemonic, operands });
            }
            Token::Eol => {
                println!("=== eol ===");
                // self.advance(); // Skip empty lines
            }
            t @ _ => {
                println!("Unknown Token: {:?}", t);
                return Err(ParserError::SyntaxError);
            }
        }
        Ok(())
    }

    // fn expect(&mut self, expected: Token, err: ParserError) -> Result<(), ParserError> {
    //     let next = self.peek();
    //     println!("next: {:?}", next);
    //     if next == Some(&expected) {
    //         self.advance();
    //         Ok(())
    //     } else {
    //         Err(err)
    //     }
    // }
}

#[cfg(test)]
mod test_super {
    use std::{fs::File, io::Read};

    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn t_parser() -> Result<(), Box<dyn std::error::Error>> {
        // let lexemes = vec![
        //     Token::Directive(".text".to_string()),
        //     Token::Eol,
        //     Token::Mnemonic("add".to_string()),
        //     Token::Register(1),
        //     Token::Comma,
        //     Token::Register(2),
        //     Token::Comma,
        //     Token::Register(3),
        //     Token::Eol,
        //     Token::Label("loop".to_string()),
        //     Token::Eol,
        //     Token::Mnemonic("addi".to_string()),
        //     Token::Register(4),
        //     Token::Comma,
        //     Token::Register(0),
        //     Token::Comma,
        //     Token::Immediate(10),
        //     Token::Eol,
        //     // Invalid example (missing comma)
        //     Token::Mnemonic("add".to_string()),
        //     Token::Register(1),
        //     Token::Register(2),
        //     Token::Eol,
        //     Token::EOF,
        // ];

        let lex = Lexer::new();

        let source = match File::open("test.asm") {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                Ok(buffer)
            }
            Err(e) => {
                println!("File Error: {:?}", e);
                Err(e)
            }
        }
        .unwrap();

        // let mut symbol_table = SymbolTable::new();

        let lexemes = lex.tokenize(&source).unwrap();
        // for (&token, span) in lexemes.symbols() {
        //     symbol_table.insert(
        //         span.to_owned(),
        //         Symbol::new(Default::default(), None, token.try_into().unwrap()),
        //     );
        // }

        let mut parser = Parser::new(source.as_ref(), lexemes);
        assert!(parser.parse()?);
        // match parser.parse() {
        //     Ok(res) => println!("res: {:?}", res),
        //     Err(e) => eprintln!("Parse error: {}", e),
        // }
        Ok(())
    }
}
