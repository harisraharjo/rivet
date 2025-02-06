use crate::instruction::Codec;
use std::ops::{BitAnd, Shl};

pub enum ImmBit {
    B14,
    B19,
}

impl ImmBit {
    pub const fn count(&self) -> u32 {
        match self {
            ImmBit::B14 => 14,
            ImmBit::B19 => 19,
        }
    }
}

impl From<ImmBit> for u32 {
    fn from(value: ImmBit) -> Self {
        value.count()
    }
}

// toDO: create compile time check for bit length only until 32 inclusive
// TODO: Make generic immediate or make it u32 instead
#[derive(Debug, PartialEq, Eq)]
pub struct Immediate<const BIT: u32>(i32);

impl<const BIT: u32> Immediate<BIT> {
    // const BIT_LENGTH: u32 = if BIT_LENGTH <= 32 {
    //     BIT_LENGTH
    // } else {
    //     panic!("Invalid offset") // Compile time panic
    // };

    // const _A: () = assert!(BIT_LENGTH <= u32::BITS, "N must not exceed u32::BITS");

    pub fn new(value: i32) -> Self {
        if BIT == 0 || BIT >= (u32::BITS) {
            panic!("Max bit length is 32");
        };

        let max = (1 << (BIT - 1)) - 1; //i32
                                        // let max = (1 << BIT_LENGTH) - 1; //u32
        let min = -(max + 1);
        assert!(
            value >= min && value <= max,
            "the value does not fit into the type `Immediate"
        );

        Immediate(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    fn convert_twos_complement_to_i32(masked_value: u32, bit_mask: u32) -> Self {
        // Check if the sign bit is set

        let sign_bit = 1u32 << (bit_mask.trailing_ones() - 1);

        if masked_value & sign_bit != 0 {
            // Negative number: extend sign by converting to two's complement
            let positive_counterpart = (sign_bit << 1) - masked_value;

            Self(-(positive_counterpart as i32))
        } else {
            // Positive number or zero
            Self(masked_value as i32)
        }
    }
}

impl<const BIT: u32> Codec for Immediate<BIT> {
    fn decode(src: u32, bit_accumulation: u32, bit_mask: u32) -> Self
    where
        Self: From<u32>,
    {
        Self::convert_twos_complement_to_i32((src >> bit_accumulation) & bit_mask, bit_mask).into()
    }
}

impl<const BIT: u32> From<Immediate<BIT>> for i32 {
    fn from(value: Immediate<BIT>) -> Self {
        value.0
    }
}

impl<const BIT: u32> From<u32> for Immediate<BIT> {
    fn from(value: u32) -> Self {
        Immediate(value as i32)
    }
}

impl<const BIT: u32> From<Immediate<BIT>> for u32 {
    fn from(value: Immediate<BIT>) -> Self {
        // let g = value.0;
        // println!("Immi: {:032b}", g);
        // println!("Immi: {:032b}", g as u32);
        value.0 as u32
    }
}

impl<const BIT: u32> BitAnd<u32> for &Immediate<BIT> {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (self.0 as u32) & rhs
    }
}

impl<const BIT: u32> Shl<u32> for &Immediate<BIT> {
    type Output = u32;

    fn shl(self, rhs: u32) -> Self::Output {
        (self.0 as u32) << rhs
    }
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn t_imm() {
        let imm = Immediate::<{ ImmBit::B14.count() }>(-31);
        let result = imm.encode(0x3FFF, 18);
        assert_eq!(result, 0xFF840000);

        let result = Immediate::decode(4286958867, 18, 0x3FFF);
        assert_eq!(result, imm);
    }
}
