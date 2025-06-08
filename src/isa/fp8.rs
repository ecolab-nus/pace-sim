use std::ops::{Add, Mul, Sub};

// Implementing the FP8 format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FP8(u8);

impl FP8 {
    const EXP_MASK: u8 = 0b01111000;
    const MANT_MASK: u8 = 0b00000111;
    const SIGN_MASK: u8 = 0b10000000;
    const EXP_BIAS: i32 = 15;
}

impl From<u8> for FP8 {
    // break down the value into sign, exponent, and mantissa
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Into<u8> for FP8 {
    fn into(self) -> u8 {
        self.0
    }
}

impl Into<f32> for FP8 {
    fn into(self) -> f32 {
        let bits = self.0;
        let sign = if (bits & Self::SIGN_MASK) != 0 {
            -1.0
        } else {
            1.0
        };
        let exp = (bits & Self::EXP_MASK) >> 2;
        let man = bits & Self::MANT_MASK;

        match exp {
            0 => sign * 0.0,
            0x1F => {
                if man == 0 {
                    sign * f32::INFINITY
                } else {
                    f32::NAN
                }
            }
            _ => {
                let e = (exp as i32) - Self::EXP_BIAS;
                let frac = 1.0 + (man as f32) / (1 << 2) as f32;
                sign * frac * 2f32.powi(e)
            }
        }
    }
}

impl From<f32> for FP8 {
    fn from(value: f32) -> Self {
        let bits = value.to_bits();
        let sign = ((bits >> 31) as u8) << 7;
        let exp32 = ((bits >> 23) & 0xFF) as i32;
        let man32 = bits & 0x7FFFFF;
        if value.is_nan() {
            return FP8(sign | Self::EXP_MASK | 0x01);
        }
        if value.is_infinite() {
            return FP8(sign | Self::EXP_MASK);
        }
        let exp_unb = exp32 - 127;
        let exp8 = exp_unb + Self::EXP_BIAS;
        if exp8 <= 0 {
            return FP8(sign);
        }
        if exp8 >= 0x1F {
            return FP8(sign | Self::EXP_MASK);
        }
        let mut exp_bits = exp8 as u8;
        let mant_shift = 23 - 2;
        let mant_hi = (man32 >> mant_shift) as u8;
        let rem = man32 & ((1 << mant_shift) - 1);
        let mut man_bits = mant_hi;
        let half = 1 << (mant_shift - 1);
        if rem > half || (rem == half && (man_bits & 1) != 0) {
            man_bits = man_bits.wrapping_add(1);
            if man_bits == (1 << 2) {
                man_bits = 0;
                exp_bits = exp_bits.wrapping_add(1);
                if exp_bits >= 0x1F {
                    return FP8(sign | Self::EXP_MASK);
                }
            }
        }
        FP8(sign | (exp_bits << 2) | (man_bits & Self::MANT_MASK))
    }
}

impl Add for FP8 {
    type Output = FP8;

    fn add(self, _other: FP8) -> FP8 {
        let a: f32 = self.into();
        let b: f32 = _other.into();
        FP8::from(a + b)
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
