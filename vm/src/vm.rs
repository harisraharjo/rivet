use crate::{
    cpu::{Registers, CPU},
    instruction::{DecodeError, Instruction},
    memory::{LinearMemory, Memory},
};

pub struct VM {
    cpu: CPU,
    memory: LinearMemory,
    halt: bool,
}

impl VM {
    pub fn new(memory_allocation: usize) -> Self {
        Self {
            cpu: CPU::new(),
            memory: LinearMemory::new(memory_allocation),
            halt: false,
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while self.cpu.pc.value() < self.memory.size() {
            self.step()?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), ()> {
        let opcode = self.fetch().unwrap();
        self.cpu.pc.increment();
        self.decode_execute(opcode)
    }
}

impl VM {
    fn fetch(&self) -> Result<Instruction, DecodeError> {
        let memory: u32 = self.memory.read(self.cpu.pc.value()).unwrap();
        memory.try_into()
    }

    //inlined bcs of hot loop (https://nnethercote.github.io/perf-book/inlining.html)
    #[inline(always)]
    fn decode_execute(&mut self, opcode: Instruction) -> Result<(), ()> {
        match opcode {
            Instruction::Nop => todo!(),
            Instruction::Add { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                let (value, overflow) = r0.overflowing_add(r1);
                self.cpu.registers.set(dest, value);

                Ok(())
            }
            Instruction::Sub { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                let (value, overflow) = r0.overflowing_sub(r1);
                self.cpu.registers.set(dest, value);

                Ok(())
            }
            Instruction::Mul { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                let (value, overflow) = r0.overflowing_mul(r1);
                self.cpu.registers.set(dest, value);

                Ok(())
            }
            Instruction::And { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.cpu.registers.set(dest, r0 & r1);
                Ok(())
            }
            Instruction::Or { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.cpu.registers.set(dest, r0 | r1);
                Ok(())
            }
            Instruction::Xor { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.cpu.registers.set(dest, r0 ^ r1);
                Ok(())
            }
            Instruction::AddI { dest, src } => {
                self.cpu
                    .registers
                    .set(dest, src + self.cpu.registers.get(dest));
                Ok(())
            }
            Instruction::LoadWord { dest, src, offset } => {
                let base = self.cpu.registers.get(src);
                let addr = offset + base;
                let w = self.memory.read(addr as usize).unwrap();
                self.cpu.registers.set(dest, w);
                Ok(())
            }
            Instruction::StoreWord { dest, src, offset } => {
                let base = self.cpu.registers.get(src);
                self.memory
                    .write((offset + base) as usize, self.cpu.registers.get(dest))
                    .unwrap();
                Ok(())
            }
            Instruction::Shl { dest, src, shift } => {
                let base = self.cpu.registers.get(src);
                self.cpu.registers.set(dest, base << shift);
                Ok(())
            }
            Instruction::Shr { dest, src, shift } => {
                let base = self.cpu.registers.get(src);
                self.cpu.registers.set(dest, base >> shift);
                Ok(())
            }
            Instruction::ShrA { dest, src, shift } => {
                let base = self.cpu.registers.get(src) as i32;
                self.cpu.registers.set(dest, (base >> shift) as u32);
                Ok(())
            }
            // Instruction::Jal { dest, src, shift } => todo!(),
            Instruction::Syscall { number } => todo!(),
            // Instruction::Halt => {
            //     self.halt = true;
            //     Ok(())
            // }
        }
    }
}
