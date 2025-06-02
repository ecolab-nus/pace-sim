pub mod mnemonics {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, multispace0, space0},
    };

    use crate::isa::operation::Operation;

    fn parse_nop(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("NOP")(input)?;
        Ok((input, Operation::NOP))
    }

    fn parse_immediate(input: &str) -> IResult<&str, u16> {
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => Ok((input, immediate.parse::<u16>().unwrap())),
            Err(e) => Err(e),
        }
    }

    fn parse_add(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("ADD")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        // try to find immediate, if no immediate, use ADD(None, update_res)
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::ADD(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::ADD(None, update_res)))
        }
    }

    fn parse_sub(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SUB")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::SUB(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::SUB(None, update_res)))
        }
    }

    fn parse_mult(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("MULT")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::MULT(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::MULT(None, update_res)))
        }
    }

    fn parse_sext(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SEXT")(input)?;
        Ok((input, Operation::SEXT))
    }

    fn parse_div(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("DIV")(input)?;
        Ok((input, Operation::DIV))
    }

    fn parse_vadd(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("VADD")(input)?;
        Ok((input, Operation::VADD))
    }

    fn parse_vmul(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("VMUL")(input)?;
        Ok((input, Operation::VMUL))
    }

    fn parse_ls(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LS")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::LS(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::LS(None, update_res)))
        }
    }

    fn parse_rs(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("RS")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::RS(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::RS(None, update_res)))
        }
    }

    fn parse_asr(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("ASR")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::ASR(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::ASR(None, update_res)))
        }
    }

    fn parse_and(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("AND")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::AND(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::AND(None, update_res)))
        }
    }

    fn parse_or(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("OR")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::OR(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::OR(None, update_res)))
        }
    }

    fn parse_xor(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("XOR")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::XOR(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::XOR(None, update_res)))
        }
    }

    fn parse_sel(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SEL")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::SEL(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::SEL(None, update_res)))
        }
    }

    fn parse_cmerge(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CMERGE")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::CMERGE(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::CMERGE(None, update_res)))
        }
    }

    fn parse_cmp(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CMP")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::CMP(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::CMP(None, update_res)))
        }
    }

    fn parse_clt(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CLT")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::CLT(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::CLT(None, update_res)))
        }
    }

    fn parse_br(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("BR")(input)?;
        Ok((input, Operation::BR))
    }

    fn parse_cgt(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CGT")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = space0(input)?;
        let r = parse_immediate(input);
        if r.is_ok() {
            let (input, immediate) = r.unwrap();
            Ok((input, Operation::CGT(Some(immediate), update_res)))
        } else {
            Ok((input, Operation::CGT(None, update_res)))
        }
    }

    fn parse_movcl(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("MOVCL")(input)?;
        Ok((input, Operation::MOVCL))
    }

    fn parse_jump(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("JUMP")(input)?;
        Ok((input, Operation::JUMP))
    }

    fn parse_movc(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("MOVC")(input)?;
        Ok((input, Operation::MOVC))
    }

    fn parse_loadd(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LOADD")(input)?;
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::LOADD(Some(immediate))))
            }
            Err(_) => Ok((input, Operation::LOADD(None))),
        }
    }

    fn parse_stored(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STORED")(input)?;
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::STORED(Some(immediate))))
            }
            Err(_) => Ok((input, Operation::STORED(None))),
        }
    }

    fn parse_load(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LOAD")(input)?;
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::LOAD(Some(immediate))))
            }
            Err(_) => Ok((input, Operation::LOAD(None))),
        }
    }

    fn parse_store(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STORE")(input)?;
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::STORE(Some(immediate))))
            }
            Err(_) => Ok((input, Operation::STORE(None))),
        }
    }

    fn parse_loadb(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LOADB")(input)?;
        Ok((input, Operation::LOADB(None)))
    }

    fn parse_storeb(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STOREB")(input)?;
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::STOREB(Some(immediate))))
            }
            Err(_) => Ok((input, Operation::STOREB(None))),
        }
    }

    fn parse_arithmetic(input: &str) -> IResult<&str, Operation> {
        alt((
            parse_add,
            parse_sub,
            parse_mult,
            parse_sext,
            parse_div,
            parse_ls,
            parse_rs,
            parse_asr,
            parse_and,
            parse_xor,
            parse_or,
            parse_sel,
            parse_cmerge,
            parse_cmp,
            parse_clt,
            parse_cgt,
        ))
        .parse(input)
    }

    fn parse_simd(input: &str) -> IResult<&str, Operation> {
        alt((parse_vadd, parse_vmul)).parse(input)
    }

    fn parse_control(input: &str) -> IResult<&str, Operation> {
        alt((parse_br, parse_cgt, parse_movcl, parse_jump, parse_movc)).parse(input)
    }

    fn parse_memory(input: &str) -> IResult<&str, Operation> {
        alt((
            parse_loadd,
            parse_stored,
            parse_load,
            parse_store,
            parse_loadb,
            parse_storeb,
        ))
        .parse(input)
    }

    pub fn parse_operation(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("operation:")(input)?;
        let (input, _) = multispace0(input)?;
        alt((
            parse_nop,
            parse_arithmetic,
            parse_simd,
            parse_control,
            parse_memory,
        ))
        .parse(input)
    }

    impl Operation {
        pub fn to_mnemonics(&self) -> String {
            // Format:
            // operation: OP_NAME[!][imm]
            // where imm is the immediate value, and ! means update_res
            fn format_imm_and_update_res(imm: &Option<u16>, update_res: &bool) -> String {
                let mut result = String::new();
                if *update_res {
                    result.push_str("!");
                }
                result.push_str(&format_imm(imm));
                result
            }

            fn format_imm(imm: &Option<u16>) -> String {
                // add a space before the imm
                let mut result = String::from(" ");
                if let Some(imm) = imm {
                    result.push_str(&imm.to_string());
                }
                result
            }

            let mut result = String::new();
            result.push_str("operation: ");
            match self {
                Operation::NOP => result.push_str("NOP"),
                Operation::ADD(imm, update_res) => {
                    result.push_str("ADD");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::SUB(imm, update_res) => {
                    result.push_str("SUB");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::MULT(imm, update_res) => {
                    result.push_str("MULT");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::SEXT => {
                    todo!()
                }
                Operation::DIV => {
                    todo!()
                }
                Operation::VADD => {
                    todo!()
                }
                Operation::VMUL => {
                    todo!()
                }
                Operation::LS(imm, update_res) => {
                    result.push_str("LS");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::RS(imm, update_res) => {
                    result.push_str("RS");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::ASR(imm, update_res) => {
                    result.push_str("ASR");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::AND(imm, update_res) => {
                    result.push_str("AND");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::OR(imm, update_res) => {
                    result.push_str("OR");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::XOR(imm, update_res) => {
                    result.push_str("XOR");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::SEL(imm, update_res) => {
                    result.push_str("SEL");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::CMERGE(imm, update_res) => {
                    result.push_str("CMERGE");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::CMP(imm, update_res) => {
                    result.push_str("CMP");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::CLT(imm, update_res) => {
                    result.push_str("CLT");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::BR => {
                    todo!()
                }
                Operation::CGT(imm, update_res) => {
                    result.push_str("CGT");
                    result.push_str(&format_imm_and_update_res(imm, update_res));
                }
                Operation::MOVCL => {
                    todo!()
                }
                Operation::JUMP => {
                    todo!()
                }
                Operation::MOVC => {
                    todo!()
                }
                Operation::LOADD(imm) => {
                    result.push_str("LOADD");
                    result.push_str(&format_imm(imm));
                }
                Operation::STORED(imm) => {
                    result.push_str("STORED");
                    result.push_str(&format_imm(imm));
                }
                Operation::LOAD(imm) => {
                    result.push_str("LOAD");
                    result.push_str(&format_imm(imm));
                }
                Operation::STORE(imm) => {
                    result.push_str("STORE");
                    result.push_str(&format_imm(imm));
                }
                Operation::LOADB(imm) => {
                    result.push_str("LOADB");
                    result.push_str(&format_imm(imm));
                }
                Operation::STOREB(imm) => {
                    result.push_str("STOREB");
                    result.push_str(&format_imm(imm));
                }
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
            assert_eq!(operation, Operation::ADD(None, false));

            let input = "operation: ADD!";
            let (_, operation) = parse_operation(input).unwrap();
            assert_eq!(operation, Operation::ADD(None, true));

            let input = "operation: SUB 1";
            let (_, operation) = parse_operation(input).unwrap();
            assert_eq!(operation, Operation::SUB(Some(1), false));
        }
    }
}

pub mod binary {
    use crate::isa::{
        binary::{ConfigField, ConfigurationField},
        operation::Operation,
    };

    impl Operation {
        pub fn to_binary(&self) -> u64 {
            let mut code: u64 = 0;
            match self {
                Operation::NOP => {}
                Operation::ADD(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::SUB(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::MULT(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::SEXT => {
                    todo!()
                }
                Operation::DIV => {
                    todo!()
                }
                Operation::VADD => {
                    todo!()
                }
                Operation::VMUL => {
                    todo!()
                }
                Operation::LS(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::RS(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::ASR(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::AND(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::OR(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::XOR(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::SEL(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::CMERGE(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::CMP(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::CLT(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::BR => {
                    todo!()
                }
                Operation::CGT(imm, update_res) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                    code.set_bool_field(ConfigField::AluUpdateResBit, update_res);
                }
                Operation::MOVCL => {
                    todo!()
                }
                Operation::JUMP => {
                    todo!()
                }
                Operation::MOVC => {
                    todo!()
                }
                Operation::LOADD(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
                Operation::STORED(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
                Operation::LOAD(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
                Operation::STORE(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
                Operation::LOADB(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
                Operation::STOREB(immediate) => {
                    code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                    code.set_field(ConfigField::Immediate, immediate.unwrap_or(0) as u32);
                }
            }
            code
        }

        pub fn from_binary(code: u64) -> Self {
            let op = Operation::op_from_binary(code.get_field(ConfigField::OpCode) as u8);
            let immediate = code.get_field(ConfigField::Immediate) as u16;
            let update_res = code.get_bool_field(ConfigField::AluUpdateResBit);
            match op {
                Operation::ADD(_, _) => Operation::ADD(Some(immediate), update_res),
                Operation::SUB(_, _) => Operation::SUB(Some(immediate), update_res),
                Operation::MULT(_, _) => Operation::MULT(Some(immediate), update_res),
                Operation::SEXT => Operation::SEXT,
                Operation::DIV => Operation::DIV,
                Operation::VADD => Operation::VADD,
                Operation::VMUL => Operation::VMUL,
                Operation::LS(_, _) => Operation::LS(Some(immediate), update_res),
                Operation::RS(_, _) => Operation::RS(Some(immediate), update_res),
                Operation::ASR(_, _) => Operation::ASR(Some(immediate), update_res),
                Operation::AND(_, _) => Operation::AND(Some(immediate), update_res),
                Operation::OR(_, _) => Operation::OR(Some(immediate), update_res),
                Operation::XOR(_, _) => Operation::XOR(Some(immediate), update_res),
                Operation::SEL(_, _) => Operation::SEL(Some(immediate), update_res),
                Operation::CMERGE(_, _) => Operation::CMERGE(Some(immediate), update_res),
                Operation::CMP(_, _) => Operation::CMP(Some(immediate), update_res),
                Operation::CLT(_, _) => Operation::CLT(Some(immediate), update_res),
                Operation::BR => Operation::BR,
                Operation::CGT(_, _) => Operation::CGT(Some(immediate), update_res),
                Operation::MOVCL => Operation::MOVCL,
                Operation::JUMP => Operation::JUMP,
                Operation::MOVC => Operation::MOVC,
                Operation::LOADD(_) => Operation::LOADD(Some(immediate)),
                Operation::STORED(_) => Operation::STORED(Some(immediate)),
                Operation::LOAD(_) => Operation::LOAD(Some(immediate)),
                Operation::STORE(_) => Operation::STORE(Some(immediate)),
                Operation::LOADB(_) => Operation::LOADB(Some(immediate)),
                Operation::STOREB(_) => Operation::STOREB(Some(immediate)),
                Operation::NOP => Operation::NOP,
            }
        }

        fn op_to_binary(&self) -> u8 {
            match self {
                Operation::NOP => 0,
                Operation::ADD(_, _) => 1,
                Operation::SUB(_, _) => 2,
                Operation::MULT(_, _) => 3,
                Operation::SEXT => 4,
                Operation::DIV => 5,
                Operation::VADD => 6,
                Operation::VMUL => 7,
                Operation::LS(_, _) => 8,
                Operation::RS(_, _) => 9,
                Operation::ASR(_, _) => 10,
                Operation::AND(_, _) => 11,
                Operation::OR(_, _) => 12,
                Operation::XOR(_, _) => 13,
                Operation::SEL(_, _) => 16,
                Operation::CMERGE(_, _) => 17,
                Operation::CMP(_, _) => 18,
                Operation::CLT(_, _) => 19,
                Operation::BR => 20,
                Operation::CGT(_, _) => 21,
                Operation::MOVCL => 23,
                Operation::JUMP => 30,
                Operation::MOVC => 31,
                Operation::LOADD(_) => 14,
                Operation::STORED(_) => 15,
                Operation::LOAD(_) => 24,
                Operation::STORE(_) => 27,
                Operation::LOADB(_) => 26,
                Operation::STOREB(_) => 29,
            }
        }

        fn op_from_binary(code: u8) -> Self {
            match code {
                0 => Operation::NOP,
                1 => Operation::ADD(None, false),
                2 => Operation::SUB(None, false),
                3 => Operation::MULT(None, false),
                4 => Operation::SEXT,
                5 => Operation::DIV,
                6 => Operation::VADD,
                7 => Operation::VMUL,
                8 => Operation::LS(None, false),
                9 => Operation::RS(None, false),
                10 => Operation::ASR(None, false),
                11 => Operation::AND(None, false),
                12 => Operation::OR(None, false),
                13 => Operation::XOR(None, false),
                16 => Operation::SEL(None, false),
                17 => Operation::CMERGE(None, false),
                18 => Operation::CMP(None, false),
                19 => Operation::CLT(None, false),
                20 => Operation::BR,
                21 => Operation::CGT(None, false),
                23 => Operation::MOVCL,
                30 => Operation::JUMP,
                31 => Operation::MOVC,
                14 => Operation::LOADD(None),
                15 => Operation::STORED(None),
                24 => Operation::LOAD(None),
                27 => Operation::STORE(None),
                26 => Operation::LOADB(None),
                29 => Operation::STOREB(None),
                _ => panic!("Invalid operation code: {}", code),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::operation::Operation;

    #[test]
    fn test_add_sub_binary_conversions() {
        let add = Operation::ADD(Some(15), true);
        let binary = add.to_binary();
        let add_from_binary = Operation::from_binary(binary);
        assert_eq!(add, add_from_binary);
        let sub = Operation::SUB(Some(13), false);
        let binary = sub.to_binary();
        let sub_from_binary = Operation::from_binary(binary);
        assert_eq!(sub, sub_from_binary);
    }
}
