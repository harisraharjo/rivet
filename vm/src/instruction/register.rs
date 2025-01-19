use std::ops::{BitAnd, Shl, Shr};

use macros::EnumCount;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumCount)]
#[repr(u8)]
pub enum Register {
    Zero,
    RA, //RA
    SP, //SP
    GP, //GP
    TP, //TP
    T0, //t0 => temporary/alternate return address
    T1, // t1
    T2, // t2
    T3, //t3
    S0, //s0 => Saved register / frame pointer
    S1, // s1
    S2, // s2
    S3, // s3
    A0, //a0 => function argument / return value
    A1, //a1
    A2, //a2
    A3, //a3
    A7, //syscall
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

impl BitAnd<u32> for &Register {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (*self as u32) & rhs
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

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}
