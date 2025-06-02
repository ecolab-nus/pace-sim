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
        binary::{ConfigField, ConfigurationField},
        configuration::Configuration,
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
    }
}
