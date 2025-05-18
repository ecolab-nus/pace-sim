use nom::{IResult, bytes::complete::tag};

use crate::isa::instruction::{Instruction, InstructionParser};

// TODO
pub struct Add {}
impl InstructionParser for Add {
    fn parse(s: &str) -> IResult<&str, Instruction> {
        todo!()
    }
}
