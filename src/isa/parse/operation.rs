pub mod mnemonics {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, multispace0},
    };

    use crate::isa::operation::Operation;

    fn parse_nop(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("NOP")(input)?;
        Ok((input, Operation::NOP))
    }

    fn parse_add(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("ADD")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::ADD(Some(immediate), update_res)))
            }
            Err(_) => Ok((input, Operation::ADD(None, update_res))),
        }
    }

    fn parse_sub(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SUB")(input)?;
        let (input, update_res) = alt((tag("!"), tag(""))).parse(input)?;
        let update_res = update_res == "!";
        let (input, _) = multispace0(input)?;
        match digit1::<_, nom::error::Error<&str>>(input) {
            Ok((input, immediate)) => {
                let immediate = immediate.parse::<u16>().unwrap();
                Ok((input, Operation::SUB(Some(immediate), update_res)))
            }
            Err(_) => Ok((input, Operation::SUB(None, update_res))),
        }
    }

    fn parse_mult(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("MULT")(input)?;
        Ok((input, Operation::MULT))
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
        Ok((input, Operation::LS))
    }

    fn parse_rs(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("RS")(input)?;
        Ok((input, Operation::RS))
    }

    fn parse_asr(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("ASR")(input)?;
        Ok((input, Operation::ASR))
    }

    fn parse_and(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("AND")(input)?;
        Ok((input, Operation::AND))
    }

    fn parse_or(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("OR")(input)?;
        Ok((input, Operation::OR))
    }

    fn parse_xor(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("XOR")(input)?;
        Ok((input, Operation::XOR))
    }

    // // TODO
    // fn parse_sel(input: &str) -> IResult<&str, Operation> {
    //     let (input, _) = tag("SEL")(input)?;
    //     Ok((input, Operation::SEL))
    // }

    // // TODO
    // fn parse_cmerge(input: &str) -> IResult<&str, Operation> {
    //     let (input, _) = tag("CMERGE")(input)?;
    //     Ok((input, Operation::CMERGE))
    // }

    // // TODO
    // fn parse_cmp(input: &str) -> IResult<&str, Operation> {
    //     let (input, _) = tag("CMP")(input)?;
    //     Ok((input, Operation::CMP))
    // }

    // // TODO
    // fn parse_clt(input: &str) -> IResult<&str, Operation> {
    //     let (input, _) = tag("CLT")(input)?;
    //     Ok((input, Operation::CLT))
    // }

    fn parse_br(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("BR")(input)?;
        Ok((input, Operation::BR))
    }

    fn parse_cgt(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CGT")(input)?;
        Ok((input, Operation::CGT))
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
            parse_add, parse_sub, parse_mult, parse_sext, parse_div, parse_ls, parse_rs, parse_asr,
            parse_and, parse_xor, parse_or,
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
