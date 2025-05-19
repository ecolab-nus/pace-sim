use core::panic;

use super::state::{ExecuteCombinatorial, PESignals, PEState};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RouterInDir {
    EastIn,
    SouthIn,
    WestIn,
    NorthIn,
    ALUOut,
    ALURes,
    Open,
}

#[derive(Debug, Clone, Copy)]
pub struct RouterSwitchConfig {
    pub predicate: RouterInDir,
    pub alu_op1: RouterInDir,
    pub alu_op2: RouterInDir,
    pub east_out: RouterInDir,
    pub south_out: RouterInDir,
    pub west_out: RouterInDir,
    pub north_out: RouterInDir,
}

impl Default for RouterSwitchConfig {
    fn default() -> Self {
        RouterSwitchConfig {
            predicate: RouterInDir::Open,
            alu_op1: RouterInDir::Open,
            alu_op2: RouterInDir::Open,
            east_out: RouterInDir::Open,
            south_out: RouterInDir::Open,
            west_out: RouterInDir::Open,
            north_out: RouterInDir::Open,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterExtraConfig {
    pub input_register_bypass: RouterRegisterBypassConfig,
    pub input_register_write: RouterRegisterWriteConfig,
}

#[derive(Debug, Clone)]
pub struct RouterRegisterBypassConfig {
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

#[derive(Debug, Clone)]
pub struct RouterRegisterWriteConfig {
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub switch_config: RouterSwitchConfig,
    pub extra_config: RouterExtraConfig,
}

impl RouterConfig {
    /// Update the operands registers (Predicate, ALU Op1 and Op2)
    pub fn update_operands_registers(&self, state: &PEState) -> PEState {
        let mut new_state = state.clone();
        match self.switch_config.alu_op1 {
            RouterInDir::EastIn => {
                if self.extra_config.input_register_bypass.east {
                    new_state.regs.reg_op1 = state.signals.wire_east_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = state.regs.reg_east_in;
                }
            }
            RouterInDir::SouthIn => {
                if self.extra_config.input_register_bypass.south {
                    new_state.regs.reg_op1 = state.signals.wire_south_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = state.regs.reg_south_in;
                }
            }
            RouterInDir::WestIn => {
                if self.extra_config.input_register_bypass.west {
                    new_state.regs.reg_op1 = state.signals.wire_west_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = state.regs.reg_west_in;
                }
            }
            RouterInDir::NorthIn => {
                if self.extra_config.input_register_bypass.north {
                    new_state.regs.reg_op1 = state.signals.wire_north_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = state.regs.reg_north_in;
                }
            }
            RouterInDir::ALUOut => {
                new_state.regs.reg_op1 = state.signals.wire_alu_out;
            }
            RouterInDir::ALURes => {
                new_state.regs.reg_op1 = state.regs.reg_res;
            }
            RouterInDir::Open => {
                panic!("ALU Op1 cannot be configured to Open")
            }
        }
        match self.switch_config.alu_op2 {
            RouterInDir::EastIn => {
                new_state.regs.reg_op2 = state.signals.wire_east_in.unwrap();
            }
            RouterInDir::SouthIn => {
                new_state.regs.reg_op2 = state.signals.wire_south_in.unwrap();
            }
            RouterInDir::WestIn => {
                new_state.regs.reg_op2 = state.signals.wire_west_in.unwrap();
            }
            RouterInDir::NorthIn => {
                new_state.regs.reg_op2 = state.signals.wire_north_in.unwrap();
            }
            RouterInDir::ALUOut => {
                new_state.regs.reg_op2 = state.signals.wire_alu_out;
            }
            RouterInDir::ALURes => {
                new_state.regs.reg_op2 = state.regs.reg_res;
            }
            RouterInDir::Open => {
                todo!()
                // TODO ZHENYU: actually ALU Op2 can be configured only in case of using immediate?
            }
        }
        match self.switch_config.predicate {
            _ => todo!(),
        }
        new_state
    }

    /// Update the outputs (wires) for the router
    pub fn update_router_outputs(&self, state: &PEState) -> PEState {
        let mut new_state = state.clone();
        match self.switch_config.east_out {
            RouterInDir::EastIn => {
                if self.extra_config.input_register_bypass.east {
                    new_state.signals.wire_east_out = Some(state.signals.wire_east_in.unwrap());
                } else {
                    new_state.signals.wire_east_out = Some(state.regs.reg_east_in);
                }
            }
            RouterInDir::SouthIn => {
                if self.extra_config.input_register_bypass.south {
                    new_state.signals.wire_east_out = Some(state.signals.wire_south_in.unwrap());
                } else {
                    new_state.signals.wire_east_out = Some(state.regs.reg_south_in);
                }
            }
            RouterInDir::WestIn => {
                if self.extra_config.input_register_bypass.west {
                    new_state.signals.wire_east_out = Some(state.signals.wire_west_in.unwrap());
                } else {
                    new_state.signals.wire_east_out = Some(state.regs.reg_west_in);
                }
            }
            RouterInDir::NorthIn => {
                if self.extra_config.input_register_bypass.north {
                    new_state.signals.wire_east_out = Some(state.signals.wire_north_in.unwrap());
                } else {
                    new_state.signals.wire_east_out = Some(state.regs.reg_north_in);
                }
            }
            RouterInDir::ALUOut => {
                new_state.signals.wire_east_out = Some(state.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                new_state.signals.wire_east_out = Some(state.regs.reg_res);
            }
            RouterInDir::Open => {
                new_state.signals.wire_east_out = None;
            }
        }
        match self.switch_config.south_out {
            RouterInDir::EastIn => {
                new_state.signals.wire_south_out = Some(state.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                new_state.signals.wire_south_out = Some(state.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                new_state.signals.wire_south_out = Some(state.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                new_state.signals.wire_south_out = Some(state.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                new_state.signals.wire_south_out = Some(state.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                new_state.signals.wire_south_out = Some(state.regs.reg_res);
            }
            RouterInDir::Open => {
                new_state.signals.wire_south_out = None;
            }
        }
        match self.switch_config.west_out {
            RouterInDir::EastIn => {
                new_state.signals.wire_west_out = Some(state.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                new_state.signals.wire_west_out = Some(state.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                new_state.signals.wire_west_out = Some(state.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                new_state.signals.wire_west_out = Some(state.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                new_state.signals.wire_west_out = Some(state.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                new_state.signals.wire_west_out = Some(state.regs.reg_res);
            }
            RouterInDir::Open => {
                new_state.signals.wire_west_out = None;
            }
        }
        match self.switch_config.north_out {
            RouterInDir::EastIn => {
                new_state.signals.wire_north_out = Some(state.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                new_state.signals.wire_north_out = Some(state.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                new_state.signals.wire_north_out = Some(state.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                new_state.signals.wire_north_out = Some(state.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                new_state.signals.wire_north_out = Some(state.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                new_state.signals.wire_north_out = Some(state.regs.reg_res);
            }
            RouterInDir::Open => {
                new_state.signals.wire_north_out = None;
            }
        }
        new_state
    }

    pub fn update_router_input_registers(&self, state: &PEState) -> PEState {
        let mut new_state = state.clone();
        if self.extra_config.input_register_write.north {
            new_state.regs.reg_north_in = state.signals.wire_north_in.unwrap();
        }
        if self.extra_config.input_register_write.south {
            new_state.regs.reg_south_in = state.signals.wire_south_in.unwrap();
        }
        if self.extra_config.input_register_write.west {
            new_state.regs.reg_west_in = state.signals.wire_west_in.unwrap();
        }
        if self.extra_config.input_register_write.east {
            new_state.regs.reg_east_in = state.signals.wire_east_in.unwrap();
        }
        new_state
    }
}
