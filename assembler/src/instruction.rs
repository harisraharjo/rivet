// use shared::EnumVariants;

// #[derive(Debug, EnumVariants)]
pub enum PseudoInstruction {
    Nop, //No operation: noop converted into addi x0, x0, 0
    Mv,  //copies value between register: e.g. mv 15, 17 converted into addi a5, a7 0
    Li,  // Load immediate
}
