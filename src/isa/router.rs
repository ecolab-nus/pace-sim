use super::pe::PE;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum RouterInDir {
    EastIn = 0,  // 0
    SouthIn = 1, // 1
    WestIn = 2,  // 2
    NorthIn = 3, // 3
    ALUOut = 4,  // 4
    ALURes = 5,  // 5
    Invalid = 6, // 6
    Open = 7,    // 7
}

impl Default for RouterInDir {
    fn default() -> Self {
        RouterInDir::Open
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct RouterSwitchConfig {
    pub predicate: RouterInDir,
    pub alu_op2: RouterInDir,
    pub alu_op1: RouterInDir,
    pub north_out: RouterInDir,
    pub west_out: RouterInDir,
    pub south_out: RouterInDir,
    pub east_out: RouterInDir,
}

pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct DirectionsOpt {
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RouterConfig {
    pub switch_config: RouterSwitchConfig,
    pub input_register_bypass: DirectionsOpt,
    pub input_register_write: DirectionsOpt,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            switch_config: RouterSwitchConfig::default(),
            input_register_bypass: DirectionsOpt::default(),
            input_register_write: DirectionsOpt::default(),
        }
    }
}

impl PE {
    /// Update the operands registers (Predicate, ALU Op1 and Op2)
    pub fn update_operands_registers(&self, router_config: &RouterConfig) -> PE {
        let mut new_state = self.clone();
        match router_config.switch_config.alu_op1 {
            RouterInDir::EastIn => {
                if router_config.input_register_bypass.east {
                    new_state.regs.reg_op1 = self.signals.wire_east_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = self.regs.reg_east_in;
                }
            }
            RouterInDir::SouthIn => {
                if router_config.input_register_bypass.south {
                    new_state.regs.reg_op1 = self.signals.wire_south_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = self.regs.reg_south_in;
                }
            }
            RouterInDir::WestIn => {
                if router_config.input_register_bypass.west {
                    new_state.regs.reg_op1 = self.signals.wire_west_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = self.regs.reg_west_in;
                }
            }
            RouterInDir::NorthIn => {
                if router_config.input_register_bypass.north {
                    new_state.regs.reg_op1 = self.signals.wire_north_in.unwrap();
                } else {
                    new_state.regs.reg_op1 = self.regs.reg_north_in;
                }
            }
            RouterInDir::ALUOut => {
                new_state.regs.reg_op1 = self.signals.wire_alu_out;
            }
            RouterInDir::ALURes => {
                new_state.regs.reg_op1 = self.regs.reg_res;
            }
            RouterInDir::Invalid => unreachable!(),
            RouterInDir::Open => {}
        }
        match router_config.switch_config.alu_op2 {
            RouterInDir::EastIn => {
                new_state.regs.reg_op2 = self.signals.wire_east_in.unwrap();
            }
            RouterInDir::SouthIn => {
                new_state.regs.reg_op2 = self.signals.wire_south_in.unwrap();
            }
            RouterInDir::WestIn => {
                new_state.regs.reg_op2 = self.signals.wire_west_in.unwrap();
            }
            RouterInDir::NorthIn => {
                new_state.regs.reg_op2 = self.signals.wire_north_in.unwrap();
            }
            RouterInDir::ALUOut => {
                new_state.regs.reg_op2 = self.signals.wire_alu_out;
            }
            RouterInDir::ALURes => {
                new_state.regs.reg_op2 = self.regs.reg_res;
            }
            RouterInDir::Invalid => unreachable!(),
            RouterInDir::Open => {}
        }
        match router_config.switch_config.predicate {
            RouterInDir::EastIn => {
                todo!()
            }
            RouterInDir::SouthIn => {
                todo!()
            }
            RouterInDir::WestIn => {
                todo!()
            }
            RouterInDir::NorthIn => {
                todo!()
            }
            RouterInDir::ALUOut => {
                todo!()
            }
            RouterInDir::ALURes => {
                todo!()
            }
            RouterInDir::Invalid => unreachable!(),
            RouterInDir::Open => {}
        }
        new_state
    }

    /// Update the outputs (wires) for the router
    pub fn update_router_output_signals(&mut self, router_config: &RouterConfig) {
        match router_config.switch_config.east_out {
            RouterInDir::EastIn => {
                if router_config.input_register_bypass.east {
                    self.signals.wire_east_out = Some(self.signals.wire_east_in.unwrap());
                } else {
                    self.signals.wire_east_out = Some(self.regs.reg_east_in);
                }
            }
            RouterInDir::SouthIn => {
                if router_config.input_register_bypass.south {
                    self.signals.wire_east_out = Some(self.signals.wire_south_in.unwrap());
                } else {
                    self.signals.wire_east_out = Some(self.regs.reg_south_in);
                }
            }
            RouterInDir::WestIn => {
                if router_config.input_register_bypass.west {
                    self.signals.wire_east_out = Some(self.signals.wire_west_in.unwrap());
                } else {
                    self.signals.wire_east_out = Some(self.regs.reg_west_in);
                }
            }
            RouterInDir::NorthIn => {
                if router_config.input_register_bypass.north {
                    self.signals.wire_east_out = Some(self.signals.wire_north_in.unwrap());
                } else {
                    self.signals.wire_east_out = Some(self.regs.reg_north_in);
                }
            }
            RouterInDir::ALUOut => {
                self.signals.wire_east_out = Some(self.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                self.signals.wire_east_out = Some(self.regs.reg_res);
            }
            RouterInDir::Open => {
                self.signals.wire_east_out = None;
            }
            RouterInDir::Invalid => unreachable!(),
        }
        match router_config.switch_config.south_out {
            RouterInDir::EastIn => {
                self.signals.wire_south_out = Some(self.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                self.signals.wire_south_out = Some(self.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                self.signals.wire_south_out = Some(self.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                self.signals.wire_south_out = Some(self.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                self.signals.wire_south_out = Some(self.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                self.signals.wire_south_out = Some(self.regs.reg_res);
            }
            RouterInDir::Open => {
                self.signals.wire_south_out = None;
            }
            RouterInDir::Invalid => unreachable!(),
        }
        match router_config.switch_config.west_out {
            RouterInDir::EastIn => {
                self.signals.wire_west_out = Some(self.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                self.signals.wire_west_out = Some(self.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                self.signals.wire_west_out = Some(self.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                self.signals.wire_west_out = Some(self.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                self.signals.wire_west_out = Some(self.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                self.signals.wire_west_out = Some(self.regs.reg_res);
            }
            RouterInDir::Open => {
                self.signals.wire_west_out = None;
            }
            RouterInDir::Invalid => unreachable!(),
        }
        match router_config.switch_config.north_out {
            RouterInDir::EastIn => {
                self.signals.wire_north_out = Some(self.signals.wire_east_in.unwrap());
            }
            RouterInDir::SouthIn => {
                self.signals.wire_north_out = Some(self.signals.wire_south_in.unwrap());
            }
            RouterInDir::WestIn => {
                self.signals.wire_north_out = Some(self.signals.wire_west_in.unwrap());
            }
            RouterInDir::NorthIn => {
                self.signals.wire_north_out = Some(self.signals.wire_north_in.unwrap());
            }
            RouterInDir::ALUOut => {
                self.signals.wire_north_out = Some(self.signals.wire_alu_out);
            }
            RouterInDir::ALURes => {
                self.signals.wire_north_out = Some(self.regs.reg_res);
            }
            RouterInDir::Open => {
                self.signals.wire_north_out = None;
            }
            RouterInDir::Invalid => unreachable!(),
        }
    }

    pub fn update_router_input_registers(&self, router_config: &RouterConfig) -> PE {
        let mut new_state = self.clone();
        if router_config.input_register_write.north {
            new_state.regs.reg_north_in = self.signals.wire_north_in.unwrap();
        }
        if router_config.input_register_write.south {
            new_state.regs.reg_south_in = self.signals.wire_south_in.unwrap();
        }
        if router_config.input_register_write.west {
            new_state.regs.reg_west_in = self.signals.wire_west_in.unwrap();
        }
        if router_config.input_register_write.east {
            new_state.regs.reg_east_in = self.signals.wire_east_in.unwrap();
        }
        new_state
    }
}
