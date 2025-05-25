pub mod readable {
    use nom::{IResult, Parser, branch::alt, bytes::complete::tag};

    use crate::isa::operation::Operation;

    fn parse_nop(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("NOP")(input)?;
        Ok((input, Operation::NOP))
    }

    fn parse_add(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("ADD")(input)?;
        Ok((input, Operation::ADD))
    }

    fn parse_sub(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SUB")(input)?;
        Ok((input, Operation::SUB))
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

    // TODO
    fn parse_sel(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("SEL")(input)?;
        Ok((input, Operation::SEL))
    }

    // TODO
    fn parse_cmerge(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CMERGE")(input)?;
        Ok((input, Operation::CMERGE))
    }

    // TODO
    fn parse_cmp(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CMP")(input)?;
        Ok((input, Operation::CMP))
    }

    // TODO
    fn parse_clt(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("CLT")(input)?;
        Ok((input, Operation::CLT))
    }

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
        Ok((input, Operation::LOADD))
    }

    fn parse_stored(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STORED")(input)?;
        Ok((input, Operation::STORED))
    }

    fn parse_load(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LOAD")(input)?;
        Ok((input, Operation::LOAD))
    }

    fn parse_store(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STORE")(input)?;
        Ok((input, Operation::STORE))
    }

    fn parse_loadb(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("LOADB")(input)?;
        Ok((input, Operation::LOADB))
    }

    fn parse_storeb(input: &str) -> IResult<&str, Operation> {
        let (input, _) = tag("STOREB")(input)?;
        Ok((input, Operation::STOREB))
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
        alt((
            parse_nop,
            parse_arithmetic,
            parse_simd,
            parse_control,
            parse_memory,
        ))
        .parse(input)
    }
}
