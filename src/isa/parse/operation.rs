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
                },
            ))
        } else {
            Ok((
                input,
                Operation {
                    op_code,
                    immediate: None,
                    update_res,
                },
            ))
        }
    }

    // fn parse_add(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("ADD")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     // try to find immediate, if no immediate, use ADD(None, update_res)
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::ADD(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::ADD(None, update_res)))
    //     }
    // }

    // fn parse_sub(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("SUB")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::SUB(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::SUB(None, update_res)))
    //     }
    // }

    // fn parse_mult(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("MULT")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::MULT(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::MULT(None, update_res)))
    //     }
    // }

    // fn parse_sext(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("SEXT")(input)?;
    //     Ok((input, OpCode::SEXT))
    // }

    // fn parse_div(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("DIV")(input)?;
    //     Ok((input, OpCode::DIV))
    // }

    // fn parse_vadd(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("VADD")(input)?;
    //     Ok((input, OpCode::VADD))
    // }

    // fn parse_vmul(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("VMUL")(input)?;
    //     Ok((input, OpCode::VMUL))
    // }

    // fn parse_ls(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("LS")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::LS(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::LS(None, update_res)))
    //     }
    // }

    // fn parse_rs(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("RS")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::RS(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::RS(None, update_res)))
    //     }
    // }

    // fn parse_asr(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("ASR")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::ASR(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::ASR(None, update_res)))
    //     }
    // }

    // fn parse_and(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("AND")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::AND(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::AND(None, update_res)))
    //     }
    // }

    // fn parse_or(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("OR")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::OR(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::OR(None, update_res)))
    //     }
    // }

    // fn parse_xor(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("XOR")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::XOR(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::XOR(None, update_res)))
    //     }
    // }

    // fn parse_sel(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("SEL")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::SEL(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::SEL(None, update_res)))
    //     }
    // }

    // fn parse_cmerge(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("CMERGE")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::CMERGE(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::CMERGE(None, update_res)))
    //     }
    // }

    // fn parse_cmp(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("CMP")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::CMP(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::CMP(None, update_res)))
    //     }
    // }

    // fn parse_clt(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("CLT")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::CLT(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::CLT(None, update_res)))
    //     }
    // }

    // fn parse_br(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("BR")(input)?;
    //     Ok((input, OpCode::BR))
    // }

    // fn parse_cgt(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("CGT")(input)?;
    //     let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
    //     let update_res = update_res == "!";
    //     let (input, _) = space0(input)?;
    //     let r = parse_immediate(input);
    //     if r.is_ok() {
    //         let (input, immediate) = r.unwrap();
    //         Ok((input, OpCode::CGT(Some(immediate), update_res)))
    //     } else {
    //         Ok((input, OpCode::CGT(None, update_res)))
    //     }
    // }

    // fn parse_movcl(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("MOVCL")(input)?;
    //     Ok((input, OpCode::MOVCL))
    // }

    // fn parse_jump(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("JUMP")(input)?;
    //     Ok((input, OpCode::JUMP))
    // }

    // fn parse_movc(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("MOVC")(input)?;
    //     Ok((input, OpCode::MOVC))
    // }

    // fn parse_loadd(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("LOADD")(input)?;
    //     let (input, _) = multispace0(input)?;
    //     match digit1::<_, nom::error::Error<&str>>(input) {
    //         Ok((input, immediate)) => {
    //             let immediate = immediate.parse::<u16>().unwrap();
    //             Ok((input, OpCode::LOADD(Some(immediate))))
    //         }
    //         Err(_) => Ok((input, OpCode::LOADD(None))),
    //     }
    // }

    // fn parse_stored(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("STORED")(input)?;
    //     let (input, _) = multispace0(input)?;
    //     match digit1::<_, nom::error::Error<&str>>(input) {
    //         Ok((input, immediate)) => {
    //             let immediate = immediate.parse::<u16>().unwrap();
    //             Ok((input, OpCode::STORED(Some(immediate))))
    //         }
    //         Err(_) => Ok((input, OpCode::STORED(None))),
    //     }
    // }

    // fn parse_load(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("LOAD")(input)?;
    //     let (input, _) = multispace0(input)?;
    //     match digit1::<_, nom::error::Error<&str>>(input) {
    //         Ok((input, immediate)) => {
    //             let immediate = immediate.parse::<u16>().unwrap();
    //             Ok((input, OpCode::LOAD(Some(immediate))))
    //         }
    //         Err(_) => Ok((input, OpCode::LOAD(None))),
    //     }
    // }

    // fn parse_store(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("STORE")(input)?;
    //     let (input, _) = multispace0(input)?;
    //     match digit1::<_, nom::error::Error<&str>>(input) {
    //         Ok((input, immediate)) => {
    //             let immediate = immediate.parse::<u16>().unwrap();
    //             Ok((input, OpCode::STORE(Some(immediate))))
    //         }
    //         Err(_) => Ok((input, OpCode::STORE(None))),
    //     }
    // }

    // fn parse_loadb(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("LOADB")(input)?;
    //     Ok((input, OpCode::LOADB(None)))
    // }

    // fn parse_storeb(input: &str) -> IResult<&str, OpCode> {
    //     let (input, _) = tag("STOREB")(input)?;
    //     let (input, _) = multispace0(input)?;
    //     match digit1::<_, nom::error::Error<&str>>(input) {
    //         Ok((input, immediate)) => {
    //             let immediate = immediate.parse::<u16>().unwrap();
    //             Ok((input, OpCode::STOREB(Some(immediate))))
    //         }
    //         Err(_) => Ok((input, OpCode::STOREB(None))),
    //     }
    // }

    // fn parse_arithmetic(input: &str) -> IResult<&str, OpCode> {
    //     alt((
    //         parse_add,
    //         parse_sub,
    //         parse_mult,
    //         parse_sext,
    //         parse_div,
    //         parse_ls,
    //         parse_rs,
    //         parse_asr,
    //         parse_and,
    //         parse_xor,
    //         parse_or,
    //         parse_sel,
    //         parse_cmerge,
    //         parse_cmp,
    //         parse_clt,
    //         parse_cgt,
    //     ))
    //     .parse(input)
    // }

    // fn parse_simd(input: &str) -> IResult<&str, OpCode> {
    //     alt((parse_vadd, parse_vmul)).parse(input)
    // }

    // fn parse_control(input: &str) -> IResult<&str, OpCode> {
    //     alt((parse_br, parse_cgt, parse_movcl, parse_jump, parse_movc)).parse(input)
    // }

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
                },
            ))
        } else {
            Ok((
                input,
                Operation {
                    op_code,
                    immediate: None,
                    update_res: false,
                },
            ))
        }
    }

    pub fn parse_operation(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("operation:")(input)?;
        let (input, _) = multispace0(input)?;
        alt((
            parse_nop,
            parse_alu_operation,
            //parse_simd,
            //parse_control,
            parse_memory,
        ))
        .parse(input)
    }

    impl Operation {
        pub fn to_mnemonics(&self) -> String {
            let mut result = String::new();
            result.push_str("operation: ");
            result.push_str(&self.op_code.to_string());
            if self.update_res {
                result.push_str("! ");
            } else {
                result.push_str(" ");
            }

            if let Some(imm) = self.immediate {
                result.push_str(&imm.to_string());
            }
            result
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
                }
            );
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
            if let Some(imm) = self.immediate {
                code.set_field(ConfigField::MsbBit, 1);
                code.set_field(ConfigField::Immediate, imm as u32);
            } else {
                code.set_field(ConfigField::MsbBit, 0);
            }
            code.set_field(ConfigField::OpCode, self.op_code.to_binary() as u32);
            code.set_field(ConfigField::AluUpdateResBit, self.update_res as u32);
            code
        }

        pub fn from_binary(code: u64) -> Self {
            let op = OpCode::from_binary(code.get_field(ConfigField::OpCode) as u8);
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
    fn test_add_sub_binary_conversions() {
        let add = Operation {
            op_code: OpCode::ADD,
            immediate: Some(15),
            update_res: NO_UPDATE_RES,
        };
        let binary = add.to_binary();
        let add_from_binary = Operation::from_binary(binary);
        assert_eq!(add, add_from_binary);
        let sub = Operation {
            op_code: OpCode::SUB,
            immediate: Some(13),
            update_res: UPDATE_RES,
        };
        let binary = sub.to_binary();
        let sub_from_binary = Operation::from_binary(binary);
        assert_eq!(sub, sub_from_binary);
    }
}
