use crate::{
    cpu::CPU,
    memory::{LinearMemory, Memory},
    register::instruction::{DecodeError, Instruction},
};

pub struct VM {
    cpu: CPU,
    memory: LinearMemory,
}

impl VM {
    pub fn new(memory_allocation: usize) -> Self {
        Self {
            cpu: CPU::new(),
            memory: LinearMemory::new(memory_allocation),
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while self.cpu.pc.value() < self.memory.size() {
            let opcode = self.fetch().unwrap();
            self.cpu.pc.increment();
            self.decode(opcode)?
        }

        Ok(())
    }
}

impl VM {
    fn fetch(&self) -> Result<Instruction, DecodeError> {
        let memory: u32 = self.memory.read(self.cpu.pc.value()).unwrap();
        memory.try_into()
    }

    //inlined bcs of hot loop (https://nnethercote.github.io/perf-book/inlining.html)
    #[inline(always)]
    fn decode(&self, opcode: Instruction) -> Result<(), ()> {
        match opcode {
            Instruction::Nop => todo!(),
            Instruction::LoadWord { dest, src } => todo!(),
            Instruction::StoreWord { src, dest } => todo!(),
            Instruction::Move { dest, src } => todo!(),
            Instruction::Push { src } => todo!(),
            Instruction::Pop { dest } => todo!(),
            Instruction::Add { dest, src1, src2 } => todo!(),
            Instruction::Sub { dest, src1, src2 } => todo!(),
            Instruction::Mul { dest, src1, src2 } => todo!(),
            // Instruction::Div { dest, src1, src2 } => todo!(),
            Instruction::And { dest, src1, src2 } => todo!(),
            Instruction::Or { dest, src1, src2 } => todo!(),
            Instruction::Xor { dest, src1, src2 } => todo!(),
            Instruction::Shl { dest, src, shift } => todo!(),
            Instruction::Shr { dest, src, shift } => todo!(),
            Instruction::Cmp { left, right } => todo!(),
            Instruction::Jmp { target } => todo!(),
            // Instruction::Je { target } => todo!(),
            // Instruction::Jne { target } => todo!(),
            // Instruction::Jg { target } => todo!(),
            // Instruction::Jl { target } => todo!(),
            Instruction::Call { target } => todo!(),
            Instruction::Ret => todo!(),
            Instruction::Syscall { number } => todo!(),
            Instruction::Halt => todo!(),
        }
    }
}
