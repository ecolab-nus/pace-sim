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
    fn set_bool_field(&mut self, field: ConfigField, value: &bool);
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

    fn set_bool_field(&mut self, field: ConfigField, value: &bool) {
        assert_eq!(
            field.get_range().1 - field.get_range().0,
            1,
            "Field is not a single bit"
        );
        self.set_field(field, *value as u32);
    }
}

// impl BinaryIO for Program {
//     fn to_binary_str(&self) -> String {
//         self.configurations
//             .iter()
//             .map(|c| c.to_binary().to_binary_str())
//             .collect::<Vec<String>>()
//             .join("\n")
//     }

//     fn from_binary_str(s: &str) -> Result<Self, String> {
//         // split by newline, remove spaces
//         let lines = s.lines().map(|l| l.trim()).collect::<Vec<&str>>();
//         // for each line, convert to binary
//         let mut configurations = Vec::new();
//         for (line_nb, line) in lines.iter().enumerate() {
//             let code = ConfigCode::from_binary_str(line);
//             if code.is_err() {
//                 return Err(format!(
//                     "Invalid binary string at line {}, {}",
//                     line_nb, line
//                 ));
//             }
//             let code = code.unwrap();
//             let configuration = Configuration::from_binary(code);
//             if configuration.is_err() {
//                 return Err(format!(
//                     "Invalid configuration at line {}, {}",
//                     line_nb, line
//                 ));
//             }
//             let configuration = configuration.unwrap();
//             configurations.push(configuration);
//         }
//         // convert to program
//         Ok(Self { configurations })
//     }

//     fn to_binary(&self) -> u64 {
//         todo!()
//     }

//     fn from_binary(_: u64) -> Self {
//         todo!()
//     }
// }

pub trait BinaryIO {
    fn to_binary_str(&self) -> String;
    fn to_binary(&self) -> ConfigCode;
    fn from_binary_str(s: &str) -> Result<Self, String>
    where
        Self: Sized;
    fn from_binary(code: u64) -> Self;
}

impl BinaryIO for u64 {
    fn to_binary_str(&self) -> String {
        format!("{:064b}", self)
    }

    fn to_binary(&self) -> u64 {
        *self
    }

    fn from_binary_str(s: &str) -> Result<Self, String> {
        let mut code: u64 = 0;
        let s = s.chars().enumerate().collect::<Vec<(usize, char)>>();
        if s.len() != 64 {
            return Err(format!("Invalid binary string length: {}", s.len()));
        }
        for (i, c) in s {
            if c == '1' {
                code |= 1 << (63 - i);
            }
        }
        Ok(code)
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
        let code = 0b1010101010101010101010101010101010101010101010101010101010101010;
        let code_str = code.to_binary_str();
        let code_binary = code.to_binary();
        assert_eq!(
            code_str,
            "1010101010101010101010101010101010101010101010101010101010101010"
        );
        assert_eq!(
            code_binary,
            0b1010101010101010101010101010101010101010101010101010101010101010
        );
        let code_from_str = ConfigCode::from_binary_str(&code_str);
        let code_from_binary = ConfigCode::from_binary(code_binary);
        assert_eq!(code_from_str, Ok(code));
        assert_eq!(code_from_binary, code);
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
        let configuration = Configuration::from_mnemonics(configuration_str).unwrap();
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
