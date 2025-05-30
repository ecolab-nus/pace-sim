use pace_sim::{
    self,
    isa::{configuration::Configuration, operation::*, router::*, state::PEState},
    sim::{dmem::DataMemory, pe::PE},
};

#[test]
fn test_single_pe() {
    // Load one 16b element from 0x10
    // Load one 16b element from 0x11
    // Add the two elements
    // Store the result at 0x12
    let configuration0 = Configuration {
        operation: Operation::LOAD(Some(0x10)),
        router_config: RouterConfig::default(),
    };
    // Taking the value from west to alu_op1, because the previous LOAD send data this cycle
    let configuration1 = Configuration {
        operation: Operation::LOAD(Some(0x11)),
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::WestIn,
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
        operation: Operation::NOP,
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::Open,
                alu_op2: RouterInDir::WestIn,
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

    // Now add the two elements, store the result in alu_res
    let configuration3 = Configuration {
        operation: Operation::ADD(NO_IMMEDIATE, UPDATE_RES),
        router_config: RouterConfig::default(),
    };

    // Store the result at 0x12
    let configuration4 = Configuration {
        operation: Operation::STORE(Some(0x12)),
        router_config: RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::Open,
                alu_op2: RouterInDir::Open,
                east_out: RouterInDir::Open,
                south_out: RouterInDir::Open,
                west_out: RouterInDir::ALURes,
                north_out: RouterInDir::Open,
            },
            extra_config: RouterExtraConfig::default(),
        },
    };

    let configurations = vec![
        configuration0,
        configuration1,
        configuration2,
        configuration3,
        configuration4,
    ];

    let mut pe = PE {
        state: PEState::default(),
        configurations,
    };

    let mut dmem = DataMemory::default();
}
