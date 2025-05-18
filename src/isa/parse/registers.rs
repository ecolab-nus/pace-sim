//! The register is the unified interfa
use nom::{IResult, character::complete::alphanumeric1};

pub fn parse_srcs(s: &str) -> IResult<&str, Register> {
    let (s, reg_name) = alphanumeric1(s)?;
    let index = SRCS.iter().position(|&name| name == reg_name);
    match index {
        Some(index) => Ok((s, index)),
        None => Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

pub fn parse_dsts(s: &str) -> IResult<&str, Register> {
    let (s, reg_name) = alphanumeric1(s)?;
    let index = DSTS.iter().position(|&name| name == reg_name);
    match index {
        Some(index) => Ok((s, index)),
        None => Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

pub fn reg_name_to_index(s: &str) -> Result<Register, String> {
    let index = REGS.iter().position(|&name| name == s);
    match index {
        Some(index) => Ok(index),
        None => Err(format!("Invalid register name: {}", s)),
    }
}
