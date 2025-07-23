use crate::agu::instruction::*;
use crate::isa::binary::binary::BinaryIO;

pub struct AGUCM {
    pub instructions: Vec<Instruction>,
}

pub struct AGUARF {
    pub arfs: Vec<u16>,
}

impl BinaryIO for AGUARF {
    fn to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        for arf in &self.arfs {
            binary.extend(arf.to_binary());
        }
        binary
    }

    fn from_binary(_binary: &Vec<u8>) -> Result<Self, String> {
        todo!()
    }
}
impl BinaryIO for AGUCM {
    fn to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        for instruction in &self.instructions {
            binary.push(instruction.to_byte());
        }
        binary
    }
    fn from_binary(_binary: &Vec<u8>) -> Result<Self, String> {
        todo!()
    }
}

impl BinaryIO for Instruction {
    fn to_binary(&self) -> Vec<u8> {
        vec![self.to_byte()]
    }
    fn from_binary(binary: &Vec<u8>) -> Result<Self, String> {
        assert!(binary.len() == 1);
        Ok(Self::from_byte(binary[0]))
    }
}
impl Instruction {
    /// Loading from one byte, bit 0 is the least significant bit, bit 7 is the most significant bit
    pub fn from_byte(bin: u8) -> Self {
        // bit 0 is inst_type, 0 is LOAD, 1 is STORE
        // bit 1 is inst_mode, 0 is STRIDED, 1 is CONST
        // bit 2-3 is data_width, 00 is B8, 01 is B16, 10 is B64
        // bit 4-7 is the stride.
        let inst_type = if bin & 0b00000001 == 0 {
            InstType::LOAD
        } else {
            InstType::STORE
        };

        let inst_mode = if bin & 0b00000010 == 0 {
            InstMode::STRIDED
        } else {
            InstMode::CONST
        };

        let data_width = if bin & 0b00001100 == 0 {
            DataWidth::B8
        } else if bin & 0b00001100 == 1 {
            DataWidth::B16
        } else if bin & 0b00001100 == 2 {
            DataWidth::B64
        } else {
            panic!("Invalid data width");
        };

        let stride = (bin & 0b11110000) >> 4;

        Self {
            inst_type,
            inst_mode,
            data_width,
            stride,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut bin = 0;
        // Bit 0: inst_type (0 = LOAD, 1 = STORE)
        bin |= (self.inst_type as u8) << 0;
        // Bit 1: inst_mode (0 = STRIDED, 1 = CONST)
        bin |= (self.inst_mode as u8) << 1;
        // Bits 2-3: data_width (00 = B8, 01 = B16, 10 = B64)
        bin |= (self.data_width as u8) << 2;
        // Bits 4-7: stride (4 bits)
        bin |= (self.stride & 0b00001111) << 4;
        bin
    }
}
