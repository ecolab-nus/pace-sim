use crate::isa::fp8::FP8;

pub struct SIMDValue(pub [FP8; 8]);

impl From<u64> for SIMDValue {
    fn from(value: u64) -> Self {
        SIMDValue(value.to_le_bytes().map(|b| FP8::from(b)))
    }
}

impl From<SIMDValue> for u64 {
    fn from(_value: SIMDValue) -> Self {
        todo!();
    }
}

impl From<[FP8; 8]> for SIMDValue {
    fn from(value: [FP8; 8]) -> Self {
        SIMDValue(value)
    }
}

impl From<SIMDValue> for [FP8; 8] {
    fn from(value: SIMDValue) -> Self {
        value.0
    }
}

impl SIMDValue {
    pub fn vadd(&self, other: &SIMDValue) -> SIMDValue {
        let mut result = [FP8::default(); 8];
        for i in 0..8 {
            result[i] = self.0[i] + other.0[i];
        }
        SIMDValue(result)
    }

    pub fn vmul(&self, other: &SIMDValue) -> SIMDValue {
        let mut result = [FP8::default(); 8];
        for i in 0..8 {
            result[i] = self.0[i] * other.0[i];
        }
        SIMDValue(result)
    }
}

pub struct ScalarValue(pub i16);

impl From<u64> for ScalarValue {
    fn from(value: u64) -> Self {
        // keep the least significant 16 bits
        ScalarValue(value as i16)
    }
}
impl From<ScalarValue> for u64 {
    fn from(value: ScalarValue) -> Self {
        u64::from(value.0 as u16)
    }
}
impl From<ScalarValue> for i16 {
    fn from(value: ScalarValue) -> Self {
        value.0
    }
}

impl From<i16> for ScalarValue {
    fn from(value: i16) -> Self {
        ScalarValue(value)
    }
}
