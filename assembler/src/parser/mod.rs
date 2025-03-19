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
    ($lexeme:expr, $expected:expr) => {
        match $lexeme {
            Some(l) => match *l.token() {
                exp if exp == $expected => Ok(l),
                t @ _ => Err(ParserError::UnexpectedToken {
                    expected: $expected,
                    found: Some(t.to_string()),
                }),
            },
            None => Err(ParserError::UnexpectedToken {
                expected: $expected,
                found: None,
            }),
        }
    };
}

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

    fn peek_token(&self) -> Option<&Token> {
        self.lexemes.get_token(self.next_index())
    }

    pub fn remainder(&self) -> &[Token] {
        // safety: Read until the end. guaranteed to be safe
        &self.lexemes.tokens()[self.next_index()..]
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        // upper bound is at the index of Eol||Eof because we don't take them in
        //safety: unwrap is safe because guaranteed (Token::Eol || Token::Eof) is always present
        let next_idx = self.next_index();
        self.lexemes
            .slice(next_idx..next_idx + self.nearest_break_idx().unwrap())
    }

    fn eat(&mut self) -> Option<Token> {
        let token = self.lexemes.tokens()[self.index];

        if token == Token::Eof {
            return None;
        }

        Some(token)
    }

    fn nearest_break_idx(&self) -> Option<usize> {
        self.remainder()
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
    fn get_source_unchecked(&self, span: Range<usize>) -> &'a [u8] {
        self.source.get(span).unwrap()
    }

    fn current_span(&self) -> &Range<usize> {
        self.get_span(self.index)
    }

    fn current_source(&self) -> &'a [u8] {
        // safety: safe because the span is guaranteed to be inside the bounds
        self.get_source_unchecked(self.current_span().to_owned())
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
                        return Err(ParserError::UnimplementedFeature(Todo::Dir(dir_type)));
                    }
                    Global => {
                        //syntax analaysis
                        let lexeme = expect_token!(self.peek(), Token::symbol())?;
                        let sym_bytes = self.get_source_unchecked(lexeme.span().to_owned());
                        println!("Global Sym name: {:?}", unsafe {
                            std::str::from_utf8_unchecked(sym_bytes)
                        });

                        self.symtab
                            .declare_global(self.sections.current().ty(), sym_bytes)?;

                        println!("SymTab Local: {:?}", unsafe {
                            std::str::from_utf8_unchecked({
                                self.symtab
                                    .locals()
                                    .get(&self.sections.current().ty())
                                    .unwrap()[0]
                                    .name()
                            })
                        });
                        println!("SymTab Global: {:?}", self.symtab.globals());

                        self.advance();

                        expect_token!(self.peek(), Token::Eol)?;
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
                return Err(ParserError::UnimplementedFeature(Todo::Symbol));
            }
            Token::Label => {
                // syntax analysis
                {
                    let lexemes = self.peek_line();

                    // find duplicate label because multiple labels can exists at the end of the line
                    if let Some(lex) = lexemes.find(|token| *token == Token::Label) {
                        return Err(ParserError::InvalidLine(Single::Label));
                    }

                    if let Some(l) = lexemes.peek() {
                        match l.token() {
                            Token::Identifier(IdentifierType::Mnemonic(_))
                            | Token::Directive(_) => {}
                            token @ _ => {
                                return Err(ParserError::UnexpectedToken {
                                    // TODO: better error reporting for multi variants. e.g dir|ins|break
                                    // expected: RuleError::InvalidLabelSequence,
                                    expected: Token::directive(),
                                    found: Some(token.to_string()),
                                });
                            }
                        }
                    }
                }

                let curr_sect = self.sections.current();
                let offset = curr_sect.offset();

                self.symtab.insert(
                    curr_sect.ty(),
                    Symbol::new(self.current_source(), Default::default(), offset.into()),
                )?;

                // curr_sect.increase_offset_by(v)
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic)) => {
                let mut lexemes = self.peek_line();
                let mut rule = grammar::InstructionRule::new(mnemonic);
                let sequence = rule.sequence();

                // Syntax analysis
                if let Some(mismatch) = sequence
                    .iter()
                    .zip(lexemes.by_ref())
                    .filter(|(ty, lex)| lex.token().to_owned() != **ty)
                    .next()
                {
                    let (&ty, lexeme) = mismatch;

                    return Err(ParserError::UnexpectedToken {
                        expected: ty.into(),
                        found: Some(
                            String::from_utf8(
                                self.get_source_unchecked(lexeme.span().to_owned()).to_vec(),
                            )
                            .unwrap(),
                        ),
                    });
                };

                let seq_len = sequence.len();
                let residue = seq_len.saturating_sub(lexemes.len());
                match (residue > 0, &lexemes.next()) {
                    (true, None) => {
                        let operand_token = sequence[seq_len - residue];
                        return Err(ParserError::UnexpectedToken {
                            expected: operand_token.into(),
                            found: None,
                        });
                    }
                    (false, Some(lex)) => {
                        return Err(ParserError::UnexpectedToken {
                            expected: OperandTokenType::Eol.into(),
                            found: Some(
                                String::from_utf8(
                                    self.get_source_unchecked(lex.span().to_owned()).to_vec(),
                                )
                                .unwrap(),
                            ),
                        });
                    }
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
