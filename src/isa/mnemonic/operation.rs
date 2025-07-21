use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, space0},
    combinator::opt,
};
use std::str::FromStr;

use crate::isa::operation::{OpCode, Operation};

fn parse_nop(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("NOP")(input)?;
    Ok((
        input,
        Operation {
            op_code: OpCode::NOP,
            immediate: None,
            update_res: false,
            loop_start: None,
            loop_end: None,
        },
    ))
}

fn parse_immediate(input: &str) -> IResult<&str, u16> {
    let (input, _) = multispace0(input)?;
    match digit1::<_, nom::error::Error<&str>>(input) {
        Ok((input, immediate)) => Ok((input, immediate.parse::<u16>().unwrap())),
        Err(e) => Err(e),
    }
}

/// ALU operation is in the format of "OPCODE [!] IMM"
fn parse_alu_operation(input: &str) -> IResult<&str, Operation> {
    let (input, op_code_str) = alpha1(input)?;
    let op_code = OpCode::from_str(op_code_str).unwrap();
    let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    let update_res = update_res == "!";
    let (input, _) = space0(input)?;
    let r = parse_immediate(input);
    if r.is_ok() {
        let (input, immediate) = r.unwrap();
        Ok((
            input,
            Operation {
                op_code,
                immediate: Some(immediate),
                update_res,
                loop_start: None,
                loop_end: None,
            },
        ))
    } else {
        Ok((
            input,
            Operation {
                op_code,
                immediate: None,
                update_res,
                loop_start: None,
                loop_end: None,
            },
        ))
    }
}

/// Jump [#loop_start, #loop_end]
fn parse_jump(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("JUMP")(input)?;
    let (input, _) = space0(input)?;
    // parse the optional destination, the dst is an immediate value
    // dst is optional.
    let (input, dst) = opt(parse_immediate).parse(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("[")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, loop_start) = digit1::<_, nom::error::Error<&str>>(input)?;
    let loop_start = loop_start.parse::<u8>().unwrap();
    // assert it is within u4
    assert!(loop_start < 16, "Loop start must be within 4 bits");
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, loop_end) = digit1::<_, nom::error::Error<&str>>(input)?;
    let loop_end = loop_end.parse::<u8>().unwrap();
    // assert it is within u4
    assert!(loop_end < 16, "Loop end must be within 4 bits");
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("]")(input)?;

    // if dst is not set, use the loop start as jump dst
    let dst = if dst.is_none() {
        Some(loop_start as u16)
    } else {
        dst
    };
    assert!(dst.is_some(), "Jump destination must be set");
    assert!(dst.unwrap() < 16, "Jump destination out of bounds");
    Ok((
        input,
        Operation {
            op_code: OpCode::JUMP,
            immediate: dst,
            update_res: false,
            loop_start: Some(loop_start),
            loop_end: Some(loop_end),
        },
    ))
}

/// Memory operation format is "OPCODE IMM"
fn parse_memory(input: &str) -> IResult<&str, Operation> {
    let (input, op_code_str) = alpha1(input)?;
    let op_code = OpCode::from_str(op_code_str).unwrap();
    let (input, _) = space0(input)?;
    let r = parse_immediate(input);
    if r.is_ok() {
        let (input, immediate) = r.unwrap();
        Ok((
            input,
            Operation {
                op_code,
                immediate: Some(immediate),
                update_res: false,
                loop_start: None,
                loop_end: None,
            },
        ))
    } else {
        Ok((
            input,
            Operation {
                op_code,
                immediate: None,
                update_res: false,
                loop_start: None,
                loop_end: None,
            },
        ))
    }
}

pub fn parse_operation(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("operation:")(input)?;
    let (input, _) = multispace0(input)?;
    alt((parse_nop, parse_jump, parse_alu_operation, parse_memory)).parse(input)
}

impl Operation {
    pub fn to_mnemonics(&self) -> String {
        let mut result = String::new();
        result.push_str("operation: ");
        if self.op_code == OpCode::JUMP {
            result.push_str(&format!(
                "JUMP [{}, {}]",
                self.loop_start.unwrap(),
                self.loop_end.unwrap()
            ));
        } else {
            result.push_str(&self.op_code.to_string());
            if self.update_res {
                result.push_str("! ");
            } else {
                result.push_str(" ");
            }

            if let Some(imm) = self.immediate {
                result.push_str(&imm.to_string());
            }
        }
        result
    }

    pub fn from_mnemonics(s: &str) -> Result<Self, String> {
        let (_, operation) = parse_operation(s).map_err(|e| e.to_string())?;
        Ok(operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operation() {
        let input = "operation: ADD";
        let (_, operation) = parse_operation(input).unwrap();
        assert_eq!(
            operation,
            Operation {
                op_code: OpCode::ADD,
                immediate: None,
                update_res: false,
                loop_start: None,
                loop_end: None,
            }
        );

        let input = "operation: ADD!";
        let (_, operation) = parse_operation(input).unwrap();
        assert_eq!(
            operation,
            Operation {
                op_code: OpCode::ADD,
                immediate: None,
                update_res: true,
                loop_start: None,
                loop_end: None,
            }
        );

        let input = "operation: SUB 1";
        let (_, operation) = parse_operation(input).unwrap();
        assert_eq!(
            operation,
            Operation {
                op_code: OpCode::SUB,
                immediate: Some(1),
                update_res: false,
                loop_start: None,
                loop_end: None,
            }
        );

        let input = "operation: SUB! 1";
        let (_, operation) = parse_operation(input).unwrap();
        assert_eq!(
            operation,
            Operation {
                op_code: OpCode::SUB,
                immediate: Some(1),
                update_res: true,
                loop_start: None,
                loop_end: None,
            }
        );

        let input = "operation: JUMP [0, 5]";
        let (_, operation) = parse_operation(input).unwrap();
        assert_eq!(
            operation,
            Operation {
                op_code: OpCode::JUMP,
                immediate: Some(0),
                update_res: false,
                loop_start: Some(0),
                loop_end: Some(5),
            }
        );
        let str_back = operation.to_mnemonics();
        assert_eq!(input, str_back);
    }
}
