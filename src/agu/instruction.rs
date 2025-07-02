use log::warn;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    sequence::delimited,
};
use std::{fmt::Display, str::FromStr};
use strum_macros::{Display, EnumString};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn from_mnemonics(s: &str) -> IResult<&str, Self> {
        let (input, inst_type) = alt((tag("LOAD"), tag("STORE"))).parse(s)?;
        let (input, _) = delimited(multispace0, tag(","), multispace0).parse(input)?;
        let (input, inst_mode) = alt((tag("STRIDED"), tag("CONST"))).parse(input)?;
        let (input, _) = delimited(multispace0, tag(","), multispace0).parse(input)?;
        let (input, data_width) = alt((tag("B8"), tag("B16"), tag("B64"))).parse(input)?;
        let (input, _) = delimited(multispace0, tag(","), multispace0).parse(input)?;
        let (input, stride) = digit1.parse(input)?;
        let inst_mode: InstMode = inst_mode.parse().unwrap();
        let stride = stride.parse::<u8>().unwrap();
        if inst_mode == InstMode::CONST && stride != 0 {
            warn!(
                "Warning when loading AGU configuration: you are in CONST mode but you specified stride in the instruction"
            );
        }
        Ok((
            input,
            Self {
                inst_type: inst_type.parse().unwrap(),
                inst_mode,
                data_width: data_width.parse().unwrap(),
                stride,
            },
        ))
    }

    /// Convert the instruction to a binary string
    /// The field is in the order of inst_type, inst_mode, data_width, stride
    /// Refere to the corresponding enums for the value
    pub fn to_binary_str(&self) -> String {
        let mut bin = String::new();
        // converting inst_type to binary, then two one bit string
        assert!((self.inst_type as u8) < 2, "Inst type must be less than 2");
        let inst_type_bin = format!("{:b}", self.inst_type as u8);
        assert!((self.inst_mode as u8) < 2, "Inst mode must be less than 2");
        let inst_mode_bin = format!("{:b}", self.inst_mode as u8);
        assert!(
            (self.data_width as u8) < 4,
            "Data width must be less than 4"
        );
        let data_width_bin = format!("{:02b}", self.data_width as u8);
        assert!(self.stride < 16, "Stride must be less than 16");
        let stride_bin = format!("{:04b}", self.stride);
        bin.push_str(&inst_type_bin);
        bin.push_str(&inst_mode_bin);
        bin.push_str(&data_width_bin);
        bin.push_str(&stride_bin);
        assert!(bin.len() == 8, "Binary string must be 8 bits");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_parsing() {
        let inst = Instruction::from_str("LOAD,STRIDED,B8,1").unwrap();
        assert_eq!(inst.inst_type, InstType::LOAD);
        assert_eq!(inst.inst_mode, InstMode::STRIDED);
        assert_eq!(inst.data_width, DataWidth::B8);
        assert_eq!(inst.stride, 1);
    }

    #[test]
    fn test_instruction_to_binary_str() {
        let inst = Instruction::from_str("LOAD,STRIDED,B16,1").unwrap();
        assert_eq!(inst.to_binary_str(), "00010001");
    }
}
