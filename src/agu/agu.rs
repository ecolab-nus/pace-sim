use std::fmt::Display;

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    multi::separated_list0,
};

use crate::{agu::instruction::DataWidth, sim::dmem::DMemInterface};

use super::instruction::{InstMode, Instruction};

/// AGU state
#[derive(Debug, Clone)]
pub struct AGU {
    pub pc: u32,
    pub cm: Vec<Instruction>,
    pub arf: Vec<u16>,
    pub max_count: u32,
    pub count: u32,
}

impl AGU {
    pub fn is_enabled(&self) -> bool {
        self.max_count > 0
    }

    pub fn update(&mut self, dmem: &mut DMemInterface) {
        assert!(
            self.is_enabled(),
            "AGU is not enabled, you should not call this function"
        );
        let inst = &self.cm[self.pc as usize];
        let pc = self.pc as usize;
        let addr = self.arf[pc];

        match inst.inst_mode {
            InstMode::STRIDED => match inst.data_width {
                DataWidth::B8 => {
                    self.arf[pc] = addr + inst.stride as u16;
                }
                DataWidth::B16 => {
                    self.arf[pc] = addr + inst.stride as u16 * 2;
                }
                DataWidth::B64 => {
                    self.arf[pc] = addr + inst.stride as u16 * 8;
                }
            },
            InstMode::CONST => {
                // do nothing
            }
        }
        dmem.wire_dmem_addr = Some(addr as u64);
    }

    pub fn next(&mut self) -> Result<(), String> {
        if self.count >= self.max_count {
            return Err("AGU execution completed".to_string());
        }
        if self.pc == self.cm.len() as u32 - 1 {
            self.pc = 0;
            self.count += 1;
        } else {
            self.pc += 1;
        }
        Ok(())
    }
}

impl Display for AGU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PC: {}\n", self.pc)?;
        write!(f, "CM: {:?}\n", self.cm)?;
        write!(f, "ARF: {:?}\n", self.arf)?;
        write!(f, "MAX COUNT: {}\n", self.max_count)?;
        write!(f, "COUNT: {}\n", self.count)?;
        Ok(())
    }
}

impl AGU {
    fn parse_mnemonics(s: &str) -> IResult<&str, Self> {
        let (input, _) = multispace0(s)?;
        let (input, _) = tag("CM:").parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, cm) = separated_list0(multispace0, Instruction::from_mnemonics).parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, _) = tag("ARF:").parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, arf) = separated_list0(multispace0, digit1).parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, _) = tag("MAX COUNT:").parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, max_count) = digit1.parse(input)?;
        let max_count = max_count.parse::<u32>().unwrap();
        let arf: Vec<u16> = arf.iter().map(|s| s.parse::<u16>().unwrap()).collect();
        let (input, _) = multispace0.parse(input)?;
        assert!(
            arf.len() == cm.len(),
            "ARF and CM must have the same length"
        );
        if arf.len() == 0 {
            assert!(max_count == 0, "max count must be 0 if the AGU is not used");
        } else {
            assert!(cm.len() > 0, "CM must have at least one instruction");
            assert!(arf.len() > 0, "ARF must have at least one address");
            assert!(max_count > 0, "max count must be greater than 0");
        }
        Ok((
            input,
            Self {
                pc: 0,
                cm,
                arf,
                max_count,
                count: 0,
            },
        ))
    }

    pub fn from_mnemonics(s: &str) -> Result<Self, String> {
        let (input, state) = Self::parse_mnemonics(s).map_err(|e| e.to_string())?;
        assert!(input.is_empty());
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_from_mnemonics() {
        let s = r"CM:
            LOAD,STRIDED,B16,1
            LOAD,CONST,B64, 0
            ARF: 
            0
            10
            MAX COUNT:
            5
            ";

        let state = AGU::from_mnemonics(s).unwrap();
        assert_eq!(
            state.cm,
            vec![
                Instruction::from_str("LOAD,STRIDED,B16,1").unwrap(),
                Instruction::from_str("LOAD,CONST,B64,0").unwrap(),
            ]
        );
        assert_eq!(state.arf, vec![0, 10]);
        assert_eq!(state.max_count, 5);

        // test AGU disabled
        let s = r"CM:

            ARF: 

            MAX COUNT:
            0
            ";
        let state = AGU::from_mnemonics(s).unwrap();
        assert!(!state.is_enabled());
    }
}
