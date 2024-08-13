use crate::{
    memory::LinearMemory,
    register::{
        instruction::{InstructionHandler, Opcode},
        pc::ProgramCounter,
        Registers,
    },
};

pub struct VM {
    registers: Registers,
    memory: LinearMemory,
}

impl VM {
    pub fn new(memory_allocation: usize) -> Self {
        Self {
            registers: Registers::new(),
            memory: LinearMemory::new(memory_allocation),
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        let mut pc = ProgramCounter::new();
        while pc.count() < self.memory.size() {
            let opcode = self.fetch(pc.count() as u8);
            pc.increment();
            self.decode(opcode)?
        }

        Ok(())
    }
}

impl InstructionHandler for VM {
    //inlined bcs of hot loop (https://nnethercote.github.io/perf-book/inlining.html)
    #[inline(always)]
    fn decode(&self, opcode: Opcode) -> Result<(), ()> {
        match opcode {
            Opcode::HLT => {
                println!("HLT encountered");
                Ok(())
            }
            Opcode::LOAD => {
                todo!()
            }
            Opcode::ADD => todo!(),
            Opcode::SUB => todo!(),
            Opcode::MUL => todo!(),
            Opcode::DIV => todo!(),
            Opcode::IGL => {
                println!("Unrecognized opcode. Terminating...");
                Ok(())
            }
        }
    }
}
