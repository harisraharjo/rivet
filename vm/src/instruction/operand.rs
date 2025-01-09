use std::ops::{Add, BitAnd};

// pub trait Operand {}

// use bitvec::{array::BitArray, order::Lsb0};
// #[derive(Debug)]
/// 24 bits Operand
// struct Op24(BitArray<[u8; 3], Lsb0>);
// impl Op24 {
//     pub fn new() -> Op24 {
//         Op24(std::default::Default::default())
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub struct Imm16(pub(super) u16);

impl From<u32> for Imm16 {
    fn from(value: u32) -> Self {
        Self(value as u16)
    }
}

impl BitAnd<u32> for &Imm16 {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (self.0 as u32) & rhs
    }
}

impl Add<u32> for Imm16 {
    type Output = u32;

    fn add(self, rhs: u32) -> Self::Output {
        (self.0 as u32) + rhs
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Imm8(pub(super) u8);

impl From<u32> for Imm8 {
    fn from(value: u32) -> Self {
        Self(value as u8)
    }
}

impl BitAnd<u32> for &Imm8 {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (self.0 as u32) & rhs
    }
}

impl Add<u32> for Imm8 {
    type Output = u32;

    fn add(self, rhs: u32) -> Self::Output {
        (self.0 as u32) + rhs
    }
}

pub struct Address;
pub struct LabelRef;

#[derive(Debug)]
pub struct Op7;
#[derive(Debug)]
pub struct Op5;
#[derive(Debug, PartialEq, Eq)]
pub struct Literal12;
