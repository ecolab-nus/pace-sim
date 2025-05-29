use super::router::RouterConfig;

#[derive(Debug, Clone, Copy)]
pub struct PERegisters {
    pub reg_north_in: u64,
    pub reg_south_in: u64,
    pub reg_west_in: u64,
    pub reg_east_in: u64,
    pub reg_op1: u64,
    pub reg_op2: u64,
    pub reg_res: u64,
    pub reg_predicate: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PESignals {
    pub wire_alu_out: u64,
    pub wire_north_in: Option<u64>,
    pub wire_south_in: Option<u64>,
    pub wire_west_in: Option<u64>,
    pub wire_east_in: Option<u64>,
    pub wire_north_out: Option<u64>,
    pub wire_south_out: Option<u64>,
    pub wire_west_out: Option<u64>,
    pub wire_east_out: Option<u64>,
    pub wire_dmem_addr: Option<u64>,
    pub wire_dmem_data: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PEState {
    pub regs: PERegisters,
    pub signals: PESignals,
}

impl Default for PEState {
    fn default() -> Self {
        PEState {
            regs: PERegisters::default(),
            signals: PESignals::default(),
        }
    }
}

// impl PEState {
//     pub fn get_lhs(&self) -> i64 {
//         match self.router_config.alu_lhs {
//             RouterInDir::ALURes => self.regs.reg_alu_out,
//             RouterInDir::ALUReg => self.regs.reg_alu_buf,
//             RouterInDir::NorthIn => self.regs.reg_north_in,
//             RouterInDir::SouthIn => self.regs.reg_south_in,
//             RouterInDir::WestIn => self.regs.reg_west_in,
//             RouterInDir::EastIn => self.regs.reg_east_in,
//             RouterInDir::Open => panic!("ALU LHS used but the router is not configured"),
//         }
//     }

//     pub fn get_rhs(&self) -> i64 {
//         match self.router_config.alu_rhs {
//             RouterInDir::ALURes => self.regs.reg_alu_out,
//             RouterInDir::ALUReg => self.regs.reg_alu_buf,
//             RouterInDir::NorthIn => self.regs.reg_north_in,
//             RouterInDir::SouthIn => self.regs.reg_south_in,
//             RouterInDir::WestIn => self.regs.reg_west_in,
//             RouterInDir::EastIn => self.regs.reg_east_in,
//             RouterInDir::Open => panic!("ALU RHS used but the router is not configured"),
//         }
//     }
// }

impl Default for PERegisters {
    fn default() -> Self {
        PERegisters {
            reg_north_in: 0,
            reg_south_in: 0,
            reg_west_in: 0,
            reg_east_in: 0,
            reg_op1: 0,
            reg_op2: 0,
            reg_res: 0,
            reg_predicate: false,
        }
    }
}

impl Default for PESignals {
    fn default() -> Self {
        PESignals {
            wire_north_in: None,
            wire_south_in: None,
            wire_west_in: None,
            wire_east_in: None,
            wire_north_out: None,
            wire_south_out: None,
            wire_west_out: None,
            wire_east_out: None,
            wire_alu_out: 0,
            wire_dmem_addr: None,
            wire_dmem_data: None,
        }
    }
}
