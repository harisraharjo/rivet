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

// #[derive(Debug)]
// enum DataValue {
//     Word(u32),
//     // Add more current_source types as needed
// }

// #[derive(Debug)]
// struct SymbolInfo {
//     position: usize,
//     is_global: bool,
// }
#[derive(Debug, Error)]
enum Single {
    #[error("label")]
    Label,
    #[error("directive")]
    Directive,
}

// #[derive(Default, Debug, Clone, PartialEq, Error)]
// pub enum LexingError {
//     #[error("Invalid Integer at {0}")]
//     InvalidInteger(usize),
//     #[error("Unknown directive {0} at {1}")]
//     UnknownDirective(String, usize),
//     #[error("Invalid suffix {0} at {1}")]
//     InvalidSuffix(String, usize),
//     #[error("Invalid Ascii Character at {0}")]
//     NonAsciiCharacter(usize),
//     #[default]
//     #[error("Unknown Syntax")]
//     UnknownSyntax,
// }

fn on_invalid_grammar<'a>(found: &Option<String>) -> String {
    if let Some(v) = found {
        format!("found {v}")
    } else {
        Default::default()
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax Error. expected LABEL|DIRECTIVE|MNEMONIC")]
    SyntaxError,
    #[error("Invalid grammar. expected {expected} {}", on_invalid_grammar(.found))]
    InvalidGrammar {
        #[source]
        expected: RuleError,
        found: Option<String>,
    },
    #[error("Invalid line. Multiple {0}s encountered. Only 1 {0} is allowed")]
    InvalidLine(Single),
    #[error("Duplicate label {0}")]
    DuplicateLabel(String),
}

// struct Line

// Parser with grammar checking
pub struct Parser<'a> {
    lexemes: Lexemes,
    index: usize,
    source: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes) -> Self {
        Parser {
            lexemes,
            index: 0,
            source,
        }
    }

    pub fn new_source<'source: 'a>(&mut self, input: &'source [u8]) {
        self.source = input;
    }

    fn peek(&self) -> Option<&Token> {
        self.lexemes.get_token(self.index + 1)
    }

    pub fn remainder(&self) -> &[Token] {
        // safety: Read until the end. guaranteed to be safe
        &self.lexemes.tokens()[self.index..]
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        //safety: unwrap is safe because guaranteed (Token::Eol || Token::Eof) is always present
        let pos = self.nearest_break().unwrap();
        // upper bound is the index of Eol||Eof because we don't take them in
        self.lexemes.slice(self.index..self.index + pos)
    }

    fn eat(&mut self) -> Option<Token> {
        let token = self.lexemes.tokens()[self.index];

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

    #[inline(always)]
    fn advance_by(&mut self, n: usize) {
        self.index += n;
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
        self.get_span(self.index)
    }

    fn current_source(&self) -> &[u8] {
        let span = self.current_span().to_owned();
        // safety: safe because the span is guaranteed to be inside the bounds
        self.get_source(span)
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        while let Some(token) = self.eat() {
            self.walk(token)?;
        }

        Ok(())
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
                let lexemes = self.peek_line();
                if let Some(lex) = lexemes.find(|token| *token == Token::Label) {
                    return Err(ParserError::InvalidLine(Single::Label));
                }

                // Record the label in the symbol table with the current position
                // self.symbol_table.insert(label.clone(), self.position);
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
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic_type)) => {
                let rule = grammar::InstructionRule::new(mnemonic_type);
                let mut rule_iter = rule.iter();
                let mut lexemes = self.peek_line();
                let mut zipped = rule_iter.by_ref().zip(&mut lexemes);

                if let Some(mismatch) = zipped.find(|(ty, lex)| lex.token().to_owned() != **ty) {
                    let (ty, lexeme) = mismatch;

                    return Err(ParserError::InvalidGrammar {
                        expected: RuleError::InvalidInstruction(*ty),
                        found: Some(
                            String::from_utf8(self.get_source(lexeme.span().to_owned()).to_vec())
                                .unwrap(),
                        ),
                    });
                };

                let residue = rule.len().saturating_sub(lexemes.len());
                match (residue > 0, &lexemes.next()) {
                    (true, None) => Err(ParserError::InvalidGrammar {
                        expected: RuleError::InvalidInstruction(rule.get(rule.len() - residue)),
                        found: None,
                    }),
                    (false, Some(lex)) => Err(ParserError::InvalidGrammar {
                        expected: RuleError::InvalidInstruction(OperandTokenType::Eol),
                        found: Some(
                            String::from_utf8(self.get_source(lex.span().to_owned()).to_vec())
                                .unwrap(),
                        ),
                    }),
                    _ => {
                        self.advance_by(rule.len());
                        Ok(())
                    }
                }?;

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
mod test {
    use std::{fs::File, io::Read};

    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn t_parser() {
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
        assert!(match parser.parse() {
            Ok(_) => true,
            Err(e) => panic!("{e}"),
        })
    }
}
