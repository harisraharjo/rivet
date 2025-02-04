use crate::{
    cpu::{Registers, CPU},
    instruction::{DecodeError, Instruction},
    memory::{MemoryError, MemoryManager},
};

pub struct VM {
    cpu: CPU,
    // memory: LinearMemory,
    memory: MemoryManager,
    halt: bool,
}

impl VM {
    pub fn new(size: u32) -> Self {
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
        self.memory.reset();
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

    pub fn registers(&mut self) -> &mut Registers {
        &mut self.cpu.registers
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
            self.memory.load_program(program_bytes)?;
            // .map_err(Box::new)?;
        }

        println!("");
        println!("Program is successfully loaded");
        println!("");
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
                self.registers().set(dest, value.into());
                // let opo = 33u32.get_bit(3);
                Ok(())
            }
            Instruction::Add { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);

                let (value, overflow) = r0.overflowing_add(r1);
                self.registers().set(dest, value);

                Ok(())
            }
            Instruction::Sub { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                let (value, overflow) = r0.overflowing_sub(r1);
                self.registers().set(dest, value);

                Ok(())
            }
            Instruction::Mul { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                let (value, overflow) = r0.overflowing_mul(r1);
                self.registers().set(dest, value);

                Ok(())
            }
            Instruction::And { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.registers().set(dest, r0 & r1);
                Ok(())
            }
            Instruction::Or { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.registers().set(dest, r0 | r1);
                Ok(())
            }
            Instruction::Xor { dest, src1, src2 } => {
                let r0 = self.cpu.registers.get(src1);
                let r1 = self.cpu.registers.get(src2);
                self.registers().set(dest, r0 ^ r1);
                Ok(())
            }
            Instruction::AddI { dest, src, value } => {
                self.cpu
                    .registers
                    .set(dest, self.cpu.registers.get(src).wrapping_add(value.into()));
                Ok(())
            }
            Instruction::Lui { dest, value } => {
                /*
                **U-type** example: `LUI x1, 0x12345`.
                Opcode 0110111.
                The immediate 0x12345 is shifted right by 12 bits to get the upper 20 bits.
                The encoding places these 20 bits in the imm[31:12] field, followed by rd (x1) and opcode.
                Decoding extracts the upper 20 bits and shifts left by 12 to reconstruct the immediate.
                */
                // let valei32 = value.value();
                println!("Lui val {:#?}", value);
                // println!("VALE i32: {:032b}", valei32); // 18
                let value32: u32 = value.into();

                println!("Lui valu32 {:#?}", value32);
                // println!("VALE u32: {:032b}", vale); // 18

                // 0xFFFFF000
                // 0b11111111111111111111000000000000
                self.cpu.registers.set(dest, value32);
                // self.registers().set(dest);
                Ok(())
            }
            Instruction::LoadWord { dest, src, offset } => {
                println!("Load Word");
                let addr = u32::from(offset) + self.cpu.registers.get(src);
                self.cpu
                    .registers
                    .set(dest, self.memory.read(addr).unwrap());
                Ok(())
            }
            Instruction::StoreWord { dest, src, offset } => {
                let address = u32::from(offset) + self.cpu.registers.get(src);
                let value = self.cpu.registers.get(dest);
                println!("Store Word address: {address}");
                println!("Store Word value: {value}");
                self.memory.write(address, value).unwrap();
                println!("Store Word quit");
                Ok(())
            }
            Instruction::Shl { dest, src, shift } => {
                let base = self.cpu.registers.get(src);
                self.registers().set(dest, base << shift);
                Ok(())
            }
            Instruction::Shr { dest, src, shift } => {
                let base = self.cpu.registers.get(src);
                self.registers().set(dest, base >> shift);
                Ok(())
            }
            Instruction::ShrA { dest, src, shift } => {
                let base = self.cpu.registers.get(src) as i32;
                self.registers().set(dest, (base >> shift) as u32);
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
    use crate::cpu::register::Register;
    use Instruction::*;

    use super::*;
    use crate::instruction::{operand::Immediate, *};

    const CASES: [(i32, i32); 10] = [
        (1, 1),
        (2, 2),
        (12, -1),
        (-2, 4),
        (-32, -33),
        (111, -112),
        (-1000, 52),
        (-201, -97),
        (333, 333),
        (300, 20),
    ];

    #[test]
    fn t_arith() {
        let size = 1024 * 1024;

        let mut vm = VM::new(size);
        for (a, b) in CASES {
            let program = &[
                // Li {
                //     dest: Register::T2,
                //     value: Immediate::new(a),
                // },
                AddI {
                    dest: Register::T2,
                    src: Register::Zero,
                    value: Immediate::new::<14>(a),
                },
                AddI {
                    dest: Register::T3,
                    src: Register::T2,
                    value: Immediate::new::<14>(b),
                },
                Syscall {
                    src1: Register::Zero,
                    src2: Register::Zero,
                    src3: Register::Zero,
                },
            ];

            // TODO: fix this, and make generic immediate
            match vm.test_run(program) {
                Ok(e) => {}
                Err(e) => println!("Test run went wrong"),
            }
            println!("\n");
            assert_eq!(
                vm.cpu.registers.get(Register::T3),
                (a + b) as u32,
                "Variable: {a} and {b}"
            );
            vm.reset();
            println!("Move on to the next items: {a} & {b}");
        }
    }

    #[test]
    fn t_load_store() -> Result<(), DecodeError> {
        let size = 1024 * 1024;

        let mut vm = VM::new(size);

        let program = &[
            Lui {
                dest: Register::T3,
                value: Immediate::new::<19>(0x4000),
            },
            AddI {
                dest: Register::T3,
                src: Register::T3,
                value: Immediate::new::<14>(0x100),
            },
            AddI {
                dest: Register::T2,
                src: Register::Zero,
                value: Immediate::new::<14>(42),
            },
            StoreWord {
                dest: Register::T2,
                src: Register::T3,
                offset: Immediate::new::<14>(0),
            },
            LoadWord {
                dest: Register::T1,
                src: Register::T3,
                offset: Immediate::new::<14>(0),
            },
            Syscall {
                src1: Register::Zero,
                src2: Register::Zero,
                src3: Register::Zero,
            },
        ];

        match vm.test_run(program) {
            Ok(e) => {}
            Err(e) => println!("Test run went wrong"),
        }
        assert_eq!(
            vm.cpu.registers.get(Register::T1),
            vm.cpu.registers.get(Register::T2)
        );
        vm.reset();
        Ok(())
    }
}
