use pace_sim::{
    self,
    isa::{
        configuration::Configuration,
        operation::*,
        pe::{PE, PERegisters, PESignals},
        router::*,
    },
    sim::dmem::DataMemory,
};

#[test]
fn test_single_pe() {
    // Load one 16b element from 0x10
    // Load one 16b element from 0x20
    // Add the two elements
    // Store the result at 0x30

    // Taking the value from west to alu_op1, because the previous LOAD send data this cycle
    let load_op1 = Configuration {
        operation: Operation {
            op_code: OpCode::LOAD,
            immediate: Some(0x10),
            update_res: NO_UPDATE_RES,
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
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
    };

    // The value of op2 is ready at this cycle, take it from west, but you cannot do the ADD yet, this cycle just load the data to the alu_op2
    let load_op2 = Configuration {
        operation: Operation {
            op_code: OpCode::LOAD,
            immediate: Some(0x20),
            update_res: NO_UPDATE_RES,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
    };

    // NOP just get the data from dmem interface to op2
    let wait_op2 = Configuration {
        operation: Operation {
            op_code: OpCode::NOP,
            immediate: NO_IMMEDIATE,
            update_res: NO_UPDATE_RES,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::Open,
                alu_op2: RouterInDir::ALUOut,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
    };

    // Now add the two elements, store the result in alu_res
    let add = Configuration {
        operation: Operation {
            op_code: OpCode::ADD,
            immediate: NO_IMMEDIATE,
            update_res: NO_UPDATE_RES,
        },
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::Open,
                north_out: RouterInDir::Open,
            },
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
    };

    // Store the result at 0x30
    let store = Configuration {
        operation: Operation {
            op_code: OpCode::STORE,
            immediate: Some(0x30),
            update_res: false,
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
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        },
    };

    let configurations = vec![load_op1, load_op2, wait_op2, add, store];

    let mut pe = PE {
        configurations,
        pc: 0,
        regs: PERegisters::default(),
        signals: PESignals::default(),
        previous_op_is_load: Some(false),
    };

    let mut dmem = DataMemory::new(65536);
    dmem.write16(0x10, 0x11);
    dmem.write16(0x20, 0x22);

    assert_eq!(dmem.read16(0x10), 0x11);
    assert_eq!(dmem.read16(0x20), 0x22);

    while pe.next_conf().is_ok() {
        pe.update_alu_out();
        pe.update_mem(&mut dmem.port1);
        dmem.update_interface();
        pe.update_registers();
    }
}
