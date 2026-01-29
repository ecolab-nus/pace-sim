use nom::{
    IResult, Parser,
    character::complete::multispace0,
    bytes::complete::tag,
    multi::separated_list0,
};

use crate::isa::{
    configuration::{Configuration, Program},
    mnemonic::operation,
    operation::OpCode,
    router::RouterConfig,
};

/// Parse a comment line starting with "//" (with optional leading whitespace)
fn parse_comment(s: &str) -> IResult<&str, ()> {
    use nom::bytes::complete::take_till;
    
    let (input, _) = multispace0(s)?;
    let (input, _) = tag("//")(input)?;
    // Consume everything until end of line or end of input
    let (input, _) = take_till(|c| c == '\n' || c == '\r')(input)?;
    // Optionally consume the newline character(s)
    let input = if input.starts_with("\r\n") {
        &input[2..]
    } else if input.starts_with('\n') {
        &input[1..]
    } else {
        input
    };
    Ok((input, ()))
}

/// Skip whitespace and comments (lines starting with "//")
/// This handles multiple consecutive comments and whitespace
fn skip_whitespace_and_comments(s: &str) -> IResult<&str, ()> {
    let mut input = s;
    loop {
        // Skip whitespace
        let (new_input, _) = multispace0(input)?;
        
        // Try to parse a comment
        match parse_comment(new_input) {
            Ok((remaining, _)) => {
                // Comment was parsed, continue loop
                input = remaining;
            }
            Err(_) => {
                // No comment found, we're done
                input = new_input;
                break;
            }
        }
    }
    Ok((input, ()))
}

impl Configuration {
    fn parse_configuration(s: &str) -> IResult<&str, Configuration> {
        let (input, _) = skip_whitespace_and_comments(s)?;
        let (input, (operation, agu_trigger)) = operation::parse_operation_with_trigger(input)?;
        let (input, _) = skip_whitespace_and_comments(input)?;
        let (input, router_config) = RouterConfig::parse_router_config(input)?;
        let (input, _) = skip_whitespace_and_comments(input)?;
        Ok((
            input,
            Configuration {
                operation,
                router_config,
                agu_trigger,
            },
        ))
    }

    pub fn from_mnemonics(s: &str) -> Result<Self, String> {
        let (input, configuration) = Self::parse_configuration(s).map_err(|e| e.to_string())?;
        assert!(input.is_empty(), "Invalid configuration: {}", input);
        Ok(configuration)
    }

    pub fn to_mnemonics(&self) -> String {
        // Build operation string with agu_trigger marker
        let op_str = self.operation_to_mnemonics_with_trigger();
        format!(
            "{}\n{}",
            op_str,
            self.router_config.to_mnemonics()
        )
    }

    /// Convert operation to mnemonics, including the ? marker for agu_trigger
    fn operation_to_mnemonics_with_trigger(&self) -> String {
        let op = &self.operation;
        let mut result = String::new();
        result.push_str("operation: ");
        
        if op.op_code == OpCode::JUMP {
            result.push_str("JUMP");
            if self.agu_trigger {
                result.push('?');
            }
            result.push_str(&format!(
                " [{}, {}]",
                op.loop_start.unwrap(),
                op.loop_end.unwrap()
            ));
        } else {
            result.push_str(&op.op_code.to_string());
            // Add flags: ! for update_res, ? for agu_trigger
            if op.update_res {
                result.push('!');
            }
            if self.agu_trigger {
                result.push('?');
            }
            result.push(' ');
            if let Some(imm) = op.immediate {
                result.push_str(&imm.to_string());
            }
        }
        result
    }
}

impl Program {
    fn parse_program(s: &str) -> IResult<&str, Program> {
        let (input, _) = skip_whitespace_and_comments(s)?;
        let (input, configurations) =
            separated_list0(skip_whitespace_and_comments, Configuration::parse_configuration).parse(input)?;
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
    fn test_parse_configuration_with_agu_trigger() {
        // Test configuration with agu_trigger marker (?)
        let input = r"operation: ADD!? 15
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
        assert_eq!(configuration.operation.op_code, OpCode::ADD);
        assert_eq!(configuration.operation.immediate, Some(15));
        assert_eq!(configuration.operation.update_res, true);
        assert_eq!(configuration.agu_trigger, true);

        // Test roundtrip with agu_trigger
        let mnemonic = configuration.to_mnemonics();
        let parsed_back = Configuration::from_mnemonics(&mnemonic).unwrap();
        assert_eq!(parsed_back.agu_trigger, true);
        assert_eq!(parsed_back.operation.update_res, true);
        assert_eq!(parsed_back.operation.immediate, Some(15));

        // Test NOP with agu_trigger
        let input = r"operation: NOP?
            switch_config: {
            Open -> predicate,
            Open -> south_out,
            Open -> west_out,
            Open -> north_out,
            Open -> east_out,
            Open -> alu_op2,
            Open -> alu_op1,
        };
        input_register_used: {};
        input_register_write: {};";
        let configuration = Configuration::from_mnemonics(input).unwrap();
        assert_eq!(configuration.operation.op_code, OpCode::NOP);
        assert_eq!(configuration.agu_trigger, true);

        // Test without agu_trigger (should be false)
        let input = r"operation: ADD! 15
            switch_config: {
            Open -> predicate,
            Open -> south_out,
            Open -> west_out,
            Open -> north_out,
            Open -> east_out,
            Open -> alu_op2,
            Open -> alu_op1,
        };
        input_register_used: {};
        input_register_write: {};";
        let configuration = Configuration::from_mnemonics(input).unwrap();
        assert_eq!(configuration.agu_trigger, false);
    }

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
        assert_eq!(configuration.agu_trigger, false);
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

    #[test]
    fn test_parse_program_with_comments() {
        // Test that comments (lines starting with "//") are ignored
        let input = r"// This is a comment
operation: NOP
// Another comment
switch_config: {
    Open -> predicate,
    Open -> south_out,
    Open -> west_out,
    Open -> north_out,
    Open -> east_out,
    Open -> alu_op2,
    Open -> alu_op1,
};
input_register_used: {};
input_register_write: {};

// Comment between configurations
operation: ADD! 15
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
input_register_write: {east, west};
// Trailing comment";
        let program = Program::from_mnemonics(input).unwrap();
        assert_eq!(program.configurations.len(), 2);
        assert_eq!(program.configurations[0].operation.op_code, OpCode::NOP);
        assert_eq!(program.configurations[1].operation.op_code, OpCode::ADD);
        assert_eq!(program.configurations[1].operation.immediate, Some(15));
    }
}
