use std::ops::BitAnd;

pub trait Operand {}

#[derive(Debug, PartialEq, Eq)]
pub struct Imm16(pub(super) u16);

impl From<u32> for Imm16 {
    fn from(value: u32) -> Self {
        println!("Input {value}");
        println!("Output {}", value as u16);
        Self(value as u16)
    }
}

impl BitAnd<u32> for &Imm16 {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        (self.0 as u32) & rhs
    }
}

pub struct Address;
pub struct LabelRef;

#[derive(Debug)]
pub struct Literal7;
#[derive(Debug)]
pub struct Literal24;
#[derive(Debug, PartialEq, Eq)]
pub struct Literal12;
