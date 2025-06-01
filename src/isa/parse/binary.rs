use crate::isa::{
    configuration::{Configuration, Program},
    operation::Operation,
    router::{DirectionsOpt, RouterConfig, RouterInDir, RouterSwitchConfig},
};

type ConfigCode = u64;

#[derive(Debug, Clone, Copy)]
pub enum ConfigField {
    PredicateBit,
    MsbBit,
    UseFloatBit,
    AluBypassBit,
    DisablePeRfBit,
    Immediate,
    LoopEndAddress,
    LoopStartAddress,
    OpCode,
    RouterWriteEnable,
    AluUpdateResBit,
    RouterBypass,
    RouterSwitchConfig,
}

impl ConfigField {
    /// Returns the bit range (start, end) where:
    /// - start is inclusive (the first bit to read)
    /// - end is exclusive (one past the last bit to read)
    /// For example, (0,1) means just bit 0, (0,2) means bits 0 and 1
    fn get_range(&self) -> (u8, u8) {
        match self {
            ConfigField::PredicateBit => (63, 64),      // 1 bit: bit 63
            ConfigField::MsbBit => (62, 63),            // 1 bit: bit 62
            ConfigField::UseFloatBit => (61, 62),       // 1 bit: bit 61
            ConfigField::AluBypassBit => (60, 61),      // 1 bit: bit 60
            ConfigField::DisablePeRfBit => (59, 60),    // 1 bit: bit 59
            ConfigField::Immediate => (35, 51),         // 16 bits: bits 35-50
            ConfigField::LoopEndAddress => (40, 44),    // 4 bits: bits 40-43
            ConfigField::LoopStartAddress => (35, 40),  // 5 bits: bits 35-39
            ConfigField::OpCode => (30, 35),            // 5 bits: bits 30-34
            ConfigField::RouterWriteEnable => (26, 30), // 4 bits: bits 26-29
            ConfigField::AluUpdateResBit => (25, 26),   // 1 bit: bit 25
            ConfigField::RouterBypass => (21, 25),      // 4 bits: bits 21-24
            ConfigField::RouterSwitchConfig => (0, 21), // 21 bits: bits 0-20
        }
    }
}

pub trait ConfigurationField {
    fn get_field(&self, field: ConfigField) -> u32;
    fn set_field(&mut self, field: ConfigField, value: u32);
    fn get_bool_field(&self, field: ConfigField) -> bool;
    fn set_bool_field(&mut self, field: ConfigField, value: bool);
}

impl ConfigurationField for u64 {
    fn get_field(&self, field: ConfigField) -> u32 {
        let (start, end) = field.get_range();
        assert!(start < 64 && end <= 64 && start < end, "Invalid bit range");
        let mask = (1 << (end - start)) - 1;
        ((self >> start) & mask) as u32
    }

    fn set_field(&mut self, field: ConfigField, value: u32) {
        let (start, end) = field.get_range();
        let len = end - start;
        assert!(
            start < 64 && end <= 64 && start < end,
            "Bit range out of bounds"
        );
        assert!(value < (1 << len), "Value too large for field width");
        let mask = ((1 << len) - 1) as u64;
        *self = (*self & !(mask << start)) | ((value as u64 & mask) << start);
    }

    fn get_bool_field(&self, field: ConfigField) -> bool {
        assert_eq!(
            field.get_range().1 - field.get_range().0,
            1,
            "Field is not a single bit"
        );
        self.get_field(field) == 1
    }

    fn set_bool_field(&mut self, field: ConfigField, value: bool) {
        assert_eq!(
            field.get_range().1 - field.get_range().0,
            1,
            "Field is not a single bit"
        );
        self.set_field(field, value as u32);
    }
}

impl RouterInDir {
    const BINARY_MAPPING: [RouterInDir; 8] = [
        RouterInDir::EastIn,  // 000
        RouterInDir::SouthIn, // 001
        RouterInDir::WestIn,  // 010
        RouterInDir::NorthIn, // 011
        RouterInDir::ALUOut,  // 100
        RouterInDir::ALURes,  // 101
        RouterInDir::Invalid, // 110
        RouterInDir::Open,    // 111
    ];

    pub fn from_binary(code: u8) -> Self {
        assert!(
            code <= 7 && code != 6,
            "Invalid router direction code: {}",
            code
        );
        Self::BINARY_MAPPING[code as usize]
    }

    pub fn to_binary(&self) -> u8 {
        assert!(
            !matches!(self, RouterInDir::Invalid),
            "Invalid router direction"
        );
        *self as u8
    }
}

impl RouterSwitchConfig {
    /// Converts a 21-bit binary code to a RouterSwitchConfig
    pub fn from_binary(code: u32) -> Self {
        // first, check if the code is in 21 bits
        assert!(
            code < (1 << 21),
            "Invalid router switch config code: {}",
            code
        );
        // From LSB to MSB
        // first 3 bits are the predicate
        let east_out = RouterInDir::from_binary((code & 0b111) as u8);
        // next 3 bits are the alu_op1 or RHS
        let south_out = RouterInDir::from_binary(((code >> 3) & 0b111) as u8);
        // next 3 bits are the alu_op2 or LHS
        let west_out = RouterInDir::from_binary(((code >> 6) & 0b111) as u8);
        // next 3 bits are the North Out
        let north_out = RouterInDir::from_binary(((code >> 9) & 0b111) as u8);
        // next 3 bits are the West Out
        let alu_op1 = RouterInDir::from_binary(((code >> 12) & 0b111) as u8);
        // next 3 bits are the South Out
        let alu_op2 = RouterInDir::from_binary(((code >> 15) & 0b111) as u8);
        // next 3 bits are the East Out
        let predicate = RouterInDir::from_binary(((code >> 18) & 0b111) as u8);
        Self {
            predicate,
            alu_op2,
            alu_op1,
            north_out,
            west_out,
            south_out,
            east_out,
        }
    }

    /// Converts a RouterSwitchConfig to a 21-bit binary code
    pub fn to_binary(&self) -> u32 {
        let mut code: u32 = 0;
        // MSB first 3 bits are the predicate
        code |= self.predicate.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the alu_op2
        code |= self.alu_op2.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the alu_op1
        code |= self.alu_op1.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the North Out
        code |= self.north_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the West Out
        code |= self.west_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the South Out
        code |= self.south_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the East Out
        code |= self.east_out.to_binary() as u32;
        code
    }
}

impl DirectionsOpt {
    /// Converts a 4-bit binary code to a DirectionsOpt
    /// in the order of North, South, West, East
    pub fn from_binary(code: u8) -> Self {
        assert!(code < 16, "Invalid directions code: {}", code);
        let mut directions = Self::default();
        directions.north = (code & 0b1000) != 0;
        directions.south = (code & 0b0100) != 0;
        directions.west = (code & 0b0010) != 0;
        directions.east = (code & 0b0001) != 0;
        directions
    }

    pub fn to_binary(&self) -> u8 {
        let mut code: u8 = 0;
        code |= (self.north as u8) << 3;
        code |= (self.south as u8) << 2;
        code |= (self.west as u8) << 1;
        code |= (self.east as u8) << 0;
        code
    }
}

impl RouterConfig {
    pub fn from_binary(code: u64) -> Self {
        let switch_config =
            RouterSwitchConfig::from_binary(code.get_field(ConfigField::RouterSwitchConfig) as u32);
        let input_register_bypass =
            DirectionsOpt::from_binary(code.get_field(ConfigField::RouterBypass) as u8);
        let input_register_write =
            DirectionsOpt::from_binary(code.get_field(ConfigField::RouterWriteEnable) as u8);
        Self {
            switch_config,
            input_register_bypass,
            input_register_write,
        }
    }

    pub fn to_binary(&self) -> u64 {
        let mut code: u64 = 0;
        code.set_field(
            ConfigField::RouterSwitchConfig,
            self.switch_config.to_binary() as u32,
        );
        code.set_field(
            ConfigField::RouterBypass,
            self.input_register_bypass.to_binary() as u32,
        );
        code.set_field(
            ConfigField::RouterWriteEnable,
            self.input_register_write.to_binary() as u32,
        );

        code
    }
}

impl Operation {
    pub fn to_binary(&self) -> u64 {
        let mut code: u64 = 0;
        match self {
            Operation::NOP => {}
            Operation::ADD(imm, update_res) => {
                code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                code.set_bool_field(ConfigField::AluUpdateResBit, *update_res);
            }
            Operation::SUB(imm, update_res) => {
                code.set_field(ConfigField::OpCode, self.op_to_binary() as u32);
                code.set_field(ConfigField::Immediate, imm.unwrap_or(0) as u32);
                code.set_bool_field(ConfigField::AluUpdateResBit, *update_res);
            }
            Operation::MULT => {
                todo!()
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
            Operation::LS => {
                todo!()
            }
            Operation::RS => {
                todo!()
            }
            Operation::ASR => {
                todo!()
            }
            Operation::AND => {
                todo!()
            }
            Operation::OR => {
                todo!()
            }
            Operation::XOR => {
                todo!()
            }
            Operation::SEL => {
                todo!()
            }
            Operation::CMERGE => {
                todo!()
            }
            Operation::CMP => {
                todo!()
            }
            Operation::CLT => {
                todo!()
            }
            Operation::BR => {
                todo!()
            }
            Operation::CGT => {
                todo!()
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
            Operation::MULT => Operation::MULT,
            Operation::SEXT => Operation::SEXT,
            Operation::DIV => Operation::DIV,
            Operation::VADD => Operation::VADD,
            Operation::VMUL => Operation::VMUL,
            Operation::LS => Operation::LS,
            Operation::RS => Operation::RS,
            Operation::ASR => Operation::ASR,
            Operation::AND => Operation::AND,
            Operation::OR => Operation::OR,
            Operation::XOR => Operation::XOR,
            Operation::SEL => Operation::SEL,
            Operation::CMERGE => Operation::CMERGE,
            Operation::CMP => Operation::CMP,
            Operation::CLT => Operation::CLT,
            Operation::BR => Operation::BR,
            Operation::CGT => Operation::CGT,
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
            Operation::MULT => 3,
            Operation::SEXT => 4,
            Operation::DIV => 5,
            Operation::VADD => 6,
            Operation::VMUL => 7,
            Operation::LS => 8,
            Operation::RS => 9,
            Operation::ASR => 10,
            Operation::AND => 11,
            Operation::OR => 12,
            Operation::XOR => 13,
            Operation::SEL => 16,
            Operation::CMERGE => 17,
            Operation::CMP => 18,
            Operation::CLT => 19,
            Operation::BR => 20,
            Operation::CGT => 21,
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
            1 => Operation::ADD(None, true),
            2 => Operation::SUB(None, true),
            3 => Operation::MULT,
            4 => Operation::SEXT,
            5 => Operation::DIV,
            6 => Operation::VADD,
            7 => Operation::VMUL,
            8 => Operation::LS,
            9 => Operation::RS,
            10 => Operation::ASR,
            11 => Operation::AND,
            12 => Operation::OR,
            13 => Operation::XOR,
            16 => Operation::SEL,
            17 => Operation::CMERGE,
            18 => Operation::CMP,
            19 => Operation::CLT,
            20 => Operation::BR,
            21 => Operation::CGT,
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

impl Configuration {
    pub fn to_binary(&self) -> u64 {
        let router_config: u64 = self.router_config.to_binary();
        let operation: u64 = self.operation.to_binary();
        let code = router_config | operation;
        code
    }
    pub fn from_binary(code: u64) -> Self {
        let router_config = RouterConfig::from_binary(code);
        let operation = Operation::from_binary(code);
        Self {
            router_config,
            operation,
        }
    }
}

impl BinaryIO for Program {
    fn to_binary_str(&self) -> String {
        self.configurations
            .iter()
            .map(|c| c.to_binary().to_binary_str())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn from_binary_str(s: &str) -> Self {
        // split by newline, remove spaces
        let lines = s.lines().map(|l| l.trim()).collect::<Vec<&str>>();
        // for each line, convert to binary
        let binaries = lines
            .iter()
            .map(|l| ConfigCode::from_binary_str(l))
            .collect::<Vec<u64>>();
        // convert to program
        Self {
            configurations: binaries
                .iter()
                .map(|b| Configuration::from_binary(*b))
                .collect(),
        }
    }

    fn to_binary(&self) -> u64 {
        todo!()
    }

    fn from_binary(_: u64) -> Self {
        todo!()
    }
}

pub trait BinaryIO {
    fn to_binary_str(&self) -> String;
    fn to_binary(&self) -> u64;
    fn from_binary_str(s: &str) -> Self;
    fn from_binary(code: u64) -> Self;
}

impl BinaryIO for u64 {
    fn to_binary_str(&self) -> String {
        format!("0b{:b}", self)
    }

    fn to_binary(&self) -> u64 {
        *self
    }

    fn from_binary_str(s: &str) -> Self {
        let mut code: u64 = 0;
        for (i, c) in s.chars().enumerate() {
            if c == '1' {
                code |= 1 << (s.len() - i - 1);
            }
        }
        code
    }

    fn from_binary(code: u64) -> Self {
        code
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::configuration::Configuration;

    use super::*;

    #[test]
    fn test_binary_io() {
        let code = 0b10101010101010101010101010101010;
        let code_str = code.to_binary_str();
        let code_binary = code.to_binary();
        assert_eq!(code_str, "0b10101010101010101010101010101010");
        assert_eq!(code_binary, 0b10101010101010101010101010101010);
        let code_from_str = ConfigCode::from_binary_str(&code_str);
        let code_from_binary = ConfigCode::from_binary(code_binary);
        assert_eq!(code_from_str, code);
        assert_eq!(code_from_binary, code);
    }

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

    #[test]
    fn test_router_switch_config_binary_conversions() {
        let switch_config = RouterSwitchConfig {
            predicate: RouterInDir::Open,
            alu_op1: RouterInDir::ALUOut,
            alu_op2: RouterInDir::ALURes,
            north_out: RouterInDir::NorthIn,
            west_out: RouterInDir::WestIn,
            south_out: RouterInDir::SouthIn,
            east_out: RouterInDir::EastIn,
        };
        let binary = switch_config.to_binary();
        let switch_config_from_binary = RouterSwitchConfig::from_binary(binary);
        assert_eq!(switch_config, switch_config_from_binary);
    }

    #[test]
    fn test_router_config_binary_conversions() {
        let router_config = RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::ALURes,
                north_out: RouterInDir::NorthIn,
                west_out: RouterInDir::WestIn,
                south_out: RouterInDir::SouthIn,
                east_out: RouterInDir::EastIn,
            },
            input_register_bypass: DirectionsOpt {
                north: true,
                south: true,
                west: false,
                east: true,
            },
            input_register_write: DirectionsOpt {
                north: true,
                south: false,
                west: false,
                east: false,
            },
        };
        let binary = router_config.to_binary();
        let router_config_from_binary = RouterConfig::from_binary(binary);
        assert_eq!(router_config, router_config_from_binary);
    }

    #[test]
    fn test_configuration_binary_conversions() {
        let configuration_str = r"operation: ADD! 15
            switch_config: {
            Open -> predicate,
            SouthIn -> south_out,
            WestIn -> west_out,
            NorthIn -> north_out,
            EastIn -> east_out,
            ALURes -> alu_op2,
            ALUOut -> alu_op1,
        };
        input_register_bypass: {north, south};
        input_register_write: {east, west};";
        let configuration = Configuration::from_str(configuration_str).unwrap();
        let binary = configuration.to_binary();
        let configuration_from_binary = Configuration::from_binary(binary);
        assert_eq!(configuration.operation, configuration_from_binary.operation);
        assert_eq!(
            configuration.router_config,
            configuration_from_binary.router_config
        );
        assert_eq!(configuration, configuration_from_binary);
    }
}
