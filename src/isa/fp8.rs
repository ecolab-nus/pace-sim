use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};

// Implementing the FP8 format
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct FP8(u8);

impl Debug for FP8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let float_val: f32 = (*self).into();
        write!(f, "{:08b}({})", self.0, float_val)
    }
}

impl FP8 {
    const EXP_MASK: u8 = 0b01111100;
    const MANT_MASK: u8 = 0b00000011;
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

/// Value Formula
/// For an 8-bit pattern b7 b6…b0, let:
/// sign = (b7 == 1 ? –1 : +1)
/// E = (b6…b2) interpreted as an unsigned integer
/// M = (b1…b0) interpreted as an unsigned integer
/// Then:
/// Zero
/// If E == 0 and M == 0 → value = sign × 0.0.
/// Subnormal (optional)
/// If E == 0 and M != 0 →
/// value = sign × (M / 2²) × 2^(1–bias)
/// (we omitted subnormals in our impl, but this is the IEEE-style definition).
/// Normalized
/// If 1 <= E <= 30 →
/// value = sign × (1 + M / 2²) × 2^(E – bias)
/// Infinity / NaN
/// If E == 31 (all ones):
///     M == 0 → ±∞
///     M != 0 → NaN
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

impl Into<f64> for FP8 {
    fn into(self) -> f64 {
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
                    sign * f64::INFINITY
                } else {
                    f64::NAN
                }
            }
            _ => {
                let e = (exp as i32) - Self::EXP_BIAS;
                let frac = 1.0 + (man as f64) / (1 << 2) as f64;
                sign * frac * 2f64.powi(e)
            }
        }
    }
}

impl From<f64> for FP8 {
    fn from(val: f64) -> Self {
        let bits = val.to_bits();
        let sign = ((bits >> 63) as u8) << 7;
        let exp64 = ((bits >> 52) & 0x7FF) as i32;
        let man64 = bits & 0x000F_FFFF_FFFF_FFFF;
        if val.is_nan() {
            return FP8(sign | Self::EXP_MASK | 0x01);
        }
        if val.is_infinite() {
            return FP8(sign | Self::EXP_MASK);
        }
        let exp_unb = exp64 - 1023;
        let exp8 = exp_unb + Self::EXP_BIAS;
        if exp8 <= 0 {
            return FP8(sign);
        }
        if exp8 >= 0x1F {
            return FP8(sign | Self::EXP_MASK);
        }
        let mut exp_b = exp8 as u8;
        let shift = 52 - 2;
        let top = (man64 >> shift) as u8;
        let rem = man64 & ((1u64 << shift) - 1);
        let mut man_b = top;
        let half = 1u64 << (shift - 1);
        if rem > half || (rem == half && (man_b & 1) != 0) {
            man_b = man_b.wrapping_add(1);
            if man_b == (1 << 2) {
                man_b = 0;
                exp_b = exp_b.wrapping_add(1);
                if exp_b >= 0x1F {
                    return FP8(sign | Self::EXP_MASK);
                }
            }
        }
        FP8(sign | (exp_b << 2) | (man_b & Self::MANT_MASK))
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
        let a: f32 = self.into();
        let b: f32 = _other.into();
        FP8::from(a * b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_raw() {
        for bits in 0u8..=255u8 {
            let f8 = FP8::from(bits);
            let back_u8: u8 = f8.into();
            assert_eq!(back_u8, bits);
        }
    }

    #[test]
    fn test_add_binary() {
        // 1.0 -> 0b00111100 (0x3C), 2.0 -> 0b01000000 (0x40)
        // 3.0 -> 0b01000010 (0x42)
        let a = FP8::from(0x3C);
        let b = FP8::from(0x40);
        let sum: u8 = (a + b).into();
        assert_eq!(sum, 0x42);
    }

    #[test]
    fn test_mul_binary() {
        // 1.0 -> 0b00111100 (0x3C), 2.0 -> 0b01000000 (0x40)
        let a = FP8::from(0b00111100);
        let b = FP8::from(0b01000000);
        let prod: u8 = (a * b).into();
        assert_eq!(prod, 0b01000000); // expected 2.0 -> 0b01000000
    }

    #[test]
    fn test_vector_mac() {
        let a_bits = [0x3C, 0x38, 0x3E, 0x3E, 0xBC, 0x3C, 0x3A, 0x44];
        let b_bits = [0x40, 0x34, 0x3E, 0x38, 0x40, 0xB8, 0x3A, 0x38];
        let c_bits = [0x3C, 0x40, 0x3C, 0x40, 0x3C, 0x40, 0x38, 0x40];
        let expected = [0x42, 0x3C, 0x42, 0x41, 0x3C, 0x00, 0x3C, 0x45];
        for i in 0..8 {
            let a = FP8::from(a_bits[i]);
            let fp32_a: f32 = a.into();
            print!("a: {:?} ", fp32_a);
            let b = FP8::from(b_bits[i]);
            let fp32_b: f32 = b.into();
            print!("b: {:?} ", fp32_b);
            let c = FP8::from(c_bits[i]);
            let fp32_c: f32 = c.into();
            println!("c: {:?}", fp32_c);
            let res: u8 = (a + b * c).into();
            assert_eq!(res, expected[i], "idx {}", i);
        }
    }
}
