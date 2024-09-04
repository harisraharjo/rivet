use std::ops::{BitAnd, Shr};

use macros::EnumCount;

pub mod instruction;

enum Testa {
    F1,
}

#[derive(Debug)]
pub enum Register {
    Zero,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    /// Stack Pointer
    SP,
    /// Base Pointer
    BP,
    /// Return Address
    RA,
}

impl Register {
    // pub fn mask<const N: u8>(&self, i: usize) -> i32 {
    //     let xx = match N {
    //         0 => ,
    //         1 =>
    //     };
    //     1
    // }
}

impl BitAnd<u32> for Register {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        self as u32 & rhs
    }
}

impl From<Register> for u32 {
    fn from(value: Register) -> Self {
        match value {
            Register::Zero => todo!(),
            Register::X1 => todo!(),
            Register::X2 => todo!(),
            Register::X3 => todo!(),
            Register::X4 => todo!(),
            Register::X5 => todo!(),
            Register::X6 => todo!(),
            Register::SP => todo!(),
            Register::BP => todo!(),
            Register::RA => todo!(),
            // Register::Test(a) => todo!(),
        }
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value {
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::BP,
            8 => Register::SP,
            9 => Register::RA,
            _ => Register::Zero,
        }
    }
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::BP,
            8 => Register::SP,
            9 => Register::RA,
            _ => Register::Zero,
        }
    }
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        match value {
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::BP,
            8 => Register::SP,
            9 => Register::RA,
            _ => Register::Zero,
        }
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    pub fn new() -> ProgramCounter {
        ProgramCounter(0)
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 4
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        self.0
    }
}
