use std::ops::{BitAnd, Shl, Shr};

use macros::EnumCount;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumCount)]
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

// pub trait Decode
// where
//     Self: From<u32>,
// {
//     fn decode(src: u32, bit_accumulation: u32, bit_length: u32) -> Self {
//         ((src >> bit_accumulation) & bit_length).into()
//     }
// }
// pub trait Encode
// where
//     for<'a> &'a Self: BitAnd<u32, Output = u32> + Shl<u32, Output = u32>,
// {
//     fn encode(&self, bit_length: u32, bit_accumulation: u32) -> u32 {
//         (self & bit_length) << bit_accumulation
//     }
// }
use crate::instruction::Codec;

impl Codec for Register {}

impl From<Register> for u32 {
    fn from(value: Register) -> Self {
        value as u32
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        // if value  > (Self::VARIANT_COUNT as u32) {
        //     Register::Zero
        // } else {
        //     value.into()
        // }

        // TODO: Try the branchless
        //branchless
        // Compute the condition as a bit mask
        // let in_range = (value <= Self::VARIANT_COUNT as u32) as u32;
        // // Select between value and Register::Zero based on the condition
        // let result_value = (in_range * value) | (!in_range & Register::Zero as u32);
        // Register::from(result_value)
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

impl Shl<Register> for u32 {
    type Output = Self;

    fn shl(self, rhs: Register) -> Self::Output {
        self << (rhs as u32)
    }
}

impl Shr<u32> for &Register {
    type Output = u32;

    fn shr(self, rhs: u32) -> Self::Output {
        (*self as u32) >> rhs
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

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn t_register() {
        let dest = Register::A0;
        let result = dest.encode(0x1F, 8u32);
        assert_eq!(result, 0xD00);

        let result = Register::decode(4286958867, 8u32, 0x1F);
        assert_eq!(result, dest);
    }
}
