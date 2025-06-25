pub mod grammar;

use grammar::{OperandRuleType, RuleToken};
use shared::{ChunksExt, EnumCount};
use std::{
    fmt::{Debug, Display},
    ops::Range,
};
use thiserror::Error;

use crate::{
    asm::directive::DirectiveType,
    exprs::Exprs,
    instruction::{Operand, OperandError, Operands, OperandsIndex},
    interner::StrId,
    ir::{IR, IRError, Node},
    lexer::{Lexeme, Lexemes, LexemesSlice},
    symbol_table::{ConstantSymbol, SymbolError, SymbolTable},
    token::{self, IdentifierType, Token},
};

fn on_invalid_grammar<'a>(found: &Option<String>) -> String {
    let Some(v) = found else {
        return Default::default();
    };

    format!(" found {v}")
}

#[derive(Error, Debug)]
pub enum ParsingError {
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
    #[error("{0} is still work in progress. Stay tuned!")]
    UnimplementedFeature(RuntimeTodo),
    // #[error("Duplicate label {0}")]
    // DuplicateLabel(String),
    #[error(transparent)]
    ValueError(#[from] OperandError),
    #[error(transparent)]
    IRError(#[from] IRError),
    #[error(" symbol {0}")]
    SymbolError(#[from] SymbolError),
    //     #[error("Undefined symbol: {0}")]
    //     UndefinedSymbol(String),
}

/// Return `Ok(lexeme)` if correct, otherwise returns error `Err(ParseError::UnexpectedToken)`
macro_rules! expect_token {
    ($lexeme:expr, None) => {
        match $lexeme {
            Some(l) => Err(ParsingError::UnexpectedToken {
                expected: RuleToken::Break,
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
                t @ _ => Err(ParsingError::UnexpectedToken {
                    expected: $ex,
                    found: Some(t.to_string()),
                }),
            },
            None => Err(ParsingError::UnexpectedToken {
                expected: $ex,
                found: None,
            }),
        }
    };
}
pub(crate) use expect_token;

#[derive(Debug)]
pub struct ParsedData {
    symtab: SymbolTable,
    ir: IR,
}

pub struct Parser<'a> {
    lexemes: Lexemes,
    index: usize,
    source: &'a [u8],
    ir: IR,
    symtab: SymbolTable,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes) -> Self {
        // let cap = source.len();
        Parser {
            lexemes,
            index: 0,
            source,
            // constants: ConstantSymbols::default(),
            ir: IR::new(10),
            symtab: SymbolTable::new(),
        }
    }

    pub fn parse(mut self) -> Result<ParsedData, ParsingError> {
        while let Some(token) = self.eat() {
            self.walk(*token)?;
        }

        // self.ir.print_sections();
        self.reset();

        let parsed = ParsedData {
            symtab: self.symtab,
            ir: self.ir,
        };

        Ok(parsed)
    }

    pub fn ir(&self) -> &IR {
        &self.ir
    }

    #[inline(always)]
    fn next_index(&self) -> usize {
        self.index + 1
    }

    fn peek(&self) -> Option<Lexeme<'_>> {
        self.lexemes.get(self.next_index())
    }

    fn reset(&mut self) {
        self.index = 0;
    }

    /// Peek the next `N` token
    fn peek_n(&self, index: usize) -> Option<Lexeme<'_>> {
        self.lexemes.get(self.index + index)
    }

    /// Peek until the next (1 + `N`) token
    fn peek_until(&self, index: usize) -> Range<usize> {
        let next_idx = self.next_index();
        next_idx..next_idx + index
    }

    /// peek the current line indices
    fn peek_line_indices(&self) -> Range<usize> {
        self.peek_until(self.next_line_index())
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        //safety: unwrap is safe because guaranteed (Token::Eol || Token::Eof) is always present
        self.lexemes.slice(self.peek_line_indices()).unwrap()
    }

    fn peek_token(&self) -> Option<&Token> {
        self.lexemes.get_token(self.next_index())
    }

    fn remainder_token(&self) -> &[Token] {
        // safety: Read until the end. guaranteed to be safe
        &self.lexemes.tokens()[self.next_index()..]
    }

    fn eat(&self) -> Option<&Token> {
        self.lexemes.tokens().get(self.index)
    }

    fn nearest_break_idx(&self) -> Option<usize> {
        self.remainder_token()
            .iter()
            .position(|t| *t == Token::Eol || *t == Token::Eof)
    }

    // Get the index of the the next line. Eol||Eof is included
    fn next_line_index(&self) -> usize {
        self.nearest_break_idx().unwrap() + 1
    }

    fn advance_line(&mut self) {
        self.advance_by(self.next_line_index());
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

    fn current_span(&self) -> &Range<usize> {
        self.get_span(self.index)
    }

    fn current_source(&self) -> &'a [u8] {
        // safety: safe because the span is guaranteed to be inside the bounds
        self.source.get(self.current_span().to_owned()).unwrap()
    }

    fn walk(&mut self, token: Token) -> Result<(), ParsingError> {
        match token {
            Token::Directive(dir_type) => {
                use crate::asm::directive::DirectiveType;

                match dir_type {
                    DirectiveType::Section => {
                        expect_token!(self.peek(), token::section_dir!(), RuleToken::SectionDir)?;
                    }
                    DirectiveType::Text
                    | DirectiveType::Data
                    | DirectiveType::Rodata
                    | DirectiveType::Bss
                    | DirectiveType::CustomSection => {
                        expect_token!(self.peek(), token::break_kind!(), RuleToken::Break)?;

                        self.ir.add_section(
                            std::str::from_utf8(self.current_source()).unwrap(),
                            dir_type.into(),
                        );
                    }
                    DirectiveType::Ascii => {
                        let lexeme = expect_token!(
                            self.peek(),
                            Token::LiteralString,
                            RuleToken::LiteralString
                        )?;
                        expect_token!(self.peek_n(2), token::break_kind!(), RuleToken::Break)?;

                        let slice = self.source.get(lexeme.span().to_owned()).unwrap();
                        // safety: guaranteed safe because it's valid utf8. Taken from the implementation of `String.to_box_slice()`
                        let box_str = unsafe { std::str::from_boxed_utf8_unchecked(slice.into()) };

                        self.ir.push(Node::String(box_str));
                    }
                    DirectiveType::Global => {
                        let symbol = expect_token!(
                            self.peek(),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;
                        expect_token!(self.peek_n(2), token::break_kind!(), RuleToken::Break)?;

                        let slice = self.source.get(symbol.span().to_owned()).unwrap();
                        let name_str = std::str::from_utf8(slice).unwrap();
                        let str_id = self.ir.alloc_str(name_str);

                        self.symtab
                            .declare_global(self.ir.active_section(), str_id, name_str)?;

                        self.advance();
                    }
                    DirectiveType::Set | DirectiveType::Equ => {
                        let line_range = self.peek_line_indices();
                        let indices_len = line_range.len();

                        let mut range_chunks = line_range.chunks(2);
                        let first_chunk = range_chunks.next().unwrap();

                        let constant = expect_token!(
                            self.lexemes.get(*first_chunk.start()),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;
                        expect_token!(
                            self.lexemes.get(*first_chunk.end()),
                            Token::Comma,
                            RuleToken::Comma
                        )?;

                        let cap_without_eol = indices_len - 1;
                        let mut exprs = Exprs::new(cap_without_eol);
                        // TODO: Doesn't feel right. Refactor this `exprs.build(..)`
                        exprs.build(
                            range_chunks,
                            &self.lexemes,
                            &self.source,
                            |name| -> StrId { self.ir.alloc_str(name) },
                        )?;

                        let constant_str = std::str::from_utf8(
                            self.source.get(constant.span().to_owned()).unwrap(),
                        )
                        .unwrap();

                        let str_id = self.ir.alloc_str(constant_str);
                        self.symtab.insert_constant(
                            dir_type.into(),
                            str_id,
                            exprs,
                            constant_str,
                        )?;

                        self.advance_line();
                    }
                    DirectiveType::Byte | DirectiveType::Half | DirectiveType::Word => {
                        return Err(ParsingError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Align | DirectiveType::Balign | DirectiveType::P2align => {
                        return Err(ParsingError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Skip => {
                        return Err(ParsingError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Comm | DirectiveType::LComm => {
                        return Err(ParsingError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::String | DirectiveType::Asciz => {
                        return Err(ParsingError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
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

                let name_str = std::str::from_utf8(self.current_source()).unwrap();
                let str_id = self.ir.alloc_str(name_str);
                self.symtab
                    .insert(self.ir.active_section(), str_id, name_str)?;

                self.ir.push(Node::Label(str_id));
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic)) => {
                let mut lexemes = self.peek_line();
                let mut rule = grammar::InstructionRule::new(mnemonic);
                let rule_sequence = rule.generate_sequence();

                // Syntax analysis
                if let Some(mismatch) = rule_sequence
                    .iter()
                    .zip(lexemes.by_ref())
                    .find(|(rule_token, lex)| *lex.token() != **rule_token)
                {
                    let (rule_token, lexeme) = mismatch;

                    return Err(ParsingError::UnexpectedToken {
                        expected: *rule_token,
                        found: std::str::from_utf8(
                            self.source.get(lexeme.span().to_owned()).unwrap(),
                        )
                        .unwrap()
                        .to_owned()
                        .into(),
                    });
                };

                let seq_len = rule_sequence.len();
                let rule_residue = seq_len.saturating_sub(lexemes.token_len());
                // check whether the input is:
                match (rule_residue > 0, &lexemes.next()) {
                    //too little
                    (true, None) => {
                        let rule_token = rule_sequence[seq_len - rule_residue];
                        return Err(ParsingError::UnexpectedToken {
                            expected: rule_token,
                            found: None,
                        });
                    }
                    //too much
                    (false, Some(lexeme)) => {
                        if *lexeme.token() != RuleToken::Break {
                            return Err(ParsingError::UnexpectedToken {
                                expected: RuleToken::Break,
                                found: std::str::from_utf8(
                                    self.source.get(lexeme.span().to_owned()).unwrap(),
                                )
                                .unwrap()
                                .to_owned()
                                .into(),
                            });
                        }
                    }
                    //impossible
                    _ => {}
                };
                let rule_ty = rule.ty();
                drop(rule);

                // Value analysis
                lexemes.reset();

                // Iterate over indices instead of the actual item to bypass the error `cannot borrow as mutable bcs it's also borrowed as immutable`
                let mut remainder_range = lexemes.range_index().clone();
                // remove Eol/Eof
                remainder_range.end -= 1;

                let range_iter = remainder_range.step_by(OperandRuleType::noises_in_every());

                let mut operand_types = [Operand::None; OperandsIndex::VARIANT_COUNT];
                for (i, op_idx) in range_iter.zip(0..OperandsIndex::VARIANT_COUNT + 1) {
                    let lexeme = self.lexemes.get_unchecked(i);
                    let slice = self.source.get(lexeme.span().to_owned()).unwrap();
                    let token = *lexeme.token();

                    let mut operand: Operand = (token, rule_ty, slice).try_into()?;
                    if let Operand::Symbol(ref mut str_id) = operand {
                        *str_id = self.ir.alloc_str(std::str::from_utf8(slice).unwrap());
                    }

                    operand_types[op_idx] = operand;
                }

                let mut operands = Operands::new();
                operands.memcpy(&operand_types);
                let ins = crate::instruction::Instruction::new(mnemonic, operands);
                println!("Instruction IR: {:?}", ins);

                // // let pseudo = PseudoInstruction
                self.ir.add_instruction(ins);
                self.advance_line();
            }
            token::break_kind!() => {
                println!("=== BREAK ===");
            }
            _ => {
                return Err(ParsingError::SyntaxError);
            }
        }

        self.advance();
        Ok(())
    }
}

#[derive(Debug)]
pub enum RuntimeTodo {
    // #[errortra]
    Dir(DirectiveType),
    // #[error("symbol")]
    // Symbol,
}

impl Display for RuntimeTodo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeTodo::Dir(directive_type) => Display::fmt(directive_type, f),
            // RuntimeTodo::Symbol => write!(f, "{}", "symbol"),
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
        .set symbol1, 1 + 2 - 3
        .set symdupe, 1
        symdupe:
            .global MY_PENDING_GLOBAL
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
        // .set symbol5, 1 + 1 + 2 + 1 + 1 + 2 + 1 + 1
        .section .my_cust
        .section .bss
        .section .rodata
        .section .data
        "#;

        // ex
        // Instructions: 4 bytes each (RV32I).
        // Word: 4 bytes.
        // Byte: 1 byte.
        // Half: 2 bytes.
        // String: Length of string + null terminator.
        // Align: Pad to the specified boundary (e.g., .align 2 → 4 bytes).
        // Skip: Add uninitialized bytes.
        // let name = String::from_utf8_lossy(&self.source[span.clone()]).to_string();

        let source = raw_source.as_bytes();

        // let mut symbol_table = SymbolTable::new();
        let lexemes = lex.tokenize(source).unwrap();

        let parser = Parser::new(source, lexemes);
        assert!(match parser.parse() {
            Ok(parsed_data) => {
                println!("Symtab: {:?}", parsed_data.symtab);
                true
            }
            Err(e) => panic!("{e}"),
        })
    }

    #[test]
    fn t_p_global() {
        let lex = Lexer::new();

        let raw_source = r#"
        .section .text
        .global MY_PENDING_GLOBAL
        MY_PENDING_GLOBAL:
        // MY_PENDING_GLOBAL:
        main:
            .global main"#;

        let source = raw_source.as_bytes();

        // let mut symbol_table = SymbolTable::new();
        let lexemes = lex.tokenize(source).unwrap();

        let parser = Parser::new(source, lexemes);
        assert!(match parser.parse() {
            Ok(parsed_data) => {
                println!("Parsed Data: {:?}", parsed_data);
                true
            }
            Err(e) => panic!("{e}"),
        })
    }
}
