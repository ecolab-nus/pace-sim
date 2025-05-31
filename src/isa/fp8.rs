use std::ops::{Add, Mul, Sub};

// Implementing the FP8 format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FP8 {
    pub sign: bool,
    pub exponent: u8, // using u8 but actual value is only 4 bits
    pub mantissa: u8, // using u8 but actual value is only 4 bits
}

impl From<u8> for FP8 {
    // break down the value into sign, exponent, and mantissa
    fn from(value: u8) -> Self {
        let sign = value & 0b10000000 != 0;
        let exponent = (value & 0b01111000) >> 3;
        let mantissa = value & 0b00000111;
        FP8 {
            sign,
            exponent,
            mantissa,
        }
    }
}

impl Add for FP8 {
    type Output = FP8;

    fn add(self, _other: FP8) -> FP8 {
        // TODO: Implement addition
        todo!();
    }
}

impl Sub for FP8 {
    type Output = FP8;

    fn sub(self, _other: FP8) -> FP8 {
        // TODO: Implement subtraction
        todo!();
    }
}

impl Mul for FP8 {
    type Output = FP8;

    fn mul(self, _other: FP8) -> FP8 {
        // TODO: Implement multiplication
        todo!();
    }
}
