use crate::instruction::Codec;
use shared::{EnumCount, EnumVariants};
use std::ops::{BitAnd, Shl, Shr};

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumCount, EnumVariants)]
pub enum Register {
    /// Zero
    Zero,
    /// Return Address
    RA,
    /// Stack Pointer
    SP,
    /// Global Pointer
    GP,
    /// Thread Pointer
    TP,
    // === temporary values ===
    /// Alternate Return Address
    T0,
    T1,
    T2,
    T3,
    // === saved values ===
    /// Frame Pointer
    S0,
    S1,
    S2,
    S3,
    // === Argument register (function args & return values) ===
    ///Return Value
    A0,
    A1,
    A2,
    A3,
    /// Syscall
    A7,
}

// Registers (x0-x31 and ABI names)
// #[regex(r"x([0-9]|[1-2][0-9]|3[0-1])")]
// #[regex(r"(zero|ra|sp|gp|tp|t[0-6]|s[0-1][0-1]|a[0-7]|ft[0-9]|ft1[0-1]|fs[0-1][0-1])")]
// Register,

impl Register {
    pub fn fp() -> Register {
        Register::S0
    }
}

impl Codec for Register {}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        // if value  > (Self::VARIANT_COUNT as u32) {
        //     Register::Zero
        // } else {
        //     value.into()
        // }

        // Check if value is within bounds (1 to 31)
        // bool casted to u8 is either 0 or 1. Yes = 1, No = 0;
        let is_in_range = ((value > 0) & (value <= (Self::VARIANT_COUNT - 1) as u32)) as u8;

        // Clamp the value to 0 if it's out of range
        // By multiplying value by is_in_range when is_in_range is 1, we get value itself.
        // When is_in_range is 0, multiplying by 0 results in 0,
        //      and we add Register::Zero (which is 0 in u8 representation) back in, effectively clamping all out-of-range values to 0.
        let clamped_value = (is_in_range * value as u8) | ((!is_in_range) * Register::Zero as u8);

        // Safety: clamped_value is ensured to be within a safe range for Register (Self::VARIANT_COUNT).
        unsafe { std::mem::transmute::<u8, Register>(clamped_value) }
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
mod test {
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
