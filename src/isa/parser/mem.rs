use crate::isa::{instruction::Instruction, registers::parse_register};
use nom::{
    IResult, Parser, bytes::complete::tag, character::complete::space0, sequence::delimited,
};

/// Top parser for all arithmetic and logic instructions
pub fn top(_: &str) -> IResult<&str, Instruction> {
    todo!()
}

pub fn parse_load(s: &str) -> IResult<&str, Instruction> {
    let (s, _) = tag("LOAD")(s)?;
    let (s, _) = space0(s)?;
    let (s, a) = parse_register(s)?;
    let (s, _) = delimited(space0, tag(","), space0).parse(s)?;
    let (s, b) = parse_register(s)?;
    Ok((s, Instruction::LOAD(a, b)))
}
