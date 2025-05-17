use nom::{IResult, character::complete::alphanumeric1};

pub type Register = usize;
pub fn parse_register(s: &str) -> IResult<&str, Register> {
    let (s, reg_name) = alphanumeric1(s)?;
    // check the reg_name in the list of register names
    let index = reg_name_to_index(reg_name);
    match index {
        Ok(index) => Ok((s, index)),
        Err(e) => panic!("Invalid register name: {}", e),
    }
}

pub fn reg_name_to_index(s: &str) -> Result<Register, String> {
    let index = REGISTER_NAMES.iter().position(|&name| name == s);
    match index {
        Some(index) => Ok(index),
        None => Err(format!("Invalid register name: {}", s)),
    }
}

// R15-18 output NoC registers NESW
const REGISTER_NAMES: [&str; 23] = [
    "R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8", "R9", "R10", "R11", "R12", "R13", "R14",
    "RNOut", "REOut", "RSOut", "RWOut", "RNIn", "REIn", "RSIn", "RWIn",
];
