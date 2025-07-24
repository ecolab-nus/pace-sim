use crate::isa::{
    binary::binary::BinaryIO,
    configuration::{Configuration, Program},
    operation::Operation,
    router::RouterConfig,
};

type ConfigCode = u64;

#[derive(Debug, Clone, Copy)]
pub enum ConfigField {
    PredicateBit,
    MsbBit,
    UseFloatBit,
    AluBypassBit,
    AguTrigger,
    Immediate,
    LoopEnd,
    LoopStart,
    OpCode,
    RouterWriteEnable,
    AluUpdateResBit,
    RouterBypass,
    RouterSwitchConfig,
    JumpDst,
}

impl ConfigField {
    /// Get the bit range for the field, MSB first, LSB last
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
            ConfigField::AguTrigger => (59, 60),        // 1 bit: bit 59
            ConfigField::Immediate => (35, 51),         // 16 bits: bits 35-50
            ConfigField::LoopEnd => (40, 45),           // 5 bits: bits 40-44
            ConfigField::LoopStart => (35, 40),         // 5 bits: bits 35-39
            ConfigField::OpCode => (30, 35),            // 5 bits: bits 30-34
            ConfigField::RouterWriteEnable => (26, 30), // 4 bits: bits 26-29
            ConfigField::AluUpdateResBit => (25, 26),   // 1 bit: bit 25
            ConfigField::RouterBypass => (21, 25),      // 4 bits: bits 21-24
            ConfigField::RouterSwitchConfig => (0, 21), // 21 bits: bits 0-20
            ConfigField::JumpDst => (45, 50),           // 14 bits: bits 21-34
        }
    }
}

pub trait ConfigurationField {
    fn get_field(&self, field: ConfigField) -> u32;
    fn set_field(&mut self, field: ConfigField, value: u32);
    fn get_bool_field(&self, field: ConfigField) -> bool;
    fn set_bool_field(&mut self, field: ConfigField, value: bool);
}

impl ConfigurationField for ConfigCode {
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

impl BinaryIO for Configuration {
    fn to_binary(&self) -> Vec<u8> {
        let router_config: u64 = self.router_config.to_u64();
        let operation: u64 = self.operation.to_u64();
        let code = router_config | operation;
        code.to_le_bytes().to_vec()
    }

    fn from_binary(code: &Vec<u8>) -> Result<Self, String> {
        let code = u64::from_binary(code)?;
        let router_config = RouterConfig::from_u64(code);
        let operation = Operation::from_u64(code);
        Ok(Self {
            router_config,
            operation,
        })
    }
}

impl BinaryIO for Program {
    fn to_binary(&self) -> Vec<u8> {
        self.configurations
            .iter()
            .map(|c| c.to_binary())
            .collect::<Vec<Vec<u8>>>()
            .concat()
    }

    fn from_binary(code: &Vec<u8>) -> Result<Self, String> {
        // make sure the length is a multiple of 8
        assert_eq!(
            code.len() % 8,
            0,
            "Invalid binary length, not multiple of 8"
        );
        // convert code into chunks of 8 bytes
        let chunks = code.chunks(8);
        // for each chunk, convert to u64 with little endian encoding
        let configurations = chunks
            .map(|c| Configuration::from_binary(&c.to_vec()))
            .collect::<Result<Vec<Configuration>, String>>()?;
        Ok(Program { configurations })
    }
}

impl Configuration {
    /// Convert the configuration to a 64-bit binary code, MSB first, LSB last
    pub fn to_u64(&self) -> u64 {
        let router_config: u64 = self.router_config.to_u64();
        let operation: u64 = self.operation.to_u64();
        let mut code = router_config | operation;
        if self.operation.is_mem() {
            code.set_bool_field(ConfigField::AguTrigger, true);
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::binary::binary::BinaryStringIO;
    use crate::isa::operation::*;
    use std::path::Path;

    use super::*;

    #[test]
    fn test_configuration_binary_conversions() {
        let configuration = Configuration::from_mnemonics(
            r"operation: ADD! 15
            switch_config: {
            Open -> predicate,
            SouthIn -> south_out,
            WestIn -> west_out,
            NorthIn -> north_out,
            EastIn -> east_out,
            ALURes -> alu_op2,
            ALUOut -> alu_op1,
        };
        input_register_used: {north, south};
        input_register_write: {east, west};",
        )
        .unwrap();
        let binary = configuration.to_binary();
        let configuration_from_binary = Configuration::from_binary(&binary).unwrap();
        assert_eq!(configuration, configuration_from_binary);

        let configuration = Configuration::from_mnemonics(
            r"operation: ADD
switch_config: {
    Open -> predicate,
    ALUOut -> south_out,
    Open -> west_out,
    Open -> north_out,
    Open -> east_out,
    Open -> alu_op2,
    Open -> alu_op1,
};
input_register_used: {};
input_register_write: {};
",
        )
        .unwrap();
        assert_eq!(
            configuration.operation,
            Operation {
                op_code: OpCode::ADD,
                immediate: None,
                update_res: NO_UPDATE_RES,
                loop_start: None,
                loop_end: None,
            }
        );
        let binary = configuration.to_binary();
        let configuration_from_binary = Configuration::from_binary(&binary).unwrap();
        assert_eq!(configuration, configuration_from_binary);
    }

    #[test]
    fn test_program_binary_conversions() {
        // Converting from binprog to prog, then back to binprog
        let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let test_file = Path::new(&root_path).join("tests/test1.binprog");
        let str_program = std::fs::read_to_string(test_file).unwrap();
        // remove the spaces and linebreaks
        let str_program = str_program.replace(" ", "").replace("\n", "");
        // convert to Vec<u8> and then to Program
        let binary = Vec::<u8>::from_binary_str(&str_program).unwrap();
        let program = Program::from_binary(&binary).unwrap();

        let new_binary = program.to_binary();
        let new_program = Program::from_binary(&new_binary).unwrap();
        assert_eq!(program, new_program);

        // Converting from prog to binprog, then back to prog
        let mnemonic_file = Path::new(&root_path).join("tests/test1.prog");
        let str_program = std::fs::read_to_string(mnemonic_file).unwrap();
        let program = Program::from_mnemonics(&str_program).unwrap();
        let new_binary = program.to_binary();
        let new_program = Program::from_binary(&new_binary).unwrap();
        assert_eq!(program, new_program);
    }
    #[test]
    fn test_file_conversion() {
        use crate::isa::configuration::Program;
        use std::path::Path;
        let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let original_binprog = Path::new(&root_path).join("tests/test1.binprog");
        let original_mnemonic = Path::new(&root_path).join("tests/test1.prog");
        let original_binprog =
            Vec::<u8>::from_binary_prog_file(original_binprog.to_str().unwrap()).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // Converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog = Program::from_binary(&original_binprog).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        // This binprog (PE-Y0X0) was converted from the mnemonic file PE-Y0X0.prog using the convert binary
        // So here we just need to validate the structure loaded from both formats match
        let original_binprog = Path::new(&root_path).join("tests/add_2x2/PE-Y0X0");
        let original_mnemonic = Path::new(&root_path).join("tests/add_2x2/PE-Y0X0.prog");
        let original_binprog = std::fs::read_to_string(original_binprog).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        let original_binprog = original_binprog.replace(" ", "").replace("\n", "");
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog =
            Program::from_binary(&Vec::<u8>::from_binary_str(&original_binprog).unwrap()).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/add_2x2/PE-Y0X1");
        let original_mnemonic = Path::new(&root_path).join("tests/add_2x2/PE-Y0X1.prog");
        let original_binprog = std::fs::read_to_string(original_binprog).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog =
            Program::from_binary(&Vec::<u8>::from_binary_str(&original_binprog).unwrap()).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/add_2x2/PE-Y1X0");
        let original_mnemonic = Path::new(&root_path).join("tests/add_2x2/PE-Y1X0.prog");
        let original_binprog = std::fs::read_to_string(original_binprog).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog =
            Program::from_binary(&Vec::<u8>::from_binary_str(&original_binprog).unwrap()).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/add_2x2/PE-Y1X1");
        let original_mnemonic = Path::new(&root_path).join("tests/add_2x2/PE-Y1X1.prog");
        let original_binprog = std::fs::read_to_string(original_binprog).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog =
            Program::from_binary(&Vec::<u8>::from_binary_str(&original_binprog).unwrap()).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/array_add_2x2/PE-Y0X0");
        let original_mnemonic = Path::new(&root_path).join("tests/array_add_2x2/PE-Y0X0.prog");
        let original_binprog =
            Vec::<u8>::from_binary_prog_file(original_binprog.to_str().unwrap()).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog = Program::from_binary(&original_binprog).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/array_add_2x2/PE-Y0X1");
        let original_mnemonic = Path::new(&root_path).join("tests/array_add_2x2/PE-Y0X1.prog");
        let original_binprog =
            Vec::<u8>::from_binary_prog_file(original_binprog.to_str().unwrap()).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog = Program::from_binary(&original_binprog).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/array_add_2x2/PE-Y1X0");
        let original_mnemonic = Path::new(&root_path).join("tests/array_add_2x2/PE-Y1X0.prog");
        let original_binprog =
            Vec::<u8>::from_binary_prog_file(original_binprog.to_str().unwrap()).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog = Program::from_binary(&original_binprog).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);

        let original_binprog = Path::new(&root_path).join("tests/array_add_2x2/PE-Y1X1");
        let original_mnemonic = Path::new(&root_path).join("tests/array_add_2x2/PE-Y1X1.prog");
        let original_binprog =
            Vec::<u8>::from_binary_prog_file(original_binprog.to_str().unwrap()).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program_from_binprog = Program::from_binary(&original_binprog).unwrap();
        let program_from_mnemonic = Program::from_mnemonics(&original_mnemonic).unwrap();
        assert_eq!(program_from_binprog, program_from_mnemonic);
    }
}
