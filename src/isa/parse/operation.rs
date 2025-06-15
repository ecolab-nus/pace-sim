pub mod mnemonics {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, digit1, multispace0, space0},
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
        Ok((
            input,
            Operation {
                op_code: OpCode::JUMP,
                immediate: None,
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
                    immediate: None,
                    update_res: false,
                    loop_start: Some(0),
                    loop_end: Some(5),
                }
            );
            let str_back = operation.to_mnemonics();
            assert_eq!(input, str_back);
        }
    }
}

pub mod binary {
    use crate::isa::{
        binary::{ConfigField, ConfigurationField},
        operation::{OpCode, Operation},
    };

    impl Operation {
        pub fn to_binary(&self) -> u64 {
            let mut code: u64 = 0;
            if self.op_code == OpCode::JUMP {
                code.set_field(ConfigField::OpCode, 30);
                code.set_field(ConfigField::LoopStart, self.loop_start.unwrap() as u32);
                code.set_field(ConfigField::LoopEnd, self.loop_end.unwrap() as u32);
            } else {
                if let Some(imm) = self.immediate {
                    code.set_field(ConfigField::MsbBit, 1);
                    code.set_field(ConfigField::Immediate, imm as u32);
                } else {
                    code.set_field(ConfigField::MsbBit, 0);
                }
                code.set_field(ConfigField::OpCode, self.op_code.to_binary() as u32);
                code.set_field(ConfigField::AluUpdateResBit, self.update_res as u32);
            }
            code
        }

        pub fn from_binary(code: u64) -> Self {
            let op = OpCode::from_binary(code.get_field(ConfigField::OpCode) as u8);

            if op == OpCode::JUMP {
                let loop_start = code.get_field(ConfigField::LoopStart) as u8;
                let loop_end = code.get_field(ConfigField::LoopEnd) as u8;
                return Operation {
                    op_code: op,
                    immediate: None,
                    update_res: false,
                    loop_start: Some(loop_start),
                    loop_end: Some(loop_end),
                };
            }

            let immediate = if code.get_field(ConfigField::MsbBit) == 1 {
                Some(code.get_field(ConfigField::Immediate) as u16)
            } else {
                None
            };
            let update_res = code.get_bool_field(ConfigField::AluUpdateResBit);
            Operation {
                op_code: op,
                immediate,
                update_res,
                loop_start: None,
                loop_end: None,
            }
        }
    }

    impl OpCode {
        fn to_binary(&self) -> u8 {
            match self {
                OpCode::NOP => 0,
                OpCode::ADD => 1,
                OpCode::SUB => 2,
                OpCode::MULT => 3,
                OpCode::SEXT => 4,
                OpCode::DIV => 5,
                OpCode::VADD => 6,
                OpCode::VMUL => 7,
                OpCode::LS => 8,
                OpCode::RS => 9,
                OpCode::ASR => 10,
                OpCode::AND => 11,
                OpCode::OR => 12,
                OpCode::XOR => 13,
                OpCode::SEL => 16,
                OpCode::CMERGE => 17,
                OpCode::CMP => 18,
                OpCode::CLT => 19,
                OpCode::BR => 20,
                OpCode::CGT => 21,
                OpCode::MOVCL => 23,
                OpCode::JUMP => 30,
                OpCode::MOVC => 31,
                OpCode::LOADD => 14,
                OpCode::STORED => 15,
                OpCode::LOAD => 24,
                OpCode::STORE => 27,
                OpCode::LOADB => 26,
                OpCode::STOREB => 29,
            }
        }

        fn from_binary(code: u8) -> Self {
            match code {
                0 => OpCode::NOP,
                1 => OpCode::ADD,
                2 => OpCode::SUB,
                3 => OpCode::MULT,
                4 => OpCode::SEXT,
                5 => OpCode::DIV,
                6 => OpCode::VADD,
                7 => OpCode::VMUL,
                8 => OpCode::LS,
                9 => OpCode::RS,
                10 => OpCode::ASR,
                11 => OpCode::AND,
                12 => OpCode::OR,
                13 => OpCode::XOR,
                16 => OpCode::SEL,
                17 => OpCode::CMERGE,
                18 => OpCode::CMP,
                19 => OpCode::CLT,
                20 => OpCode::BR,
                21 => OpCode::CGT,
                23 => OpCode::MOVCL,
                30 => OpCode::JUMP,
                31 => OpCode::MOVC,
                14 => OpCode::LOADD,
                15 => OpCode::STORED,
                24 => OpCode::LOAD,
                27 => OpCode::STORE,
                26 => OpCode::LOADB,
                29 => OpCode::STOREB,
                _ => panic!("Invalid operation code: {}", code),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::operation::*;

    #[test]
    fn test_binary_conversions() {
        let add = Operation {
            op_code: OpCode::ADD,
            immediate: Some(15),
            update_res: NO_UPDATE_RES,
            loop_start: None,
            loop_end: None,
        };
        let binary = add.to_binary();
        let add_from_binary = Operation::from_binary(binary);
        assert_eq!(add, add_from_binary);
        let sub = Operation {
            op_code: OpCode::SUB,
            immediate: Some(13),
            update_res: UPDATE_RES,
            loop_start: None,
            loop_end: None,
        };
        let binary = sub.to_binary();
        let sub_from_binary = Operation::from_binary(binary);
        assert_eq!(sub, sub_from_binary);

        let jump = Operation {
            op_code: OpCode::JUMP,
            immediate: None,
            update_res: false,
            loop_start: Some(0),
            loop_end: Some(5),
        };
        let binary = jump.to_binary();
        let jump_from_binary = Operation::from_binary(binary);
        assert_eq!(jump, jump_from_binary);
    }
}
