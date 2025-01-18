use crate::{
    cpu::CPU,
    instruction::{register::Register, DecodeError, Instruction},
    memory::{LinearMemory, Memory, MemoryError},
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

    #[cfg(test)]
    pub fn test_run(&mut self, program: &[Instruction]) -> Result<(), Box<dyn std::error::Error>> {
        use crate::instruction::*;
        use crate::memory::Load;

        let program_words: Vec<u32> = program.iter().map(|x| x.into()).collect();
        unsafe {
            let program_bytes = program_words.align_to::<u8>().1;
            self.memory
                .load_from_vec(program_bytes, 0)
                .map_err(Box::new)?;
            // https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closure
        }

        //  if r == Register::Zero {
        //     return;
        // };
        self.cpu.registers.set(register::Register::X2, 1024 * 3);

        while !self.halt {
            self.step().map_err(|_| "error terminating..")?;
        }
        Ok(())
    }
}

struct TTest {
    a: u32,
}

impl VM {
    fn fetch(&self) -> Result<Instruction, DecodeError> {
        let memory: u32 = self.memory.read(self.cpu.pc.value()).unwrap();
        let TTest { a } = &TTest { a: 11 };

        let gg = a & 0xFF;
        memory.try_into()
    }

    //inlined bcs of hot loop (https://nnethercote.github.io/perf-book/inlining.html)
    #[inline(always)]
    fn decode_execute(&mut self, opcode: Instruction) -> Result<(), ()> {
        match opcode {
            Instruction::Li { dest, value } => {
                let mut ff = 41u32;
                ff |= ((10u16 as u32) & 0xFF) << 16;
                self.cpu.registers.set(dest, value.into());
                Ok(())
            }
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
                // TODO: Check me later

                let reg = Register::from(dest as u32);
                self.cpu
                    .registers
                    .set(dest, (src as u32) + self.cpu.registers.get(reg));
                Ok(())
            }
            // Instruction::LoadWord { dest, src, offset } => {
            //     let base = self.cpu.registers.get(src);
            //     let addr = offset + base;
            //     let w = self.memory.read(addr as usize).unwrap();
            //     self.cpu.registers.set(dest, w);
            //     Ok(())
            // }
            // Instruction::StoreWord { dest, src, offset } => {
            //     let base = self.cpu.registers.get(src);
            //     self.memory
            //         .write((offset + base) as usize, self.cpu.registers.get(dest))
            //         .unwrap();
            //     Ok(())
            // }
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

#[cfg(test)]
mod test {
    use register::Register;
    use Instruction::*;

    use super::*;
    use crate::instruction::*;

    // fn run_program_code(vm: &mut VM, program: &[Instruction]) -> Result<(), String> {
    //
    // }

    const CASES: [(u16, u16); 10] = [
        (1, 1),
        (2, 2),
        (12, 1),
        (2, 4),
        (32, 33),
        (111, 112),
        (1000, 52),
        (201, 97),
        (333, 333),
        (300, 20),
    ];

    #[test]
    fn t_run() {
        for (a, b) in CASES {
            let mut vm = VM::new(1024);
            let program = &[
                Add {
                    dest: todo!(),
                    src1: todo!(),
                    src2: todo!(),
                },
                // Imm(Register::X2, Register::X2),
                // Add(Register::X2, Register::X2, Register::X2),
                // System(Zero, Zero, Nibble::new_checked(SIGHALT).unwrap()),
            ];
            // vm.test_run(program)
        }
        // vm.run();
    }
}
