use crate::instruction::Codec;
use std::ops::{BitAnd, Shl};

pub(crate) enum ImmediateType {
    B14,
    B19,
}

impl ImmediateType {
    pub const fn length(&self) -> u32 {
        match self {
            ImmediateType::B14 => 14,
            ImmediateType::B19 => 19,
        }
    }
}

impl From<ImmediateType> for u32 {
    fn from(value: ImmediateType) -> Self {
        value.length()
    }
}

// TODO: Make generic immediate or make it u32 instead
#[derive(Debug, PartialEq, Eq)]
pub struct Immediate<const BIT: u32>(i32);
pub type Immediate14 = Immediate<14>;
pub type Immediate19 = Immediate<19>;

impl<const BIT: u32> Immediate<BIT> {
    const _BIT: () = if BIT == 14 || BIT == 19 {
        ()
    } else {
        panic!("Immediate only support 14 & 19 Bit");
    };

    pub fn new(value: i32) -> Self {
        let _ = Self::_BIT;

        let max = (1 << (BIT - 1)) - 1; //i32
                                        // let max = (1 << BIT) - 1; //u32
        let min = -(max + 1);
        assert!(
            value >= min && value <= max,
            "the value does not fit into `{}` bit",
            BIT
        );

        Immediate(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    fn convert_twos_complement_to_i32(masked_value: u32, bit_mask: u32) -> i32 {
        // Check if the sign bit is set
        let sign_bit = 1u32 << (bit_mask.trailing_ones() - 1);

        if masked_value & sign_bit != 0 {
            // Negative number: extend sign by converting to two's complement
            let positive_counterpart = (sign_bit << 1) - masked_value;

            -(positive_counterpart as i32)
        } else {
            // Positive number or zero
            masked_value as i32
        }
    }
}

impl Immediate<{ ImmediateType::B14.length() }> {}
impl Immediate<{ ImmediateType::B19.length() }> {}

impl<const BIT: u32> Codec for Immediate<BIT> {
    fn decode(src: u32, bit_accumulation: u32, bit_mask: u32) -> Self
    where
        Self: From<u32>,
    {
        Self::new(Self::convert_twos_complement_to_i32(
            (src >> bit_accumulation) & bit_mask,
            bit_mask,
        ))
        .into()
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

    // TODO: add test for overflow
    // Immediate14::new(-0x1FFF),
    //  assert_eq!(
    //         vm.registers().get(Register::X10),
    //         Immediate14::new(-0x1FFF).into(),
    //         "Addi error"
    //     );

    //     assert_eq!(
    //         vm.registers().get(Register::X5),
    //         Immediate19::new(0x3FFFF).into(),
    //         "lui error"
    //     );

    #[test]
    fn t_imm() {
        let imm = Immediate14::new(-31);
        let result = imm.encode(0x3FFF, 18);
        assert_eq!(result, 0xFF840000);

        let result = Immediate::decode(4286958867, 18, 0x3FFF);
        assert_eq!(result, imm);
    }

    // #[test]
    // #[should_panic]
    // fn t_imm_panic() {
    //     let result = std::panic::catch_unwind(|| Immediate::<32>::new(-0x1FFF));
    //     assert!(result.is_err());
    // }
}
