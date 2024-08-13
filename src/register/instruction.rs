#[derive(Debug)]
pub enum Opcode {
    /// HALT
    HLT,
    /// Illegal
    IGL,
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::HLT,
            _ => Opcode::IGL,
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction { opcode }
    }
}

pub trait InstructionHandler {
    #[inline(always)]
    fn fetch(&self, memory: u8) -> Opcode {
        Opcode::from(memory)
    }

    fn decode(&self, opcode: Opcode) -> Result<(), ()>;
}
