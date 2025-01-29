use crate::{
    cpu::CPU,
    instruction::{register::Register, DecodeError, Instruction},
    memory::{MemoryError, MemoryManager},
};

pub struct VM {
    cpu: CPU,
    // memory: LinearMemory,
    memory: MemoryManager,
    halt: bool,
}

impl VM {
    pub fn new(size: usize) -> Self {
        Self {
            cpu: CPU::new(),
            memory: MemoryManager::new(size),
            halt: false,
        }
    }

    pub fn reset(&mut self) {
        // let _ = self.memory.zero_all();
        // self.flags = 0;
        self.cpu.registers.reset();
        self.halt = false;
        self.cpu.pc.reset();
    }

    pub fn run(&mut self) -> Result<(), ()> {
        // while self.cpu.pc.value() < self.memory.size() {
        //     self.step()?;
        // }

        while !self.halt {
            self.step()?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), ()> {
        match self.fetch() {
            Ok::<Instruction, _>(instruction) => {
                self.cpu.pc.increment();
                self.decode_execute(instruction)
            }
            Err(e) => {
                // TODO: Not like this
                println!("Can't decode from memory: {:#?}", e);
                Err(())
            }
        }
    }

    // #[cfg(test)]
    // pub fn load_from_vec<T>(&mut self, program: &[T], addr: u32) -> Result<(), MemoryError>
    // where
    //     T: Copy,
    //     M: ReadWrite<T>,
    // {
    //     for (i, b) in program.iter().enumerate() {
    //         let addr = addr + ((i as u32) * 4);
    //         self.memory.write(addr, *b)?
    //     }
    //     Ok(())
    // }

    #[cfg(test)]
    pub fn test_run(&mut self, program: &[Instruction]) -> Result<(), MemoryError> {
        let program_words: Vec<u32> = program
            .iter()
            .map(|instruction| instruction.into())
            .collect();

        unsafe {
            let program_bytes = program_words.align_to::<u8>().1;
            self.memory.load_program(program_bytes, 0)?;
            // .map_err(Box::new)?;
        }

        // self.cpu
        //     .registers
        //     .set(register::Register::SP, self.memory.size() as u32);

        while !self.halt {
            self.step().unwrap();
        }

        // while self.cpu.pc.value() < self.memory.size() && !self.halt {
        //     self.step()?;
        // }

        Ok(())
    }
}

impl VM {
    fn fetch(&self) -> Result<Instruction, DecodeError> {
        // TODO: unify the error
        let memory = self.memory.read::<u32>(self.cpu.pc.value()).unwrap();

        memory.try_into()
    }

    //inlined bcs of hot loop (https://nnethercote.github.io/perf-book/inlining.html)
    #[inline(always)]
    fn decode_execute(&mut self, opcode: Instruction) -> Result<(), ()> {
        match opcode {
            Instruction::Li { dest, value } => {
                self.cpu.registers.set(dest, value as u32);
                // let opo = 33u32.get_bit(3);
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
            Instruction::AddI { dest, src, value } => {
                // TODO: Check me later

                let reg = Register::from(dest as u32);
                self.cpu
                    .registers
                    .set(dest, (src as u32) + self.cpu.registers.get(reg));
                Ok(())
            }
            Instruction::LoadWord { dest, src, offset } => {
                let base = self.cpu.registers.get(src);
                let addr = offset as u32 + base;
                let w = self.memory.read(addr).unwrap();
                self.cpu.registers.set(dest, w);
                Ok(())
            }
            Instruction::StoreWord { dest, src, offset } => {
                let base = self.cpu.registers.get(src);
                self.memory
                    .write(offset as u32 + base, self.cpu.registers.get(dest))
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
            Instruction::Syscall { src1, src2, src3 } => {
                self.halt = true;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use register::Register;
    use Instruction::*;

    use super::*;
    use crate::instruction::*;

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
    fn t_arith() {
        let size = 1024 * 1024;

        let mut vm = VM::new(size);
        for (a, b) in CASES {
            let program = &[
                Li {
                    dest: Register::T1,
                    value: a as u32,
                },
                Li {
                    dest: Register::T2,
                    value: b as u32,
                },
                Add {
                    dest: Register::T3,
                    src1: Register::T2,
                    src2: Register::T1,
                },
                Syscall {
                    src1: Register::Zero,
                    src2: Register::Zero,
                    src3: Register::Zero,
                },
            ];

            match vm.test_run(program) {
                Ok(e) => println!("Test run Ok"),
                Err(e) => println!("Test run went wrong"),
            }
            assert_eq!(vm.cpu.registers.get(Register::T3), (a + b) as u32);
            vm.reset();
        }
    }

    #[test]
    fn t_load_store() -> Result<(), DecodeError> {
        let size = 1024 * 1024;

        let mut vm = VM::new(size);

        // for (a, b) in CASES {
        //     let program = &[
        //         Li {
        //             dest: Register::T1,
        //             value: a as u32,
        //         },
        //         Li {
        //             dest: Register::T2,
        //             value: b as u32,
        //         },
        //         Add {
        //             dest: Register::T3,
        //             src1: Register::T2,
        //             src2: Register::T1,
        //         },
        //         Syscall {
        //             src1: Register::Zero,
        //             src2: Register::Zero,
        //             src3: Register::Zero,
        //         },
        //     ];

        //     vm.test_run(program).unwrap();
        //     assert_eq!(vm.cpu.registers.get(Register::T3), (a + b) as u32);
        //     vm.reset();
        // }

        // let store = Instruction::StoreWord { dest: Register::A0, src: Register::A1, offset: () }

        Ok(())
    }
}
