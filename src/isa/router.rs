use std::ops::Index;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
pub enum RouterOutDir {
    PredicateOut,
    ALUOp1,
    ALUOp2,
    EastOut,
    SouthOut,
    WestOut,
    NorthOut,
}

impl RouterOutDir {
    pub fn opposite_in_dir(&self) -> RouterInDir {
        match self {
            RouterOutDir::NorthOut => RouterInDir::SouthIn,
            RouterOutDir::SouthOut => RouterInDir::NorthIn,
            RouterOutDir::WestOut => RouterInDir::EastIn,
            RouterOutDir::EastOut => RouterInDir::WestIn,
            _ => panic!("You cannot get the opposite input direction from inside of PE"),
        }
    }
}

impl RouterInDir {
    pub fn is_reg_source(&self) -> bool {
        return self == &RouterInDir::ALUOut || self == &RouterInDir::ALURes;
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

impl Index<RouterOutDir> for RouterSwitchConfig {
    type Output = RouterInDir;

    fn index(&self, dir: RouterOutDir) -> &Self::Output {
        match dir {
            RouterOutDir::PredicateOut => &self.predicate,
            RouterOutDir::ALUOp1 => &self.alu_op1,
            RouterOutDir::ALUOp2 => &self.alu_op2,
            RouterOutDir::EastOut => &self.east_out,
            RouterOutDir::SouthOut => &self.south_out,
            RouterOutDir::WestOut => &self.west_out,
            RouterOutDir::NorthOut => &self.north_out,
        }
    }
}

pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct DirectionsOpt {
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

impl Index<Direction> for DirectionsOpt {
    type Output = bool;

    fn index(&self, dir: Direction) -> &Self::Output {
        match dir {
            Direction::North => &self.north,
            Direction::South => &self.south,
            Direction::West => &self.west,
            Direction::East => &self.east,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct RouterConfig {
    pub switch_config: RouterSwitchConfig,
    pub input_register_used: DirectionsOpt,
    pub input_register_write: DirectionsOpt,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            switch_config: RouterSwitchConfig::default(),
            input_register_used: DirectionsOpt {
                north: false,
                south: false,
                west: false,
                east: false,
            },
            input_register_write: DirectionsOpt::default(),
        }
    }
}

impl RouterConfig {
    /// Determine if the Router is giving data from the internal registers to other PEs
    /// Used to check if need to follow the path for multi-hop paths
    pub fn is_path_source(&self) -> bool {
        let switch_config = self.switch_config;
        return switch_config.north_out.is_reg_source()
            || switch_config.south_out.is_reg_source()
            || switch_config.west_out.is_reg_source()
            || switch_config.east_out.is_reg_source();
    }

    /// Find the output directions that are sources from registers
    pub fn find_outputs_from_reg(&self) -> Vec<RouterOutDir> {
        let mut path_sources = Vec::new();
        if self.switch_config.north_out.is_reg_source() {
            path_sources.push(RouterOutDir::NorthOut);
        }
        if self.switch_config.south_out.is_reg_source() {
            path_sources.push(RouterOutDir::SouthOut);
        }
        if self.switch_config.west_out.is_reg_source() {
            path_sources.push(RouterOutDir::WestOut);
        }
        if self.switch_config.east_out.is_reg_source() {
            path_sources.push(RouterOutDir::EastOut);
        }
        path_sources
    }
}

impl RouterSwitchConfig {
    pub fn find_output_directions(&self, in_direction: RouterInDir) -> Vec<RouterOutDir> {
        let mut output_directions = Vec::new();
        for out_dir in RouterOutDir::iter() {
            if self[out_dir] == in_direction
                && out_dir != RouterOutDir::PredicateOut
                && out_dir != RouterOutDir::ALUOp1
                && out_dir != RouterOutDir::ALUOp2
            {
                output_directions.push(out_dir);
            }
        }
        output_directions
    }
}

impl PE {
    /// Update the operands registers (Predicate, ALU Op1 and Op2)
    pub fn update_operands_registers(
        &mut self,
        router_config: &RouterConfig,
    ) -> Result<(), String> {
        match router_config.switch_config.alu_op1 {
            RouterInDir::EastIn => {
                if router_config.input_register_used.east {
                    self.regs.reg_op1 = self.regs.reg_east_in;
                } else {
                    self.regs.reg_op1 = self.signals.wire_east_in.ok_or("EastIn is not updated")?;
                }
            }
            RouterInDir::SouthIn => {
                if router_config.input_register_used.south {
                    self.regs.reg_op1 = self.regs.reg_south_in;
                } else {
                    self.regs.reg_op1 =
                        self.signals.wire_south_in.ok_or("SouthIn is not updated")?;
                }
            }
            RouterInDir::WestIn => {
                if router_config.input_register_used.west {
                    self.regs.reg_op1 = self.regs.reg_west_in;
                } else {
                    self.regs.reg_op1 = self.signals.wire_west_in.ok_or("WestIn is not updated")?;
                }
            }
            RouterInDir::NorthIn => {
                if router_config.input_register_used.north {
                    self.regs.reg_op1 = self.regs.reg_north_in;
                } else {
                    self.regs.reg_op1 =
                        self.signals.wire_north_in.ok_or("NorthIn is not updated")?;
                }
            }
            RouterInDir::ALUOut => {
                self.regs.reg_op1 = self
                    .signals
                    .wire_alu_out
                    .ok_or("Updating ALU Op1 register but the wire signal is not updated")?;
            }
            RouterInDir::ALURes => {
                self.regs.reg_op1 = self.regs.reg_res;
            }
            RouterInDir::Invalid => unreachable!(),
            RouterInDir::Open => {}
        }
        match router_config.switch_config.alu_op2 {
            RouterInDir::EastIn => {
                if router_config.input_register_used.east {
                    self.regs.reg_op2 = self.regs.reg_east_in;
                } else {
                    self.regs.reg_op2 = self.signals.wire_east_in.ok_or("EastIn is not updated")?;
                }
            }
            RouterInDir::SouthIn => {
                if router_config.input_register_used.south {
                    self.regs.reg_op2 = self.regs.reg_south_in;
                } else {
                    self.regs.reg_op2 =
                        self.signals.wire_south_in.ok_or("SouthIn is not updated")?;
                }
            }
            RouterInDir::WestIn => {
                if router_config.input_register_used.west {
                    self.regs.reg_op2 = self.regs.reg_west_in;
                } else {
                    self.regs.reg_op2 = self.signals.wire_west_in.ok_or("WestIn is not updated")?;
                }
            }
            RouterInDir::NorthIn => {
                if router_config.input_register_used.north {
                    self.regs.reg_op2 = self.regs.reg_north_in;
                } else {
                    self.regs.reg_op2 =
                        self.signals.wire_north_in.ok_or("NorthIn is not updated")?;
                }
            }
            RouterInDir::ALUOut => {
                self.regs.reg_op2 = self
                    .signals
                    .wire_alu_out
                    .ok_or("Updating ALU Op2 register but the wire signal is not updated")?;
            }
            RouterInDir::ALURes => {
                self.regs.reg_op2 = self.regs.reg_res;
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
        Ok(())
    }

    /// Update the outputs (wires) for the router
    pub fn execute_router_output(&mut self, router_config: &RouterConfig) -> Result<(), String> {
        match router_config.switch_config.east_out {
            RouterInDir::EastIn => {
                if router_config.input_register_used.east {
                    self.signals.wire_east_out = Some(self.regs.reg_east_in);
                } else {
                    self.signals.wire_east_out =
                        Some(self.signals.wire_east_in.ok_or("EastIn is not updated")?);
                }
            }
            RouterInDir::SouthIn => {
                if router_config.input_register_used.south {
                    self.signals.wire_east_out = Some(self.regs.reg_south_in);
                } else {
                    self.signals.wire_east_out =
                        Some(self.signals.wire_south_in.ok_or("SouthIn is not updated")?);
                }
            }
            RouterInDir::WestIn => {
                if router_config.input_register_used.west {
                    self.signals.wire_east_out = Some(self.regs.reg_west_in);
                } else {
                    self.signals.wire_east_out =
                        Some(self.signals.wire_west_in.ok_or("WestIn is not updated")?);
                }
            }
            RouterInDir::NorthIn => {
                if router_config.input_register_used.north {
                    self.signals.wire_east_out = Some(self.regs.reg_north_in);
                } else {
                    self.signals.wire_east_out =
                        Some(self.signals.wire_north_in.ok_or("NorthIn is not updated")?);
                }
            }
            RouterInDir::ALUOut => {
                self.signals.wire_east_out =
                    Some(self.signals.wire_alu_out.ok_or("ALUOut is not updated")?);
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
                self.signals.wire_south_out =
                    Some(self.signals.wire_east_in.ok_or("EastIn is not updated")?);
            }
            RouterInDir::SouthIn => {
                self.signals.wire_south_out =
                    Some(self.signals.wire_south_in.ok_or("SouthIn is not updated")?);
            }
            RouterInDir::WestIn => {
                self.signals.wire_south_out =
                    Some(self.signals.wire_west_in.ok_or("WestIn is not updated")?);
            }
            RouterInDir::NorthIn => {
                self.signals.wire_south_out =
                    Some(self.signals.wire_north_in.ok_or("NorthIn is not updated")?);
            }
            RouterInDir::ALUOut => {
                self.signals.wire_south_out =
                    Some(self.signals.wire_alu_out.ok_or("ALUOut is not updated")?);
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
                self.signals.wire_west_out = Some(self.signals.wire_alu_out.unwrap());
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
                self.signals.wire_north_out =
                    Some(self.signals.wire_east_in.ok_or("EastIn is not updated")?);
            }
            RouterInDir::SouthIn => {
                self.signals.wire_north_out =
                    Some(self.signals.wire_south_in.ok_or("SouthIn is not updated")?);
            }
            RouterInDir::WestIn => {
                self.signals.wire_north_out =
                    Some(self.signals.wire_west_in.ok_or("WestIn is not updated")?);
            }
            RouterInDir::NorthIn => {
                self.signals.wire_north_out =
                    Some(self.signals.wire_north_in.ok_or("NorthIn is not updated")?);
            }
            RouterInDir::ALUOut => {
                self.signals.wire_north_out =
                    Some(self.signals.wire_alu_out.ok_or("ALUOut is not updated")?);
            }
            RouterInDir::ALURes => {
                self.signals.wire_north_out = Some(self.regs.reg_res);
            }
            RouterInDir::Open => {
                self.signals.wire_north_out = None;
            }
            RouterInDir::Invalid => unreachable!(),
        }
        Ok(())
    }

    pub fn update_router_input_registers(
        &mut self,
        router_config: &RouterConfig,
    ) -> Result<(), String> {
        if router_config.input_register_write.north {
            self.regs.reg_north_in = self.signals.wire_north_in.ok_or("NorthIn is not updated")?;
        }
        if router_config.input_register_write.south {
            self.regs.reg_south_in = self.signals.wire_south_in.ok_or("SouthIn is not updated")?;
        }
        if router_config.input_register_write.west {
            self.regs.reg_west_in = self.signals.wire_west_in.ok_or("WestIn is not updated")?;
        }
        if router_config.input_register_write.east {
            self.regs.reg_east_in = self.signals.wire_east_in.ok_or("EastIn is not updated")?;
        }
        Ok(())
    }

    /// Update the signals of the current PE from the given PE from the given direction
    pub fn update_router_signals_from(
        &mut self,
        src_pe: &PE,
        direction: RouterInDir,
    ) -> Result<(), String> {
        match direction {
            RouterInDir::NorthIn => {
                self.signals.wire_north_in = Some(
                    src_pe
                        .signals
                        .wire_south_out
                        .ok_or("SouthOut is not updated")?,
                );
            }
            RouterInDir::SouthIn => {
                self.signals.wire_south_in = Some(
                    src_pe
                        .signals
                        .wire_north_out
                        .ok_or("NorthOut is not updated")?,
                );
            }
            RouterInDir::WestIn => {
                self.signals.wire_west_in = Some(
                    src_pe
                        .signals
                        .wire_east_out
                        .ok_or("EastOut is not updated")?,
                );
            }
            RouterInDir::EastIn => {
                self.signals.wire_east_in = Some(
                    src_pe
                        .signals
                        .wire_west_out
                        .ok_or("WestOut is not updated")?,
                );
            }
            _ => panic!("You cannot propagate router signals from inside of PE"),
        }
        Ok(())
    }
}
