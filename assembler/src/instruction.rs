use isa::instruction::InstructionType;

use crate::lexer::LexemesSlice;

// #[derive(Debug, EnumVariants)]
pub enum PseudoMnemonic {
    Nop, //No operation: noop converted into addi x0, x0, 0
    Mv,  //copies value between register: e.g. mv 15, 17 converted into addi a5, a7 0
    Li,  // Load immediate
}

#[derive(Debug)]
pub enum PseudoInstruction {}

pub trait OperandType {}

#[derive(Debug)]
pub struct Operands<T>
where
    T: OperandType,
{
    dest: isa::Register,
    src1: T,
    src2: T,
}

// impl<'a, T> From<&LexemesSlice<'a>> for Operands<T> {
//     fn from(value: &LexemesSlice<'a>) -> Self {
//         let mut tokens_iter = value.tokens().iter();
//         let dest = {
//             match tokens_iter.next().unwrap() {
//                 Token::Identifier(IdentifierType::Register(reg)) => *reg,
//                 _ => isa::Register::X0,
//             }
//         };

//         let src1 = ;
//         // remove the noise e.g comma, parentheses
//         for t in tokens_iter.skip(1) {

//         }

//         Self {
//             dest,
//             src1: todo!(),
//             src2: todo!(),
//         }
//     }
// }
