use std::ops::{BitAnd, Shl, Shr};

use macros::EnumCount;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumCount)]
#[repr(u8)]
pub enum Register {
    Zero,
    RA,
    SP,
    GP,
    TP,
    /// temporary values
    T0, // alternate return address
    T1,
    T2,
    T3,
    /// saved values
    S0, //frame pointer
    S1,
    S2,
    S3,
    /// Argument register (function args & return values)
    A0, //return value
    A1,
    A2,
    A3,
    A7, //syscall
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            1 => Register::RA,
            2 => Register::SP,
            3 => Register::GP,
            4 => Register::TP,
            5 => Register::T0,
            6 => Register::T1,
            7 => Register::T2,
            8 => Register::T3,
            9 => Register::S0,
            10 => Register::S1,
            11 => Register::S2,
            12 => Register::S3,
            13 => Register::A0,
            14 => Register::A1,
            15 => Register::A2,
            16 => Register::A3,
            17 => Register::A7,
            _ => Register::Zero,
        }
    }
}

impl From<Register> for u32 {
    fn from(value: Register) -> Self {
        match value {
            Register::Zero => todo!(),
            Register::RA => todo!(),
            Register::SP => todo!(),
            Register::GP => todo!(),
            Register::TP => todo!(),
            Register::T0 => todo!(),
            Register::T1 => todo!(),
            Register::T2 => todo!(),
            Register::T3 => todo!(),
            Register::S0 => todo!(),
            Register::S1 => todo!(),
            Register::S2 => todo!(),
            Register::S3 => todo!(),
            Register::A0 => todo!(),
            Register::A1 => todo!(),
            Register::A2 => todo!(),
            Register::A3 => todo!(),
            Register::A7 => todo!(),
        }
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value {
            1 => Register::RA,
            2 => Register::SP,
            3 => Register::GP,
            4 => Register::TP,
            5 => Register::T0,
            6 => Register::T1,
            7 => Register::T2,
            8 => Register::T3,
            9 => Register::S0,
            10 => Register::S1,
            11 => Register::S2,
            12 => Register::S3,
            13 => Register::A0,
            14 => Register::A1,
            15 => Register::A2,
            16 => Register::A3,
            17 => Register::A7,
            _ => Register::Zero,
        }
    }
}

impl BitAnd<u32> for &Register {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (*self as u32) & rhs
    }
}

impl Shl<u32> for &Register {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (*self as u32) << rhs
    }
}

impl Shr<u32> for &Register {
    type Output = u32;

    fn shr(self, rhs: u32) -> Self::Output {
        (*self as u32) >> rhs
    }
}

impl Shl<Register> for u32 {
    type Output = Self;

    fn shl(self, rhs: Register) -> Self::Output {
        self << (rhs as u32)
    }
}

impl Shr<Register> for u32 {
    type Output = Self;

    fn shr(self, rhs: Register) -> Self::Output {
        self >> (rhs as u32)
    }
}

impl Shr<Register> for i32 {
    type Output = Self;

    fn shr(self, rhs: Register) -> Self::Output {
        self >> (rhs as i32)
    }
}

#[derive(Default, Debug)]
pub struct ProgramCounter(u32);

impl ProgramCounter {
    pub fn new() -> ProgramCounter {
        ProgramCounter(0)
    }

    #[inline(always)]
    pub fn increment(&mut self) {
        self.0 += 4
    }

    #[inline(always)]
    pub fn value(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}
