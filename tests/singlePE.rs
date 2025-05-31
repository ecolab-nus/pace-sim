use pace_sim::{
    self,
    isa::{configuration::Configuration, operation::*, router::*, state::PEState},
    sim::{dmem::DataMemory, pe::PE},
};

#[test]
fn test_single_pe() {
    // Load one 16b element from 0x10
    // Load one 16b element from 0x20
    // Add the two elements
    // Store the result at 0x30

    // Taking the value from west to alu_op1, because the previous LOAD send data this cycle
    let configuration1 = Configuration {
        operation: Operation::LOAD(Some(0x10)),
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
            extra_config: RouterExtraConfig {
                input_register_bypass: DirectionsOpt {
                    north: false,
                    south: false,
                    east: false,
                    west: false,
                },
                input_register_write: DirectionsOpt {
                    east: false,
                    west: false,
                    north: false,
                    south: false,
                },
            },
        },
    };

    // The value of op2 is ready at this cycle, take it from west, but you cannot do the ADD yet, this cycle just load the data to the alu_op2
    let configuration2 = Configuration {
        operation: Operation::LOAD(Some(0x20)),
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
            extra_config: RouterExtraConfig {
                input_register_bypass: DirectionsOpt::default(),
                input_register_write: DirectionsOpt::default(),
            },
        },
    };

    // Now add the two elements, store the result in alu_res
    let configuration3 = Configuration {
        operation: Operation::ADD(NO_IMMEDIATE, UPDATE_RES),
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
            extra_config: RouterExtraConfig {
                input_register_bypass: DirectionsOpt::default(),
                input_register_write: DirectionsOpt::default(),
            },
        },
    };

    // Store the result at 0x30
    let configuration4 = Configuration {
        operation: Operation::STORE(Some(0x30)),
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
            extra_config: RouterExtraConfig::default(),
        },
    };

    let configurations = vec![
        configuration1,
        configuration2,
        configuration3,
        configuration4,
    ];

    let mut pe = PE {
        state: PEState::default(),
        configurations,
        pc: 0,
        is_mem: true,
    };

    let mut dmem = DataMemory::new(65536);
    dmem.write16(0x10, 0x11);
    dmem.write16(0x20, 0x22);

    assert_eq!(dmem.read16(0x10), 0x11);
    assert_eq!(dmem.read16(0x20), 0x22);

    loop {
        pe.execute_combinatorial().unwrap();
        pe.execute_memory(&mut dmem).unwrap();
        pe.update_registers().unwrap();
        if pe.next_conf().is_err() {
            break;
        }
        println!(
            "op1: {:?}, op2: {:?}, res: {:?}",
            pe.state.regs.reg_op1, pe.state.regs.reg_op2, pe.state.regs.reg_res
        );
    }

    assert_eq!(dmem.read16(0x30), 0x33);
}
