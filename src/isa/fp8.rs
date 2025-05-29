use std::ops::{Add, Mul, Sub};

// Implementing the FP8 format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct fp8 {
    pub sign: bool,
    pub exponent: u8, // using u8 but actual value is only 4 bits
    pub mantissa: u8, // using u8 but actual value is only 4 bits
}

impl From<u8> for fp8 {
    // break down the value into sign, exponent, and mantissa
    fn from(value: u8) -> Self {
        let sign = value & 0b10000000 != 0;
        let exponent = (value & 0b01111000) >> 3;
        let mantissa = value & 0b00000111;
        fp8 {
            sign,
            exponent,
            mantissa,
        }
    }
}

impl Add for fp8 {
    type Output = fp8;

    fn add(self, other: fp8) -> fp8 {
        // TODO: Implement addition
        todo!();
        self
    }
}

impl Sub for fp8 {
    type Output = fp8;

    fn sub(self, other: fp8) -> fp8 {
        // TODO: Implement subtraction
        todo!();
        self
    }
}

impl Mul for fp8 {
    type Output = fp8;

    fn mul(self, other: fp8) -> fp8 {
        // TODO: Implement multiplication
        todo!();
        self
    }
}
