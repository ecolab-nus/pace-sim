use crate::isa::router::{RouterInDir, RouterSwitchConfig};

pub type Register = usize;

const N_REGS: usize = 12;
const REGS: [&str; N_REGS] = [
    "ALUOut", "ALUReg", "SIMDOut", "NorthIn", "SouthIn", "WestIn", "EastIn", "NorthOut",
    "SouthOut", "WestOut", "EastOut", "ALURes",
];

#[derive(Debug, Clone, Copy)]
pub struct RegisterFile {
    pub reg_north_in: i64,
    pub reg_south_in: i64,
    pub reg_west_in: i64,
    pub reg_east_in: i64,
    pub reg_north_out: i64,
    pub reg_south_out: i64,
    pub reg_west_out: i64,
    pub reg_east_out: i64,
    pub reg_alu_out: i64,
    pub reg_alu_buf: i64,
    pub pc: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PESignals {
    pub wire_north_in: i64,
    pub wire_south_in: i64,
    pub wire_west_in: i64,
    pub wire_east_in: i64,
    pub wire_north_out: i64,
    pub wire_south_out: i64,
    pub wire_west_out: i64,
    pub wire_east_out: i64,
    pub wire_alu_out: i64,
}
#[derive(Debug, Clone, Copy)]
pub struct PEState {
    pub regs: RegisterFile,
    pub router_config: RouterSwitchConfig,
    pub signals: PESignals,
}

impl Default for PEState {
    fn default() -> Self {
        PEState {
            regs: RegisterFile::default(),
            router_config: RouterSwitchConfig::default(),
            signals: PESignals::default(),
        }
    }
}

impl PEState {
    pub fn get_lhs(&self) -> i64 {
        match self.router_config.alu_lhs {
            RouterInDir::ALURes => self.regs.reg_alu_out,
            RouterInDir::ALUReg => self.regs.reg_alu_buf,
            RouterInDir::NorthIn => self.regs.reg_north_in,
            RouterInDir::SouthIn => self.regs.reg_south_in,
            RouterInDir::WestIn => self.regs.reg_west_in,
            RouterInDir::EastIn => self.regs.reg_east_in,
            RouterInDir::Open => panic!("ALU LHS used but the router is not configured"),
        }
    }

    pub fn get_rhs(&self) -> i64 {
        match self.router_config.alu_rhs {
            RouterInDir::ALURes => self.regs.reg_alu_out,
            RouterInDir::ALUReg => self.regs.reg_alu_buf,
            RouterInDir::NorthIn => self.regs.reg_north_in,
            RouterInDir::SouthIn => self.regs.reg_south_in,
            RouterInDir::WestIn => self.regs.reg_west_in,
            RouterInDir::EastIn => self.regs.reg_east_in,
            RouterInDir::Open => panic!("ALU RHS used but the router is not configured"),
        }
    }
}

pub trait Executable {
    fn execute(&self, state: &PEState) -> PEState;
}
impl Default for RegisterFile {
    fn default() -> Self {
        RegisterFile {
            reg_north_in: 0,
            reg_south_in: 0,
            reg_west_in: 0,
            reg_east_in: 0,
            reg_north_out: 0,
            reg_south_out: 0,
            reg_west_out: 0,
            reg_east_out: 0,
            reg_alu_out: 0,
            reg_alu_buf: 0,
            pc: 0,
        }
    }
}

impl Default for PESignals {
    fn default() -> Self {
        PESignals {
            wire_north_in: 0,
            wire_south_in: 0,
            wire_west_in: 0,
            wire_east_in: 0,
            wire_north_out: 0,
            wire_south_out: 0,
            wire_west_out: 0,
            wire_east_out: 0,
            wire_alu_out: 0,
        }
    }
}
