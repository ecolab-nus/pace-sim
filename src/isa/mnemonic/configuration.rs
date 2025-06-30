use nom::{IResult, Parser, character::complete::multispace0, multi::separated_list0};

use crate::isa::{
    configuration::{Configuration, Program},
    mnemonic::operation,
    router::RouterConfig,
};

impl Configuration {
    fn parse_configuration(s: &str) -> IResult<&str, Configuration> {
        let (input, _) = multispace0(s)?;
        let (input, operation) = operation::parse_operation(input)?;
        let (input, _) = multispace0(input)?;
        let (input, router_config) = RouterConfig::parse_router_config(input)?;
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
        operation::{OpCode, Operation, UPDATE_RES},
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
        input_register_used: {north, south};
        input_register_write: {east, west};";
        let configuration = Configuration::from_mnemonics(input).unwrap();
        assert_eq!(
            configuration.operation,
            Operation {
                op_code: OpCode::ADD,
                immediate: Some(15),
                update_res: UPDATE_RES,
                loop_start: None,
                loop_end: None,
            }
        );
        let expected_switch_config = RouterSwitchConfig {
            predicate: RouterInDir::Open,
            alu_op1: RouterInDir::ALUOut,
            alu_op2: RouterInDir::ALURes,
            east_out: RouterInDir::EastIn,
            south_out: RouterInDir::SouthIn,
            west_out: RouterInDir::WestIn,
            north_out: RouterInDir::NorthIn,
        };
        let expected_register_used = DirectionsOpt {
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
            configuration.router_config.input_register_used,
            expected_register_used
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
                                           input_register_used: {};
                                           input_register_write: {};
                                           ";
        let _configuration = Configuration::from_mnemonics(test_str).unwrap();

        let input = r"operation: NOP 
switch_config: {
    Open -> predicate,
    Open -> south_out,
    Open -> west_out,
    Open -> north_out,
    ALUOut -> east_out,
    EastIn -> alu_op2,
    SouthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};

operation: ADD 
switch_config: {
    Open -> predicate,
    ALUOut -> south_out,
    Open -> west_out,
    Open -> north_out,
    Open -> east_out,
    WestIn -> alu_op2,
    NorthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};

operation: SUB 
switch_config: {
    Open -> predicate,
    Open -> south_out,
    ALUOut -> west_out,
    Open -> north_out,
    Open -> east_out,
    EastIn -> alu_op2,
    SouthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};

operation: MULT 
switch_config: {
    Open -> predicate,
    Open -> south_out,
    Open -> west_out,
    ALUOut -> north_out,
    Open -> east_out,
    WestIn -> alu_op2,
    NorthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};

operation: MULT 
switch_config: {
    Open -> predicate,
    Open -> south_out,
    Open -> west_out,
    Open -> north_out,
    ALUOut -> east_out,
    EastIn -> alu_op2,
    SouthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};

operation: ADD 
switch_config: {
    Open -> predicate,
    ALUOut -> south_out,
    Open -> west_out,
    Open -> north_out,
    Open -> east_out,
    WestIn -> alu_op2,
    NorthIn -> alu_op1,
};
input_register_used: {};
input_register_write: {};";
        let program = Program::from_mnemonics(input).unwrap();
        let mnemonic = program.to_mnemonics();
        assert_eq!(mnemonic, input);
    }
}
