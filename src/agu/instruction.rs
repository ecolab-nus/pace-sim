use std::{fmt::Display, str::FromStr};
use strum_macros::{Display, EnumString};

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub inst_type: InstType,
    pub inst_mode: InstMode,
    pub data_width: DataWidth,
    pub stride: u8, // can only be used as 4b integer
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, EnumString)]
pub enum InstType {
    LOAD = 0,
    STORE = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, EnumString)]
pub enum InstMode {
    STRIDED = 0,
    CONST = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, EnumString)]
pub enum DataWidth {
    B8 = 0,
    B16 = 1,
    B64 = 2,
}

impl Instruction {
    /// Loading from one byte, MSB of the value is the most significant bit
    pub fn from_binary(bin: u8) -> Self {
        // bit 0 is inst_type, 0 is LOAD, 1 is STORE
        // bit 1 is inst_mode, 0 is STRIDED, 1 is CONST
        // bit 2-3 is data_width, 00 is B8, 01 is B16, 10 is B64
        // bit 4-7 is the stride.
        let inst_type = if bin & 0b10000000 == 0 {
            InstType::LOAD
        } else {
            InstType::STORE
        };

        let inst_mode = if bin & 0b01000000 == 0 {
            InstMode::STRIDED
        } else {
            InstMode::CONST
        };

        let data_width = if bin & 0b00110000 == 0 {
            DataWidth::B8
        } else if bin & 0b00110000 == 0b01000000 {
            DataWidth::B16
        } else if bin & 0b00110000 == 0b10000000 {
            DataWidth::B64
        } else {
            panic!("Invalid data width");
        };

        let stride = bin & 0b00001111;

        Self {
            inst_type,
            inst_mode,
            data_width,
            stride,
        }
    }

    pub fn to_binary(&self) -> u8 {
        let mut bin = 0;
        bin |= self.inst_type as u8;
        bin |= self.inst_mode as u8;
        bin |= self.data_width as u8;
        bin |= self.stride as u8;
        bin
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.inst_type, self.inst_mode, self.data_width, self.stride
        )
    }
}

impl FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid instruction format: {}", s));
        }

        Ok(Self {
            inst_type: parts[0].parse().unwrap(),
            inst_mode: parts[1].parse().unwrap(),
            data_width: parts[2].parse().unwrap(),
            stride: parts[3].parse().unwrap(),
        })
    }
}
