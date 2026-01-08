use pace_sim::{
    self,
    isa::{configuration::Configuration, operation::*, pe::*, router::*},
};

/// Test a single PE doing ALU operations (ADD, SUB, MULT)
/// Note: Memory operations (LOAD/STORE) now require AGU and are tested separately.
#[test]
fn test_single_pe_alu() {
    // Test sequence: Initialize loop, perform some ALU operations

    // initialize the loop
    let init_loop = Configuration {
        operation: Operation {
            op_code: OpCode::JUMP,
            immediate: Some(1),
            update_res: NO_UPDATE_RES,
            loop_start: Some(1),
            loop_end: Some(4),
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::Open,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_used: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
        agu_trigger: false,
    };

    // ADD with immediate: result = op1 + 10 (op1 starts at 0)
    let add_imm = Configuration {
        operation: Operation {
            op_code: OpCode::ADD,
            immediate: Some(10),
            update_res: UPDATE_RES,
            loop_start: None,
            loop_end: None,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALURes, // Use previous result as op1
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_used: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
        agu_trigger: false,
    };

    // MULT with immediate: result = op1 * 2
    let mult_imm = Configuration {
        operation: Operation {
            op_code: OpCode::MULT,
            immediate: Some(2),
            update_res: UPDATE_RES,
            loop_start: None,
            loop_end: None,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALURes,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_used: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
        agu_trigger: false,
    };

    // SUB with immediate: result = op1 - 5
    let sub_imm = Configuration {
        operation: Operation {
            op_code: OpCode::SUB,
            immediate: Some(5),
            update_res: UPDATE_RES,
            loop_start: None,
            loop_end: None,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALURes,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_used: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
        agu_trigger: false,
    };

    // NOP at end
    let nop = Configuration {
        operation: Operation {
            op_code: OpCode::NOP,
            immediate: NO_IMMEDIATE,
            update_res: NO_UPDATE_RES,
            loop_start: None,
            loop_end: None,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::Open,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_used: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
        agu_trigger: false,
    };

    let configurations = vec![init_loop, add_imm, mult_imm, sub_imm, nop];

    let mut pe = PE {
        configurations: configurations,
        pc: 0,
        regs: PERegisters::default(),
        signals: PESignals::default(),
        previous_op_is_load: None, // Not a memory PE
        previous_op: None,
    };

    // Run simulation
    for cycle in 0..5 {
        pe.update_alu_out();
        pe.update_registers(None).expect("PEUpdateError");
        println!("Cycle {}: PC={}, reg_res={}", cycle, pe.pc, pe.regs.reg_res);
        if pe.pc >= 4 {
            break;
        }
        pe.next_conf();
    }

    // After: 0 + 10 = 10, 10 * 2 = 20, 20 - 5 = 15
    assert_eq!(pe.regs.reg_res, 15, "Expected final result to be 15");
}
