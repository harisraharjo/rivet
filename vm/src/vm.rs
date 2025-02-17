use isa::instruction::Instruction;

use crate::{
    cpu::{register::Registers, CPU},
    memory::{MemoryConfiguration, MemoryManager},
};

pub struct VM {
    pub(crate) cpu: CPU,
    memory: MemoryManager,
    halt: bool,
}

impl VM {
    pub fn new(configuration: MemoryConfiguration) -> Self {
        Self {
            cpu: CPU::new(),
            memory: MemoryManager::new(&configuration),
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

    pub fn run(&mut self) -> anyhow::Result<()> {
        while !self.halt {
            self.step()?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        let instruction = self.fetch()?;
        self.cpu.pc.increment();
        self.decode_execute(instruction)
    }

    pub fn registers(&mut self) -> &mut Registers {
        &mut self.cpu.registers
    }

    // #[cfg(test)]
    // pub fn load_asm(&self, path: &std::path::Path) -> i32 {
    //     use std::fs::File;

    //     let file = File::open(path);
    //     let mut reader = BufReader::new(f);
    //     // use std::path::Path;

    //     1;
    // }

    #[cfg(test)]
    pub fn test_run(&mut self, program: &[Instruction]) -> anyhow::Result<()> {
        use isa::register::Register;

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
        self.cpu
            .registers
            .set(Register::SP, self.memory.stack_start());

        while !self.halt {
            self.step()?;
        }

        // while self.cpu.pc.value() < self.memory.size() && !self.halt {
        //     self.step()?;
        // }

        Ok(())
    }
}

impl VM {
    fn fetch(&self) -> anyhow::Result<Instruction> {
        let memory = self.memory.read::<u32>(self.cpu.pc.value())?;

        Ok(Instruction::try_from(memory)?)
    }

    // TODO: Should it be inlined bcs of hot loop? (https://nnethercote.github.io/perf-book/inlining.html)
    // #[inline(always)]
    fn decode_execute(&mut self, instruction: Instruction) -> anyhow::Result<()> {
        // println!("Decoded Instruction: {:?}", instruction);
        match instruction {
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
                println!("");
                println!("Addi");
                println!("Imm: {:?}", value);
                let v: u32 = value.into();
                println!("Imm_u32: {v}");
                println!("src reg: {:?}", src);
                let src = self.cpu.registers.get(src);
                println!("src val: {:?}", src);
                let result = src.wrapping_add(v);
                println!("src wrapping_add imm_u32: {result}");
                println!("Storing to dest reg: {:?}", dest);
                self.cpu.registers.set(dest, result);
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

                self.cpu.registers.set(dest, value.into());
                Ok(())
            }
            Instruction::Lw { src, dest, offset } => {
                let addr = u32::from(offset) + self.cpu.registers.get(src);
                self.memory
                    .alignment_check(std::mem::size_of::<u32>(), addr)?;
                self.cpu.registers.set(dest, self.memory.read(addr)?);
                Ok(())
            }
            Instruction::Sw { dest, src, offset } => {
                // Alignment check (RISC-V requires alignment for LW/SW/LH/SH)
                println!("");
                println!("Store word");

                println!("dest reg: {:?}", dest);
                let dest = self.cpu.registers.get(dest);
                println!("Dest value: {dest}");
                let offset = u32::from(offset);
                println!("Offset: {}", u32::from(offset));
                // TODO: dest and offset potential to be minus
                let address = offset + dest;
                println!("offset + dest: {address}");
                self.memory
                    .alignment_check(std::mem::size_of::<u32>(), address)?;
                let value = self.cpu.registers.get(src);
                println!("src reg: {:?}", src);
                println!("src value: {:?}", value);
                self.memory.write(address, value)?;
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
    use isa::{
        operand::{Immediate, Immediate14, Immediate19},
        register::Register,
    };
    use Instruction::*;

    use super::*;

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

        let mut vm = VM::new(crate::memory::MemoryConfiguration::new(size));
        for (a, b) in CASES {
            let program = &[
                AddI {
                    dest: Register::T2,
                    src: Register::Zero,
                    value: Immediate14::new(a),
                },
                AddI {
                    dest: Register::T3,
                    src: Register::T2,
                    value: Immediate14::new(b),
                },
                Syscall {
                    src1: Register::Zero,
                    src2: Register::Zero,
                    src3: Register::Zero,
                },
            ];

            match vm.test_run(program) {
                Ok(_) => {}
                Err(e) => println!("Test run went wrong {}", e),
            }
            println!("\n");
            assert_eq!(
                vm.cpu.registers.get(Register::T3),
                (a + b) as u32,
                "Variable: {a} and {b}"
            );
            vm.reset();
        }
    }

    #[test]
    fn t_load_store_on_the_stack() {
        let size = 1024 * 1024;

        let mut vm = VM::new(crate::memory::MemoryConfiguration::new(size));

        // # Function prologue - setup stack frame
        // function_start:
        //      addi sp, sp, -16
        //      sw ra, 12(sp)     # Save return address
        //      sw s0, 8(sp)      # Save frame pointer (if needed)
        // # Store a variable on the stack
        //      li t0, 42         # Load value 42 into t0
        //      sw t0, 0(sp)      # Store this value on the stack
        let program = &[
            AddI {
                dest: Register::RA,
                src: Register::Zero,
                value: Immediate14::new(5),
            },
            AddI {
                dest: Register::S0,
                src: Register::Zero,
                value: Immediate14::new(13),
            },
            AddI {
                dest: Register::SP,
                src: Register::SP,
                value: Immediate14::new(-15), //allocate 15 bytes/index
            },
            Sw {
                dest: Register::SP,
                src: Register::RA,
                offset: Immediate14::new(0), //store value at SP + 0
            },
            Sw {
                dest: Register::SP,
                src: Register::S0,
                offset: Immediate14::new(4), //store value at SP + 4
            },
            Lui {
                dest: Register::T0,
                value: Immediate19::new(43),
            },
            Sw {
                dest: Register::SP,
                src: Register::T0,
                offset: Immediate14::new(8), //store value at SP + 8
            },
            // Load data in address SP + 0 to T1. This is used for test
            Lw {
                dest: Register::T1,
                src: Register::SP,
                offset: Immediate14::new(0),
            },
            // Load data in address SP + 4 to T2. This is used for test
            Lw {
                dest: Register::T2,
                src: Register::SP,
                offset: Immediate14::new(4),
            },
            // Load data in address SP + 8 to T3. This is used for test
            Lw {
                dest: Register::T3,
                src: Register::SP,
                offset: Immediate14::new(8),
            },
            Syscall {
                src1: Register::Zero,
                src2: Register::Zero,
                src3: Register::Zero,
            },
        ];

        match vm.test_run(program) {
            Ok(_) => {}
            Err(e) => println!("Test run went wrong {}", e),
        }

        assert_eq!(vm.cpu.registers.get(Register::T1), 5);
        assert_eq!(vm.cpu.registers.get(Register::S0), 13);
        assert_eq!(vm.cpu.registers.get(Register::T0), 43);
        vm.reset();
    }
}
