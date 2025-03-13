use std::ops::Range;

use isa::operand::{Immediate14, Immediate19};
use shared::{EnumCount, EnumVariants};

use crate::{
    lexer::{Lexeme, LexemesSlice},
    parser::grammar::OperandRuleType,
    token::{self, LiteralIntegerType},
};

#[derive(Debug, EnumVariants, EnumCount)]
pub enum PseudoMnemonic {
    Nop, //No operation: noop converted into addi x0, x0, 0
    Mv,  //copies value between register: e.g. mv 15, 17 converted into addi a5, a7 0
    Li,  // Load immediate
}

#[derive(Debug)]
pub struct Instruction {
    mnemonic: isa::instruction::Mnemonic,
    operands: Operands,
}

impl Instruction {
    pub fn new(mnemonic: isa::instruction::Mnemonic, operands: Operands) -> Instruction {
        Instruction { mnemonic, operands }
    }
}

#[derive(Debug)]
pub struct Operands {
    dest: OperandType,
    src1: OperandType,
    src2: OperandType,
}

pub type Source<'a> = &'a [u8];
impl<'a> From<(&mut LexemesSlice<'a>, OperandRuleType, Source<'a>)> for Operands {
    fn from((lexemes, rule, source): (&mut LexemesSlice<'a>, OperandRuleType, Source<'a>)) -> Self {
        let mut iter = lexemes
            //`step_by(2)` to skip noises e.g. Comma, ParenL, ParenR
            .step_by(2)
            .map(|lexeme| -> OperandType {
                println!("Lexeme: {:?}", lexeme);
                (lexeme, rule, source).into()
            });

        Self {
            dest: iter.next().unwrap(),
            src1: iter.next().unwrap(),
            src2: iter.next().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default)]
pub enum OperandType {
    Symbol(Range<usize>),
    Label(Range<usize>),
    Register(isa::Register),
    Literal14(isa::operand::Immediate14),
    Literal19(isa::operand::Immediate19),
    #[default]
    Unknown,
}

impl<'a> From<(Lexeme<'a>, OperandRuleType, Source<'a>)> for OperandType {
    fn from((lexeme, rule, source): (Lexeme<'a>, OperandRuleType, Source<'a>)) -> Self {
        use OperandRuleType::*;
        use token::Token::*;

        let token = *lexeme.token();

        match (token, rule) {
            (Identifier(token::IdentifierType::Symbol), _) => {
                Self::Symbol(lexeme.span().to_owned())
            }
            (Identifier(token::IdentifierType::Register(r)), _) => Self::Register(r),
            (Label, _) => Self::Label(lexeme.span().to_owned()),
            (LiteralDecimal | LiteralHex | LiteralBinary, R2I | RIR | RI) => {
                let src = source.get(lexeme.span().to_owned()).unwrap();
                let int_ty = LiteralIntegerType::from(token);
                let target =
                    LiteralIntegerType::filter(src, LiteralIntegerType::head_len(int_ty as u8));
                let src_str = std::str::from_utf8(target).unwrap();
                // todo: should not unwrap to check if it's inside
                let imm = i32::from_str_radix(src_str, int_ty.base()).unwrap();

                match rule {
                    R2I | RIR => Self::Literal14(Immediate14::new(imm)),
                    _ => Self::Literal19(Immediate19::new(imm)),
                }
            }
            _ => Self::Unknown,
        }
    }
}
