pub mod grammar;
mod token;

use grammar::{OperandTokenType, RuleError};
use std::{
    fmt::{Debug, Display},
    ops::Range,
};
use thiserror::Error;

use crate::{
    asm::{
        directive::DirectiveType,
        section::{Element, Sections},
    },
    instruction::Operands,
    lexer::{Lexemes, LexemesSlice},
    token::{IdentifierType, Token},
};

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

fn on_invalid_grammar<'a>(found: &Option<String>) -> String {
    if let Some(v) = found {
        format!("found {v}")
    } else {
        Default::default()
    }
}

#[derive(Debug)]
enum Todo {
    // #[errortra]
    Dir(DirectiveType),
    // #[error("symbol")]
    Symbol,
}

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Todo::Dir(directive_type) => format!("{directive_type}"),
                Todo::Symbol => "symbol".to_string(),
            }
        )
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax Error. expected LABEL|DIRECTIVE|MNEMONIC")]
    SyntaxError,
    #[error("Unexpected Token. expected {expected} {}", on_invalid_grammar(.found))]
    UnexpectedToken {
        #[source]
        expected: RuleError,
        found: Option<String>,
    },
    #[error("Invalid line. Multiple {0}s encountered. Only 1 {0} is allowed")]
    InvalidLine(Single),
    #[error("{0} is still work in progress. Stay tuned!")]
    UnimplementedFeature(Todo),
    #[error("Duplicate label {0}")]
    DuplicateLabel(String),
    //     #[error("Undefined symbol: {0}")]
    //     UndefinedSymbol(String),
}
// Parser with grammar checking
pub struct Parser<'a> {
    lexemes: Lexemes,
    index: usize,
    source: &'a [u8],
    sections: Sections,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes) -> Self {
        Parser {
            lexemes,
            index: 0,
            source,
            sections: Sections::default(),
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
        // upper bound is at the index of Eol||Eof because we don't take them in
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
                use crate::asm::directive::DirectiveType::*;
                println!("Directive : {:?}", dir_type);

                // let line = self.peek_line();
                // if line.contains(&Token::Directive(dir_type)) {
                //     return Err(ParserError::InvalidLine(Single::Directive));
                // }

                match dir_type {
                    Section | Text | Data | Rodata | Bss => {
                        self.sections
                            .switch(dir_type, &self.current_span().to_owned());
                    }
                    Byte | Half | Word => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                        // let ff = dir_type as usize;
                        // self.sections.insert(data);
                        // todo!()
                    }
                    String | Asciz | Ascii => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                        // todo!()
                    }
                    Align | Balign | P2align => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Set | Equ | Globl => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Comm | LComm => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Skip => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                }

                // self.advance_line();
                // if let Some(Token::Symbol(name)) = self.peek() {
                //     let section_name = name.clone();
                //     self.advance(); // Consume section name
                //     self.current_section = section_name;

                //     // Check for optional flags
                //     self.section_flags = None;
                //     if self.peek() == Some(&Token::Comma) {
                //         self.advance(); // Consume comma
                //         if let Some(Token::Symbol(flags)) = self.peek() {
                //             let flags = flags.clone();
                //             self.advance(); // Consume flags
                //             // Validate flags (optional)
                //             if !flags.chars().all(|c| "axwrMSI".contains(c)) {
                //                 return Err(format!("Invalid section flags: {}", flags));
                //             }
                //             self.section_flags = Some(flags);
                //         } else {
                //             return Err("Expected section flags after comma".to_string());
                //         }
                //     }
                //     self.expect(Token::Newline, "Expected newline after .section")?;
                // } else {
                //     return Err("Expected section name after .section".to_string());
                // }

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
                return Err(ParserError::UnimplementedFeature(Todo::Symbol));
                // self.advance_line();
            }
            Token::Label => {
                let lexemes = self.peek_line();

                // find duplicate label first because a label can exists at the end of the line
                if let Some(lex) = lexemes.find(|token| *token == Token::Label) {
                    return Err(ParserError::InvalidLine(Single::Label));
                }

                if let Some(l) = lexemes.peek() {
                    match l.token() {
                        Token::Identifier(IdentifierType::Mnemonic(_)) => {}
                        Token::Directive(_) => {}
                        token @ _ => {
                            return Err(ParserError::UnexpectedToken {
                                expected: RuleError::InvalidLabelSequence,
                                found: Some(format!("{}", *token)),
                            });
                        }
                    }
                }

                // Record the label in the symbol table with the current position
                // self.symbol_table.insert(label.clone(), self.position);
                // self.advance();
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic)) => {
                let mut lexemes = self.peek_line();
                let mut rule = grammar::InstructionRule::new(mnemonic);
                let sequence = rule.sequence();

                if let Some(mismatch) = sequence
                    .iter()
                    .zip(lexemes.by_ref())
                    .filter(|(ty, lex)| lex.token().to_owned() != **ty)
                    .next()
                {
                    let (ty, lexeme) = mismatch;

                    return Err(ParserError::UnexpectedToken {
                        expected: RuleError::InvalidInstructionSequence(*ty),
                        found: Some(
                            String::from_utf8(self.get_source(lexeme.span().to_owned()).to_vec())
                                .unwrap(),
                        ),
                    });
                };

                let seq_len = sequence.len();
                let residue = seq_len.saturating_sub(lexemes.len());
                match (residue > 0, &lexemes.next()) {
                    (true, None) => {
                        return Err(ParserError::UnexpectedToken {
                            expected: RuleError::InvalidInstructionSequence(
                                sequence[seq_len - residue],
                            ),
                            found: None,
                        });
                    }
                    (false, Some(lex)) => {
                        return Err(ParserError::UnexpectedToken {
                            expected: RuleError::InvalidInstructionSequence(OperandTokenType::Eol),
                            found: Some(
                                String::from_utf8(self.get_source(lex.span().to_owned()).to_vec())
                                    .unwrap(),
                            ),
                        });
                    }
                    _ => {}
                };

                lexemes.reset();
                let operands = Operands::from((&mut lexemes, rule.ty(), self.source));
                println!("Operands: {:?}", operands);
                let ins = crate::instruction::Instruction::new(mnemonic, operands);

                // let pseudo = PseudoInstruction

                self.sections.insert(Element::Instruction(ins));
                self.advance_by(seq_len);
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
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn t_parser() {
        let lex = Lexer::new();

        let raw_source = r#"
        .section .data
        main:
            lw x5, -0b1000(x0)
            
            addi x5, x6, my_symbol
            # my_symbol x11, x22, 11 //this is a wrong instruction pattern
            # eds0110xFF //valid symbol
            # addi x6, 0x2000(x4)
            # lui x6, 0b111(x4)
            lw x1, 10(x5)
            sw x1, 111(x5)
            lui x1, 0x1212
            // 0x1000MP # invalid literal bin
            // 99beto // invalid literal decimal
            // 0b11kl // invalid literal Binary
        "#;

        let source = raw_source.to_string();

        // let mut symbol_table = SymbolTable::new();

        let lexemes = lex.tokenize(source.as_bytes()).unwrap();
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
