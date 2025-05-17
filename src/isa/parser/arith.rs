use crate::isa::{instruction::Instruction, registers::parse_register};
use nom::{
    IResult, Parser, bytes::complete::tag, character::complete::space0, sequence::delimited,
};

/// Top parser for all arithmetic and logic instructions
pub fn top(s: &str) -> IResult<&str, Instruction> {
    parse_al_op(s)
}

pub fn parse_al_op(s: &str) -> IResult<&str, Instruction> {
    nom::branch::alt((parse_add, parse_sub)).parse(s)
}

pub fn parse_add(s: &str) -> IResult<&str, Instruction> {
    let (s, _) = tag("ADD")(s)?;
    let (s, _) = space0(s)?;
    let (s, a) = parse_register(s)?;
    let (s, _) = delimited(space0, tag(","), space0).parse(s)?;
    let (s, b) = parse_register(s)?;
    let (s, _) = delimited(space0, tag(","), space0).parse(s)?;
    let (s, c) = parse_register(s)?;
    Ok((s, Instruction::Add(a, b, c)))
}

pub fn parse_sub(s: &str) -> IResult<&str, Instruction> {
    let (s, _) = tag("SUB")(s)?;
    let (s, _) = space0(s)?;
    let (s, a) = parse_register(s)?;
    let (s, _) = delimited(space0, tag(","), space0).parse(s)?;
    let (s, b) = parse_register(s)?;
    let (s, _) = delimited(space0, tag(","), space0).parse(s)?;
    let (s, c) = parse_register(s)?;
    Ok((s, Instruction::Sub(a, b, c)))
}
