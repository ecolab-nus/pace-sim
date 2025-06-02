pub mod mnemonics {
    use nom::{IResult, Parser, character::complete::multispace0, multi::separated_list0};

    use crate::isa::{
        configuration::{Configuration, Program},
        parse::{operation, router},
    };

    impl Configuration {
        fn parse_configuration(s: &str) -> IResult<&str, Configuration> {
            let (input, operation) = operation::mnemonics::parse_operation(s)?;
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
            let (_, configuration) = Self::parse_configuration(s).map_err(|e| e.to_string())?;
            Ok(configuration)
        }

        pub fn to_mnemonics(&self) -> String {
            format!(
                "operation: {:?}\nswitch_config: {}\ninput_register_bypass: {}\ninput_register_write: {}",
                self.operation.to_mnemonics(),
                self.router_config.switch_config.to_mnemonics(),
                self.router_config.input_register_bypass.to_mnemonics(),
                self.router_config.input_register_write.to_mnemonics()
            )
        }
    }

    impl Program {
        fn parse_program(s: &str) -> IResult<&str, Program> {
            let (input, configurations) =
                separated_list0(multispace0, Configuration::parse_configuration).parse(s)?;
            Ok((input, Program { configurations }))
        }

        pub fn from_mnemonics(s: &str) -> Result<Self, String> {
            let (_, program) = Self::parse_program(s).map_err(|e| e.to_string())?;
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

        pub fn from_binary_str(s: &str) -> Self {
            let code = u64::from_binary_str(s);
            Self::from_binary(code)
        }

        pub fn to_binary_str(&self) -> String {
            self.to_binary().to_binary_str()
        }
    }

    impl Program {
        pub fn to_binary(&self) -> Vec<u64> {
            self.configurations.iter().map(|c| c.to_binary()).collect()
        }
        pub fn from_binary(code: Vec<u64>) -> Self {
            let configurations = code
                .iter()
                .map(|c| Configuration::from_binary(*c))
                .collect();
            Self { configurations }
        }

        pub fn from_binary_str(s: &str) -> Self {
            // Split the string into lines
            let lines = s.lines().collect::<Vec<&str>>();
            // for each line, remove the whitespace
            let lines = lines.iter().map(|l| l.trim()).collect::<Vec<&str>>();
            // for each line, convert to configuration
            let configurations = lines
                .iter()
                .map(|l| Configuration::from_binary_str(l))
                .collect();
            Self { configurations }
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
            let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let test_file = Path::new(&root_path).join("tests/test1.binprog");
            let str_program = std::fs::read_to_string(test_file).unwrap();
            let program = Program::from_binary_str(&str_program);
            // output a mnemonic version to a file
            let mnemonic_file = Path::new(&root_path).join("tests/test1.prog");
            std::fs::write(mnemonic_file, program.to_mnemonics()).unwrap();
            let new_str_program = program.to_binary_str();
            let new_program = Program::from_binary_str(&new_str_program);
            assert_eq!(program, new_program);
        }
    }
}
