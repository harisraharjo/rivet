use isa::instruction::{InstructionType, Mnemonic};

// #[derive(Debug, EnumVariants)]
pub enum PseudoInstruction {
    Nop, //No operation: noop converted into addi x0, x0, 0
    Mv,  //copies value between register: e.g. mv 15, 17 converted into addi a5, a7 0
    Li,  // Load immediate
}

// pub struct Instruction {
//     mnemonic: Mnemonic,
//     rule: OperandRuleType,
//     ty: InstructionType,
// }

// impl Instruction {
//     pub fn new(mnemonic: Mnemonic) -> Instruction {
//         Instruction {
//             mnemonic,
//             rule: todo!(),
//             ty: todo!(),
//         }
//     }
// }
