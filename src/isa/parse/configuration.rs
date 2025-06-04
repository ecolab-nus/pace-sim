pub mod mnemonics {
    use nom::{IResult, Parser, character::complete::multispace0, multi::separated_list0};

    use crate::isa::{
        configuration::{Configuration, Program},
        parse::{operation, router},
    };

    impl Configuration {
        fn parse_configuration(s: &str) -> IResult<&str, Configuration> {
            let (input, _) = multispace0(s)?;
            let (input, operation) = operation::mnemonics::parse_operation(input)?;
            let (input, _) = multispace0(input)?;
            let (input, router_config) = router::mnemonics::parse_router_config(input)?;
            let (input, _) = multispace0(input)?;
            Ok((
                input,
                Configuration {
                    operation,
                    router_config,
                },
            ))
        }

        pub fn from_mnemonics(s: &str) -> Result<Self, String> {
            let (input, configuration) = Self::parse_configuration(s).map_err(|e| e.to_string())?;
            assert!(input.is_empty(), "Invalid configuration: {}", input);
            Ok(configuration)
        }

        pub fn to_mnemonics(&self) -> String {
            format!(
                "{}\n{}",
                self.operation.to_mnemonics(),
                self.router_config.to_mnemonics()
            )
        }
    }

    impl Program {
        fn parse_program(s: &str) -> IResult<&str, Program> {
            let (input, _) = multispace0(s)?;
            let (input, configurations) =
                separated_list0(multispace0, Configuration::parse_configuration).parse(input)?;
            Ok((input, Program { configurations }))
        }

        pub fn from_mnemonics(s: &str) -> Result<Self, String> {
            let (input, program) = Self::parse_program(s).map_err(|e| e.to_string())?;
            assert!(input.is_empty(), "Invalid program: \n{}", input);
            Ok(program)
        }

        pub fn to_mnemonics(&self) -> String {
            self.configurations
                .iter()
                .map(|c| c.to_mnemonics())
                .collect::<Vec<String>>()
                .join("\n\n")
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::isa::{
            operation::Operation,
            router::{DirectionsOpt, RouterInDir, RouterSwitchConfig},
        };

        use super::*;

        #[test]
        fn test_parse_configuration() {
            let input = r"operation: ADD! 15
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
            let configuration = Configuration::from_mnemonics(input).unwrap();
            assert_eq!(configuration.operation, Operation::ADD(Some(15), true));
            let expected_switch_config = RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::ALURes,
                east_out: RouterInDir::EastIn,
                south_out: RouterInDir::SouthIn,
                west_out: RouterInDir::WestIn,
                north_out: RouterInDir::NorthIn,
            };
            let expected_register_bypass = DirectionsOpt {
                north: true,
                south: true,
                east: false,
                west: false,
            };
            let expected_register_write = DirectionsOpt {
                east: true,
                west: true,
                north: false,
                south: false,
            };
            assert_eq!(
                configuration.router_config.switch_config,
                expected_switch_config
            );
            assert_eq!(
                configuration.router_config.input_register_bypass,
                expected_register_bypass
            );
            assert_eq!(
                configuration.router_config.input_register_write,
                expected_register_write
            );

            let test_str = r"operation: NOP
                                           switch_config: {
                                               Open -> predicate,
                                               Open -> south_out,
                                               Open -> west_out,
                                               Open -> north_out,
                                               ALUOut -> east_out,
                                               EastIn -> alu_op2,
                                               SouthIn -> alu_op1,
                                           };
                                           input_register_bypass: {};
                                           input_register_write: {};
                                           ";
            let _configuration = Configuration::from_mnemonics(test_str).unwrap();
        }
    }
}

pub mod binary {
    use crate::isa::{
        binary::BinaryIO,
        configuration::{Configuration, Program},
        operation::Operation,
        router::RouterConfig,
    };

    impl BinaryIO for Configuration {
        fn to_binary(&self) -> u64 {
            let router_config: u64 = self.router_config.to_binary();
            let operation: u64 = self.operation.to_binary();
            let code = router_config | operation;
            code
        }
        fn from_binary(code: u64) -> Self {
            let router_config = RouterConfig::from_binary(code);
            let operation = Operation::from_binary(code);
            Self {
                router_config,
                operation,
            }
        }

        fn from_binary_str(s: &str) -> Result<Self, String> {
            let code = u64::from_binary_str(s);
            if code.is_err() {
                return Err(format!("Invalid binary string: {}", s));
            }
            let code = code.unwrap();
            Ok(Self::from_binary(code))
        }

        fn to_binary_str(&self) -> String {
            self.to_binary().to_binary_str()
        }
    }

    impl Program {
        fn to_binary(&self) -> Vec<u64> {
            self.configurations.iter().map(|c| c.to_binary()).collect()
        }

        fn from_binary(code: Vec<u64>) -> Result<Self, String> {
            let configurations = code
                .iter()
                .map(|c| Configuration::from_binary(*c))
                .collect();
            Ok(Self { configurations })
        }

        pub fn from_binary_str(s: &str) -> Result<Self, String> {
            // Split the string into lines
            let lines = s.lines().collect::<Vec<&str>>();
            // for each line, remove the whitespace
            let lines = lines.iter().map(|l| l.trim()).collect::<Vec<&str>>();
            // for each line, convert to configuration
            let mut configurations = Vec::new();
            for (line_nb, line) in lines.iter().enumerate() {
                let configuration = Configuration::from_binary_str(line);
                if configuration.is_err() {
                    return Err(format!(
                        "Invalid binary string at line {}: {}",
                        line_nb, line
                    ));
                }
                configurations.push(configuration.unwrap());
            }
            Ok(Self { configurations })
        }

        pub fn to_binary_str(&self) -> String {
            self.to_binary()
                .iter()
                .map(|c| c.to_binary_str())
                .collect::<Vec<String>>()
                .join("\n")
        }
    }

    #[cfg(test)]
    mod tests {
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
        input_register_bypass: {north, south};
        input_register_write: {east, west};",
            )
            .unwrap();
            let binary = configuration.to_binary();
            let configuration_from_binary = Configuration::from_binary(binary);
            assert_eq!(configuration, configuration_from_binary);
        }

        #[test]
        fn test_program_binary_conversions() {
            // Converting from binprog to prog, then back to binprog
            let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_file = Path::new(&root_path).join("tests/test1.binprog");
            let str_program = std::fs::read_to_string(test_file).unwrap();
            let program = Program::from_binary_str(&str_program).unwrap();

            let new_str_program = program.to_binary_str();
            let new_program = Program::from_binary_str(&new_str_program).unwrap();
            assert_eq!(program, new_program);

            // Converting from prog to binprog, then back to prog
            let mnemonic_file = Path::new(&root_path).join("tests/test1.prog");
            let str_program = std::fs::read_to_string(mnemonic_file).unwrap();
            let program = Program::from_mnemonics(&str_program).unwrap();
            let new_str_program = program.to_binary_str();
            let new_program = Program::from_binary_str(&new_str_program).unwrap();
            assert_eq!(program, new_program);
        }
    }
}

mod tests {

    #[test]
    fn test_file_conversion() {
        use crate::isa::configuration::Program;
        use std::path::Path;
        let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let original_binprog = Path::new(&root_path).join("tests/test1.binprog");
        let original_mnemonic = Path::new(&root_path).join("tests/test1.prog");
        let original_binprog = std::fs::read_to_string(original_binprog).unwrap();
        let original_mnemonic = std::fs::read_to_string(original_mnemonic).unwrap();
        // converting from binprog to mnemonic, compare with original mnemonic
        let program = Program::from_binary_str(&original_binprog).unwrap();
        let mnemonic = program.to_mnemonics();
        assert_eq!(mnemonic, original_mnemonic);
        // converting from mnemonic to binprog, compare with original binprog
        let program = Program::from_mnemonics(&original_mnemonic).unwrap();
        let binprog = program.to_binary_str();
        assert_eq!(binprog, original_binprog);
    }
}
