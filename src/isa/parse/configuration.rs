pub mod mnemonics {
    use nom::{IResult, character::complete::multispace0};

    use crate::isa::{
        configuration::{Configuration, Program},
        parse::{operation, router},
    };

    pub fn parse_configuration(s: &str) -> IResult<&str, Configuration> {
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

    impl Configuration {
        pub fn from_str(s: &str) -> Result<Self, String> {
            let (_, configuration) = parse_configuration(s).map_err(|e| e.to_string())?;
            Ok(configuration)
        }
    }

    impl Program {
        pub fn from_str(s: &str) -> Result<Self, String> {
            let mut configurations = Vec::new();
            for line in s.lines() {
                let configuration = Configuration::from_str(line)?;
                configurations.push(configuration);
            }
            Ok(Program { configurations })
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
            let (_, configuration) = parse_configuration(input).unwrap();
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
