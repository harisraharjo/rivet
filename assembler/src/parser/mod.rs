pub mod grammar;

use grammar::{OperandRuleType, RuleToken};
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
    token::{self, IdentifierType, Token},
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
        expected: RuleToken,
        // expected: Token,
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

/// Return `Ok(lexeme)` if correct, otherwise returns error `Err(ParseError::UnexpectedToken)`
macro_rules! expect_token {
    ($lexeme:expr, None) => {
        match $lexeme {
            Some(l) => Err(ParserError::UnexpectedToken {
                expected: RuleToken::Eol,
                found: Some({ *l.token() }.to_string()),
            }),
            None => Ok(()),
        }
    };
    // TODO: modularize this macro to just extracting token and not lexeme
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

    /// Peek the next `N` token
    fn peek_n(&self, index: usize) -> Option<Lexeme<'_>> {
        self.lexemes.get(self.index + index)
    }

    /// Peek until the next (1 + `N`) token
    fn peek_until(&self, index: usize) -> Option<LexemesSlice<'_>> {
        let next_idx = self.next_index();
        self.lexemes.slice(next_idx..next_idx + index)
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

    // #[inline(always)]
    // fn get_source(&self, span: Range<usize>) -> Option<&'a [u8]> {
    //     self.source.get(span)
    // }

    // #[inline(always)]
    // fn get_source_unwrap(&self, span: Range<usize>) -> &'a [u8] {
    //     self.source.get(span).unwrap()
    // }

    fn current_span(&self) -> &Range<usize> {
        self.get_span(self.index)
    }

    fn current_source(&self) -> &'a [u8] {
        // safety: safe because the span is guaranteed to be inside the bounds
        self.source.get(self.current_span().to_owned()).unwrap()
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
                    Section => {
                        expect_token!(self.peek(), token::section_dir!(), RuleToken::SectionDir)?;
                    }
                    Text | Data | Rodata | Bss => {
                        //syntax analaysis
                        let mut lexemes = self.peek_line();
                        expect_token!(lexemes.next(), None)?;

                        self.sections
                            .switch(dir_type, self.current_span().to_owned());
                    }
                    Byte | Half | Word => {
                        //syntax analaysis
                        let mut lexemes = self.peek_line();
                        loop {
                            expect_token!(
                                lexemes.next(),
                                token::literal_integer!()
                                    | Token::Identifier(IdentifierType::Symbol),
                                RuleToken::SymbolOrLiteral
                            )?;

                            match lexemes.next() {
                                Some(l) => match *l.token() {
                                    token::operator!() => Ok(()),
                                    t @ _ => Err(ParserError::UnexpectedToken {
                                        expected: RuleToken::Eol,
                                        found: Some(t.to_string()),
                                    }),
                                },
                                None => {
                                    break;
                                }
                            }?;
                        }

                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Ascii => {
                        //syntax analaysis
                        let lexeme = expect_token!(
                            self.peek(),
                            Token::LiteralString,
                            RuleToken::LiteralString
                        )?;
                        expect_token!(self.peek_n(2), Token::Eol | Token::Eof, RuleToken::Eol)?;
                        println!("String value: {:?}", lexeme);

                        // TODO: finish the value
                    }
                    Align | Balign | P2align => {
                        let mut lexemes = self.peek_line();
                        loop {
                            expect_token!(
                                lexemes.next(),
                                token::literal_integer!()
                                    | Token::Identifier(IdentifierType::Symbol),
                                RuleToken::SymbolOrLiteral
                            )?;

                            match lexemes.next() {
                                Some(l) => match *l.token() {
                                    token::operator!() => Ok(()),
                                    t @ _ => Err(ParserError::UnexpectedToken {
                                        expected: RuleToken::Eol,
                                        found: Some(t.to_string()),
                                    }),
                                },
                                None => {
                                    break;
                                }
                            }?;
                        }

                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Set | Equ => {
                        //syntax analaysis
                        let mut lexemes = self.peek_line();
                        let symbol = expect_token!(
                            lexemes.next(),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;
                        expect_token!(lexemes.next(), Token::Comma, RuleToken::Comma)?;
                        //check expression
                        loop {
                            expect_token!(
                                lexemes.next(),
                                token::literal_integer!()
                                    | Token::Identifier(IdentifierType::Symbol),
                                RuleToken::SymbolOrLiteral
                            )?;

                            match lexemes.next() {
                                Some(l) => match *l.token() {
                                    token::operator!() => Ok(()),
                                    t @ _ => Err(ParserError::UnexpectedToken {
                                        expected: RuleToken::Eol,
                                        found: Some(t.to_string()),
                                    }),
                                },
                                None => {
                                    break;
                                }
                            }?;

                            // let name = String::from_utf8_lossy(&self.source[span.clone()]).to_string();
                            // symbols.symbols.iter().find(|s| s.0 == name).map(|s| s.1)
                        }

                        // TODO: syntax analysis is done. Now complete the expression and all the offsets for each token
                        let src = self.source.get(symbol.span().to_owned()).unwrap();
                        let curr_sect = self.sections.current();
                        self.symtab.insert(
                            curr_sect.ty(),
                            Symbol::new(src, Default::default(), None, curr_sect.offset()),
                        )?;
                    }
                    Global => {
                        //syntax analaysis
                        let symbol = expect_token!(
                            self.peek(),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;
                        expect_token!(self.peek_n(2), Token::Eol | Token::Eof, RuleToken::Eol)?;

                        let span = symbol.span().to_owned();
                        self.symtab.declare_global(
                            self.sections.current().ty(),
                            self.source.get(span).unwrap(),
                        )?;
                        self.advance();
                    }
                    Skip => {
                        let mut lexemes = self.peek_line();
                        loop {
                            expect_token!(
                                lexemes.next(),
                                token::literal_integer!()
                                    | Token::Identifier(IdentifierType::Symbol),
                                RuleToken::SymbolOrLiteral
                            )?;

                            match lexemes.next() {
                                Some(l) => match *l.token() {
                                    token::operator!() => Ok(()),
                                    t @ _ => Err(ParserError::UnexpectedToken {
                                        expected: RuleToken::Eol,
                                        found: Some(t.to_string()),
                                    }),
                                },
                                None => {
                                    break;
                                }
                            }?;
                        }
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Comm | LComm => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    String | Asciz => {
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                }
            }
            Token::Label => {
                // syntax analysis
                expect_token!(
                    self.peek(),
                    Token::Identifier(IdentifierType::Mnemonic(_))
                        | Token::Directive(_)
                        | Token::Eol
                        | Token::Eof,
                    RuleToken::InstructionOrDir
                )?;

                let src = self.current_source();
                let curr_sect = self.sections.current();
                self.symtab.insert(
                    curr_sect.ty(),
                    Symbol::new(src, Default::default(), None, curr_sect.offset()),
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
                    let (rule_token, lexeme) = mismatch;

                    return Err(ParserError::UnexpectedToken {
                        expected: *rule_token,
                        found: std::str::from_utf8(
                            self.source.get(lexeme.span().to_owned()).unwrap(),
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
                        let rule_token = sequence[seq_len - rule_residue];
                        return Err(ParserError::UnexpectedToken {
                            expected: rule_token,
                            found: None,
                        });
                    }
                    (false, Some(lexeme)) => {
                        return Err(ParserError::UnexpectedToken {
                            expected: RuleToken::Eol,
                            found: std::str::from_utf8(
                                self.source.get(lexeme.span().to_owned()).unwrap(),
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
                    match &value {
                        OperandType::Symbol(span) | OperandType::Label(span) => {
                            self.symtab.insert(
                                current_section.ty(),
                                Symbol::new(
                                    self.source.get(span.to_owned()).unwrap(),
                                    Default::default(),
                                    None,
                                    current_section.offset(),
                                ),
                            )?;
                        }
                        _ => {}
                    }

                    *target = value;
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
        .set symbol1, 1 + 1
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

    #[test]
    fn t_parser_global() {
        let lex = Lexer::new();

        let raw_source = r#"
        .section .text
        // .set symbol1, 1 + 1
        main:
            .global main"#;

        let source = raw_source.as_bytes();

        let mut symbol_table = SymbolTable::new();
        let lexemes = lex.tokenize(source).unwrap();

        let mut parser = Parser::new(source, lexemes, &mut symbol_table);
        assert!(match parser.parse() {
            Ok(_) => true,
            Err(e) => panic!("{e}"),
        })
    }
}
