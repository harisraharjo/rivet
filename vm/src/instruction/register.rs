use std::ops::BitAnd;

use macros::EnumCount;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumCount)]
#[repr(u8)]
pub enum Register {
    Zero,
    X1,  //RA
    X2,  //SP
    X3,  //GP
    X4,  //TP
    X5,  //t0 => temporary/alternate return address
    X6,  // t1
    X7,  // t2
    X8,  //s0 => Saved register / frame pointer
    X9,  // s1,
    X10, //a0 => function argument / return value
    X11, //a1
    X12, //a2
    X13, //a3
    X14, //t3
    X15, //t4
         // /// Base Pointer
         // BP,
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
            Register::X1 => todo!(),
            Register::X2 => todo!(),
            Register::X3 => todo!(),
            Register::X4 => todo!(),
            Register::X5 => todo!(),
            Register::X6 => todo!(),
            Register::X7 => todo!(),
            Register::X8 => todo!(),
            Register::X9 => todo!(),
            Register::X10 => todo!(),
            Register::X11 => todo!(),
            Register::X12 => todo!(),
            Register::X13 => todo!(),
            Register::X14 => todo!(),
            Register::X15 => todo!(),
            // Register::Test(a) => todo!(),
        }
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value {
            0 => Register::Zero,
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::X7,
            8 => Register::X8,
            9 => Register::X9,
            10 => Register::X10,
            11 => Register::X11,
            12 => Register::X12,
            13 => Register::X13,
            14 => Register::X14,
            _ => Register::X15,
        }
    }
}

// impl From<u8> for Register {
//     fn from(value: u8) -> Self {
//         match value {
//             1 => Register::X1,
//             2 => Register::X2,
//             3 => Register::X3,
//             4 => Register::X4,
//             5 => Register::X5,
//             6 => Register::X6,
//             _ => Register::Zero,
//         }
//     }
// }

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
