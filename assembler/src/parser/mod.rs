pub mod grammar;

// use bumpalo::{Bump, collections::Vec as BumpVec};
use grammar::{OperandRuleType, RuleToken};
use std::{
    fmt::{Debug, Display},
    ops::Range,
};
use thiserror::Error;

use crate::{
    asm::{directive::DirectiveType, section::Section},
    instruction::{Operand, OperandError, Operands},
    interner::Interner,
    ir::{Expr, Exprs, IRError, Node},
    lexer::{Lexeme, Lexemes, LexemesSlice, RangeExt},
    symbol_table::{ConstantSymbol, ConstantSymbols, SymbolError},
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
    UnimplementedFeature(RuntimeTodo),
    #[error("Duplicate label {0}")]
    DuplicateLabel(String),
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
            Some(l) => Err(ParserError::UnexpectedToken {
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

#[derive(Debug)]
pub struct IR {
    // intern: ustr
    // nodes: BumpVec<'bump, &'bump dyn Debug>
    nodes: Vec<Node>,
    str_tab: Interner,
}

impl IR {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            str_tab: Interner::with_capacity(0),
        }
    }

    fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }

    #[cfg(test)]
    fn interns(&self) -> &Interner {
        &self.str_tab
    }
}

pub struct Parser<'a> {
    lexemes: Lexemes,
    index: usize,
    source: &'a [u8],
    // sections: Sections,
    constants: ConstantSymbols,
    // arena: bumpalo::Bump,
    ir: IR,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [u8], lexemes: Lexemes) -> Self {
        // let cap = source.len();
        Parser {
            lexemes,
            index: 0,
            source,
            // sections: Sections::default(),
            constants: ConstantSymbols::default(),
            // arena: Bump::with_capacity(cap),
            ir: IR::new(),
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
    fn peek_until(&self, index: usize) -> Range<usize> {
        let next_idx = self.next_index();
        next_idx..next_idx + index
    }

    /// peek the current line indices
    fn peek_line_indices(&self) -> Range<usize> {
        //// upper bound is at the index of Eol||Eof because we don't take them in
        // Eol||Eof is included
        self.peek_until(self.nearest_break_idx().unwrap() + 1)
    }

    /// peek the current line
    fn peek_line(&self) -> LexemesSlice<'_> {
        //safety: unwrap is safe because guaranteed (Token::Eol || Token::Eof) is always present
        self.lexemes.slice(self.peek_line_indices()).unwrap()
    }

    fn peek_token(&self) -> Option<&Token> {
        self.lexemes.get_token(self.next_index())
    }

    pub fn remainder_token(&self) -> &[Token] {
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
        // let mut vec = BumpVec::new_in(&self.arena);
        // BumpVec<&dyn Debug>

        while let Some(token) = self.eat() {
            self.walk(*token)?;
            // vec.push(value);
        }

        println!("{:?}", self.ir.str_tab);
        println!("{:?}", self.ir.nodes);

        Ok(())
    }

    fn walk(&mut self, token: Token) -> Result<(), ParserError> {
        println!("Parsing... {:?}", token);
        match token {
            Token::Directive(dir_type) => {
                use crate::asm::directive::DirectiveType;

                match dir_type {
                    DirectiveType::Section => {
                        expect_token!(self.peek(), token::section_dir!(), RuleToken::SectionDir)?;
                    }
                    DirectiveType::Text | DirectiveType::Data | DirectiveType::Rodata | DirectiveType::Bss
                    // | CustomSection
                     => {
                        expect_token!(self.peek(), Token::Eol | Token::Eof, RuleToken::Break)?;

                        // self.sections
                        //     .switch(dir_type, self.current_span().to_owned());

                        let slice = self.current_source();
                        // safety: guaranteed safe because of it's valid utf8. Taken from the implementation of `String.to_box_slice()`
                        // let box_str = unsafe { std::str::from_boxed_utf8_unchecked(slice.into()) };
                        let str_id= self.ir.str_tab.intern(std::str::from_utf8(
                            slice
                        )
                        .unwrap());
                        let data = Section::new(dir_type.into(), str_id);
                        self.ir.push(Node::Section(data));
                        // curr_sect.insert(Node::String(box_str));
                    }
                    DirectiveType::Byte | DirectiveType::Half | DirectiveType::Word => {
                        return Err(ParserError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Ascii => {
                        let lexeme = expect_token!(
                            self.peek(),
                            Token::LiteralString,
                            RuleToken::LiteralString
                        )?;
                        expect_token!(self.peek_n(2), Token::Eol | Token::Eof, RuleToken::Break)?;
                        // println!("String value: {:?}", lexeme);

                        let span = lexeme.span().to_owned();
                        let slice = self.source.get(span).unwrap();
                        // safety: guaranteed safe because of it's valid utf8. Taken from the implementation of `String.to_box_slice()`
                        let box_str = unsafe { std::str::from_boxed_utf8_unchecked(slice.into()) };

                        self.ir.push(Node::String(box_str));
                    }
                    DirectiveType::Align | DirectiveType::Balign | DirectiveType::P2align => {

                        return Err(ParserError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Set | DirectiveType::Equ => {
                        let line_range = self.peek_line_indices();
                        let indices_len = line_range.len();
                        
                        let mut range_chunks = line_range.chunks(2);
                        let first_chunk = range_chunks.next().unwrap();

                        let constant_name = expect_token!(
                            self.lexemes.get(first_chunk.start),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;

                        expect_token!(self.lexemes.get(first_chunk.end-1), Token::Comma, RuleToken::Comma)?;


                        // - 1 = exclude Eol/Eof
                        let mut exprs = Exprs::with_capacity(indices_len - 1 );
                        //exprs check
                        for chunk in range_chunks {
                            let var = expect_token!(
                                self.lexemes.get(chunk.start),
                                token::symbol_or_numeric!(),
                                RuleToken::SymbolOrNumeric
                            )?;

                            let op = expect_token!(
                                self.lexemes.get(chunk.end-1),
                                token::operator!() | token::break_kind!(),
                                RuleToken::OperatorOrBreak
                            )?;

                            // let slice = self.source.get(var.span().to_owned()).unwrap();
                            // let slice = self.source.get(op.span().to_owned()).unwrap();

                            // exprs.push(Expr::try_from((*var.token(), slice))?);
                            // exprs.push(Expr::try_from((*op.token(), slice))?);
                        }

                        self.constants.insert(
                            constant_name.span().to_owned(),
                            dir_type.into(),
                            // TODO: Data for expression is not quite right still
                            ConstantSymbol::new(exprs),
                            self.source,
                        )?;
                        self.advance_by(indices_len);
                    }
                    DirectiveType::Global => {
                        let symbol = expect_token!(
                            self.peek(),
                            Token::Identifier(IdentifierType::Symbol),
                            RuleToken::Symbol
                        )?;
                        expect_token!(self.peek_n(2), Token::Eol | Token::Eof, RuleToken::Break)?;

                        let span = symbol.span().to_owned();
                        let slice = self.source.get(span).unwrap();
                        // safety: guaranteed safe because of it's valid utf8. Taken from the implementation of `String.to_box_slice()`
                        // let box_str = unsafe { std::str::from_boxed_utf8_unchecked(slice.into()) };
                        let str_id= self.ir.str_tab.intern(std::str::from_utf8(
                            slice
                        )
                        .unwrap());
                        self.ir.push(Node::Global(str_id));

                        self.advance();
                    }
                    DirectiveType::Skip => {
                        return Err(ParserError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::Comm | DirectiveType::LComm => {
                        return Err(ParserError::UnimplementedFeature(RuntimeTodo::Dir(
                            dir_type,
                        )));
                    }
                    DirectiveType::String | DirectiveType::Asciz => {
                        return Err(ParserError::UnimplementedFeature(RuntimeTodo::Dir(
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

                let slice = self.current_source();
                let str_id = self.ir.str_tab.intern(std::str::from_utf8(slice).unwrap());

                self.ir.push(Node::Label(str_id));
                // self.ir.push(Node::Label(Symbol::new(
                //     str_id,
                //     Visibility::Local,
                //     None,
                //     SymbolType::Label,
                // )));
            }
            Token::Identifier(IdentifierType::Mnemonic(mnemonic)) => {
                let mut lexemes = self.peek_line();
                let mut rule = grammar::InstructionRule::new(mnemonic);
                let rule_sequence = rule.generate_sequence();

                // Syntax analysis
                if let Some(mismatch) = rule_sequence
                    .iter()
                    .zip(lexemes.by_ref())
                    .filter(|(rule_token, lex)| lex.token().to_owned() != **rule_token)
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

                let seq_len = rule_sequence.len();
                let rule_residue = seq_len.saturating_sub(lexemes.token_len());
                // check whether the input is:
                match (rule_residue > 0, &lexemes.next()) {
                    //too little
                    (true, None) => {
                        let rule_token = rule_sequence[seq_len - rule_residue];
                        return Err(ParserError::UnexpectedToken {
                            expected: rule_token,
                            found: None,
                        });
                    }
                    //too much
                    (false, Some(lexeme)) => {
                        if *lexeme.token() != RuleToken::Break {
                            return Err(ParserError::UnexpectedToken {
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
                let mut remainder_range = lexemes
                    .range_index()
                    .clone();
                // remove Eol/Eof
                remainder_range.end -= 1;
                let range_iter =  remainder_range.step_by(OperandRuleType::noises_in_every());
                let mut operand_types = Vec::with_capacity(range_iter.len());
                for i in range_iter {
                    let lexeme = self.lexemes.get_unchecked(i);
                    let slice = self.source.get(lexeme.span().to_owned()).unwrap();
                    let token = *lexeme.token();

                    let mut operand: Operand = (token, rule_ty, slice).try_into()?;
                    match operand {
                        Operand::Symbol(_) => {
                            let str_id =
                                self.ir.str_tab.intern(std::str::from_utf8(slice).unwrap());
                            operand = Operand::Symbol(str_id);
                        }
                        _ => {}
                    }

                    operand_types.push(operand);
                }

                let mut operands = Operands::new();
                operands.copy_from_slice(&operand_types);
                let ins = crate::instruction::Instruction::new(mnemonic, operands);
                println!("Instruction IR: {:?}", ins);

                // // let pseudo = PseudoInstruction
                self.ir.push(Node::Instruction(ins));
                self.advance_by(seq_len);
            }
            Token::Eol | Token::Eof => {
                println!("=== BREAK ===");
            }
            t @ _ => {
                println!(
                    "Unknown Token: {:?} -> {:?}",
                    t,
                    std::str::from_utf8(self.current_source())
                );
                return Err(ParserError::SyntaxError);
            }
        }

        self.advance();
        Ok(())
    }
}

#[derive(Debug)]
enum RuntimeTodo {
    // #[errortra]
    Dir(DirectiveType),
    // #[error("symbol")]
    Symbol,
}

impl Display for RuntimeTodo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeTodo::Dir(directive_type) => Display::fmt(directive_type, f),
            RuntimeTodo::Symbol => write!(f, "{}", "symbol"),
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
        // .set symbol5, 1 + 1 + 2 + 1 + 1 + 2 + 1 + 1
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
        // symbols.symbols.iter().find(|s| s.0 == name).map(|s| s.1)

        let source = raw_source.as_bytes();

        // let mut symbol_table = SymbolTable::new();
        let lexemes = lex.tokenize(source).unwrap();

        let mut parser = Parser::new(source, lexemes);
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

        // let mut symbol_table = SymbolTable::new();
        let lexemes = lex.tokenize(source).unwrap();

        let mut parser = Parser::new(source, lexemes);
        assert!(match parser.parse() {
            Ok(_) => true,
            Err(e) => panic!("{e}"),
        })
    }
}
