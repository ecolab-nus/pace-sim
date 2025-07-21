use crate::isa::{
    binary::configuration::{ConfigField, ConfigurationField},
    operation::{OpCode, Operation},
};

impl Operation {
    /// Convert the operation to a 64-bit binary code, MSB first, LSB last
    pub fn to_u64(&self) -> u64 {
        let mut code: u64 = 0;
        if self.op_code == OpCode::JUMP {
            code.set_field(ConfigField::OpCode, 30);
            if let Some(_) = self.immediate {
                unimplemented!("Jump to immediate destination is not implemented");
            } else {
                // if jump dst not set, use the loop start as jump dst
                code.set_field(ConfigField::JumpDst, self.loop_start.unwrap() as u32);
            }
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

    pub fn from_u64(code: u64) -> Self {
        let op = OpCode::from_binary(code.get_field(ConfigField::OpCode) as u8);

        if op == OpCode::JUMP {
            let loop_start = code.get_field(ConfigField::LoopStart) as u8;
            let loop_end = code.get_field(ConfigField::LoopEnd) as u8;
            let jump_dst = code.get_field(ConfigField::JumpDst) as u8;
            assert_eq!(
                jump_dst, loop_start,
                "Jump destination must be the same as loop start"
            );
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
        let binary = add.to_u64();
        let add_from_binary = Operation::from_u64(binary);
        assert_eq!(add, add_from_binary);
        let sub = Operation {
            op_code: OpCode::SUB,
            immediate: Some(13),
            update_res: UPDATE_RES,
            loop_start: None,
            loop_end: None,
        };
        let binary = sub.to_u64();
        let sub_from_binary = Operation::from_u64(binary);
        assert_eq!(sub, sub_from_binary);

        let jump = Operation {
            op_code: OpCode::JUMP,
            immediate: None,
            update_res: false,
            loop_start: Some(0),
            loop_end: Some(5),
        };
        let binary = jump.to_u64();
        let jump_from_binary = Operation::from_u64(binary);
        assert_eq!(jump, jump_from_binary);
    }
}
