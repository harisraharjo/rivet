pub mod grammar;
mod token;

use grammar::{OperandRuleType, OperandTokenType};
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
    instruction::{OperandError, OperandType, Operands},
    lexer::{Lexeme, Lexemes, LexemesSlice},
    symbol_table::{Symbol, SymbolError, SymbolTable},
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
        format!(" found {v}")
    } else {
        Default::default()
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Syntax Error. expected LABEL|DIRECTIVE|MNEMONIC")]
    SyntaxError,
    #[error("Expected {expected}{}", on_invalid_grammar(.found))]
    UnexpectedToken {
        // #[source]
        // expected: RuleError,
        expected: Token,
        found: Option<String>,
    },
    #[error("Invalid line. Multiple {0}s encountered. Only 1 {0} is allowed")]
    InvalidLine(Single),
    #[error("{0} is still work in progress. Stay tuned!")]
    UnimplementedFeature(Todo),
    #[error("Duplicate label {0}")]
    DuplicateLabel(String),
    #[error(transparent)]
    ValueError(#[from] OperandError),
    #[error(" symbol {0}")]
    SymbolError(#[from] SymbolError),
    //     #[error("Undefined symbol: {0}")]
    //     UndefinedSymbol(String),
}

/// Return the `lexeme` if correct, otherwise returns error `UnexpectedToken`
macro_rules! expect_token {
    ($lexeme:expr, None) => {
        match $lexeme {
            Some(l) => Err(ParserError::UnexpectedToken {
                expected: Token::Eol,
                found: Some({ *l.token() }.to_string()),
            }),
            None => Ok(()),
        }
    };
    ($lexeme:expr, $x:expr) => {
        match $lexeme {
            Some(l) => match *l.token() {
                exp if exp == $x => Ok(l),
                t @ _ => Err(ParserError::UnexpectedToken {
                    expected: $x,
                    found: Some(t.to_string()),
                }),
            },
            None => Err(ParserError::UnexpectedToken {
                expected: $x,
                found: None,
            }),
        }
    };
    ($lexeme:expr, $pattern:pat, $ex:expr) => {
        match $lexeme {
            Some(l) => match *l.token() {
                $pattern => Ok(l),
                t @ _ => Err(ParserError::UnexpectedToken {
                    expected: $ex,
                    found: Some(t.to_string()),
                }),
            },
            None => Err(ParserError::UnexpectedToken {
                expected: $ex,
                found: None,
            }),
        }
    };
}

// macro_rules! foo {
//     ($($a:literal)|+) => {$($a)|+}
// }

// macro_rules! expect_tokens {
//     () => (
//         $crate::vec::Vec::new()
//     );
//     ($elem:expr; $n:expr) => (
//         $crate::vec::from_elem($elem, $n)
//     );
//     ($($x:expr),+ $(,)?) => (
//         <[_]>::into_vec(
//             // This rustc_box is not required, but it produces a dramatic improvement in compile
//             // time when constructing arrays with many elements.
//             #[rustc_box]
//             $crate::boxed::Box::new([$($x),+])
//         )
//     );
// }

// macro_rules! matches_flex {
//     ($token:expr, $custom_pat:pat => $custom_res:expr, $( $pat:pat => $res:expr ),* ) => {
//         let _: Result<(), ParserError> = match $token {
//                 $custom_pat => $custom_res,
//                 $( $pat => $res ),*
//             };
//     };
// }
// Parser with grammar checking
pub struct Parser<'a> {
    lexemes: Lexemes,
    index: usize,
    source: &'a [u8],
    sections: Sections,
    symtab: &'a mut SymbolTable<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes, symtab: &'a mut SymbolTable<'a>) -> Self {
        Parser {
            lexemes,
            index: 0,
            source,
            sections: Sections::default(),
            symtab,
        }
    }

    pub fn new_source<'source: 'a>(&mut self, input: &'source [u8]) {
        self.source = input;
    }

    #[inline(always)]
    fn next_index(&self) -> usize {
        self.index + 1
    }

    fn peek(&self) -> Option<Lexeme<'_>> {
        self.lexemes.get(self.next_index())
    }

    /// Peek the next (1 + `N`) token
    fn peek_n(&self, index: usize) -> Option<Lexeme<'_>> {
        self.lexemes.get(self.next_index() + index)
    }

    /// Peek until the next (1 + `N`) token
    fn peek_until(&self, index: usize) -> Option<LexemesSlice<'_>> {
        let next_idx = self.next_index();
        self.lexemes.slice(next_idx..self.next_index() + index)
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        // upper bound is at the index of Eol||Eof because we don't take them in
        //safety: unwrap is safe because guaranteed (Token::Eol || Token::Eof) is always present
        self.peek_until(self.nearest_break_idx().unwrap()).unwrap()
    }

    fn peek_token(&self) -> Option<&Token> {
        self.lexemes.get_token(self.next_index())
    }

    pub fn remainder_token(&self) -> &[Token] {
        // safety: Read until the end. guaranteed to be safe
        &self.lexemes.tokens()[self.next_index()..]
    }

    fn eat(&mut self) -> Option<Token> {
        let token = self.lexemes.tokens()[self.index];

        if token == Token::Eof {
            return None;
        }

        Some(token)
    }

    fn nearest_break_idx(&self) -> Option<usize> {
        self.remainder_token()
            .iter()
            .position(|t| *t == Token::Eol || *t == Token::Eof)
    }

    fn advance_line(&mut self) {
        self.advance_by(self.nearest_break_idx().unwrap());
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
    fn get_source(&self, span: Range<usize>) -> Option<&'a [u8]> {
        self.source.get(span)
    }

    #[inline(always)]
    fn get_source_unwrap(&self, span: Range<usize>) -> &'a [u8] {
        self.source.get(span).unwrap()
    }

    fn current_span(&self) -> &Range<usize> {
        self.get_span(self.index)
    }

    fn current_source(&self) -> &'a [u8] {
        // safety: safe because the span is guaranteed to be inside the bounds
        self.get_source_unwrap(self.current_span().to_owned())
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        while let Some(token) = self.eat() {
            self.walk(token)?;
        }

        Ok(())
    }

    fn walk(&mut self, token: Token) -> Result<(), ParserError> {
        println!("Parsing... {:?}", token);
        match token {
            Token::Directive(dir_type) => {
                use crate::asm::directive::DirectiveType::*;

                match dir_type {
                    Section | Text | Data | Rodata | Bss => {
                        self.sections
                            .switch(dir_type, self.current_span().to_owned());
                    }
                    Byte | Half | Word => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    String | Asciz | Ascii => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Align | Balign | P2align => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Set | Equ => {
                        //syntax analaysis
                        let mut line = self.peek_line();
                        let symbol =
                            expect_token!(line.next(), Token::Identifier(IdentifierType::Symbol))?;
                        expect_token!(line.next(), Token::Comma)?;
                        // line.
                        match line.next() {
                            Some(l) => match *l.token() {
                                Token::LiteralBinary
                                | Token::LiteralHex
                                | Token::LiteralDecimal => Ok(()),
                                t @ _ => Err(ParserError::UnexpectedToken {
                                    expected: Token::LiteralDecimal,
                                    found: Some(t.to_string()),
                                }),
                            },
                            None => Err(ParserError::UnexpectedToken {
                                expected: Token::LiteralDecimal,
                                found: None,
                            }),
                        }?;

                        let symbol_span = symbol.span().to_owned();
                        let (ty, offset) = {
                            let curr_sect = self.sections.current();
                            (curr_sect.ty(), curr_sect.offset())
                        };

                        self.symtab.insert(
                            ty,
                            Symbol::new(
                                self.get_source_unwrap(symbol_span),
                                Default::default(),
                                offset.into(),
                            ),
                        )?;

                        // return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Global => {
                        //syntax analaysis
                        let mut lexemes = self.peek_line();
                        let symbol = expect_token!(
                            lexemes.next(),
                            Token::Identifier(IdentifierType::Symbol)
                        )?;
                        expect_token!(lexemes.next(), None)?;
                        let span = symbol.span().to_owned();

                        // let span = expect_token!(self.peek(), Token::symbol())?
                        //     .span()
                        //     .to_owned();
                        // expect_token!(self.peek_n(1), Token::Eol)?;

                        self.symtab.declare_global(
                            self.sections.current().ty(),
                            self.get_source_unwrap(span),
                        )?;
                        self.advance();
                    }
                    Comm | LComm => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Skip => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                }
            }
            Token::Identifier(IdentifierType::Symbol) => {
                // TODO: correct symbol
                let curr_sect = self.sections.current();
                let offset = curr_sect.offset();

                self.symtab.insert(
                    curr_sect.ty(),
                    Symbol::new(self.current_source(), Default::default(), offset.into()),
                )?;
            }
            Token::Label => {
                // syntax analysis
                {
                    let mut lexemes = self.peek_line();

                    // find duplicate label because multiple labels can exists at the end of the line
                    if let Some(_) = lexemes.find_token(|token| *token == Token::Label) {
                        return Err(ParserError::InvalidLine(Single::Label));
                    }

                    if let Some(l) = lexemes.next() {
                        match l.token() {
                            Token::Identifier(IdentifierType::Mnemonic(_))
                            | Token::Directive(_) => {}
                            token @ _ => {
                                return Err(ParserError::UnexpectedToken {
                                    // TODO: better error reporting for multi variants. e.g dir|ins|break
                                    expected: Token::directive(),
                                    found: Some(token.to_string()),
                                });
                            }
                        };
                    };
                }

                let curr_sect = self.sections.current();
                let offset = curr_sect.offset();

                self.symtab.insert(
                    curr_sect.ty(),
                    Symbol::new(self.current_source(), Default::default(), offset.into()),
                )?;
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic)) => {
                let mut lexemes = self.peek_line();
                let mut rule = grammar::InstructionRule::new(mnemonic);
                let sequence = rule.generate_sequence();

                // Syntax analysis
                if let Some(mismatch) = sequence
                    .iter()
                    .zip(lexemes.by_ref())
                    .filter(|(ty, lex)| lex.token().to_owned() != **ty)
                    .next()
                {
                    let (_, lexeme) = mismatch;

                    return Err(ParserError::UnexpectedToken {
                        expected: *lexeme.token(),
                        found: std::str::from_utf8(
                            self.get_source_unwrap(lexeme.span().to_owned()),
                        )
                        .unwrap()
                        .to_owned()
                        .into(),
                    });
                };

                let seq_len = sequence.len();
                let rule_residue = seq_len.saturating_sub(lexemes.len());
                // check whether the input is: (too little, too much)
                match (rule_residue > 0, &lexemes.next()) {
                    (true, None) => {
                        let operand_token = sequence[seq_len - rule_residue];
                        return Err(ParserError::UnexpectedToken {
                            expected: operand_token.into(),
                            found: None,
                        });
                    }
                    (false, Some(lexeme)) => {
                        return Err(ParserError::UnexpectedToken {
                            expected: Token::Eol,
                            found: std::str::from_utf8(
                                self.get_source_unwrap(lexeme.span().to_owned()),
                            )
                            .unwrap()
                            .to_owned()
                            .into(),
                        });
                    }
                    //impossible
                    _ => {}
                };
                let rule_ty = rule.ty();
                drop(rule);

                // Value analysis
                lexemes.reset();
                let operand_types = lexemes
                    .step_by(OperandRuleType::noises_in_every())
                    .map(|lexeme| (lexeme, rule_ty, self.source).try_into())
                    .collect::<Result<Vec<OperandType>, OperandError>>()?;

                let mut current_section = self.sections.current();

                let mut operands = Operands::new();
                for (target, value) in operands.iter_mut().zip(operand_types) {
                    *target = value;
                    match target {
                        OperandType::Symbol(span) | OperandType::Label(span) => {
                            let sym_bytes = self.source.get(span.to_owned()).unwrap();
                            println!("Sym name: {:?}", unsafe {
                                std::str::from_utf8_unchecked(sym_bytes)
                            });

                            self.symtab.insert(
                                current_section.ty(),
                                Symbol::new(
                                    sym_bytes,
                                    Default::default(),
                                    current_section.offset().into(),
                                ),
                            )?;
                        }
                        _ => {}
                    }
                }

                let ins = crate::instruction::Instruction::new(mnemonic, operands);
                println!("Instruction IR: {:?}", ins);
                println!("Current offset: {:?}", current_section.offset());

                // let pseudo = PseudoInstruction
                current_section.insert(Element::Instruction(ins));
                current_section.increase_offset_by(4);
                self.advance_by(seq_len);
            }
            Token::Eol => {
                println!("=== eol ===");
            }
            Token::Comma => {
                //do nothing
            }
            t @ _ => {
                println!("Unknown Token: {:?}", t);
                return Err(ParserError::SyntaxError);
            }
        }

        self.advance();
        Ok(())
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
        match self {
            Todo::Dir(directive_type) => Display::fmt(directive_type, f),
            Todo::Symbol => write!(f, "{}", "symbol"),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn t_parser() {
        let lex = Lexer::new();

        let raw_source = r#"
        .section .text
        main:
            .global main
            addi x5, x6, my_symbol
            // my_symbol x11, x22, 11 //this is a wrong instruction pattern
            // eds0110xFF //valid symbol
            // sw x6, -2147483647(x4) //error too small imm value
            add x6, x0, x4
            lw x1, 10(x5)
            sw x1, 111(x5)
            lui x1, 0x1212
            // 0x1000MP # invalid literal bin
            // 99beto // invalid literal decimal
            // 0b11kl // invalid literal Binary
        "#;

        let source = raw_source.as_bytes();

        let mut symbol_table = SymbolTable::new();

        let lexemes = lex.tokenize(source).unwrap();
        println!("Sym table locals: {:?}", symbol_table.locals());

        let mut parser = Parser::new(source, lexemes, &mut symbol_table);
        assert!(match parser.parse() {
            Ok(_) => true,
            Err(e) => panic!("{e}"),
        })
    }
}
