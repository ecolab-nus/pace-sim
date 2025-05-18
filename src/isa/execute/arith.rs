use crate::isa::state::{Executable, PEState};

pub struct Add {}
impl Executable for Add {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs + rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct Sub {}
impl Executable for Sub {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs - rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct Mult {}
impl Executable for Mult {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs * rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

// TODO: Implement SEXT
pub struct SEXT {}
impl Executable for SEXT {
    fn execute(&self, _state: &PEState) -> PEState {
        todo!()
    }
}

pub struct Div {}
impl Executable for Div {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs / rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

// TODO: Implement VADD
pub struct VADD {}
impl Executable for VADD {
    fn execute(&self, _state: &PEState) -> PEState {
        todo!()
    }
}

// TODO: Implement VMUL
pub struct VMUL {}
impl Executable for VMUL {
    fn execute(&self, _state: &PEState) -> PEState {
        todo!()
    }
}

pub struct LS {}
impl Executable for LS {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs() as u64;
        // TODO check if this is correct
        let rhs = new_state.get_rhs() as u32;

        new_state.regs.reg_alu_out = (lhs << rhs) as i64;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct RS {}
impl Executable for RS {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs() as u64;
        // TODO check if this is correct
        let rhs = new_state.get_rhs() as u32;

        new_state.regs.reg_alu_out = (lhs >> rhs) as i64;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct ASR {}
impl Executable for ASR {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs >> rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct AND {}
impl Executable for AND {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs & rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct OR {}
impl Executable for OR {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs | rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct XOR {}
impl Executable for XOR {
    fn execute(&self, state: &PEState) -> PEState {
        let mut new_state: PEState = state.clone();

        let lhs = new_state.get_lhs();
        let rhs = new_state.get_rhs();

        new_state.regs.reg_alu_out = lhs ^ rhs;
        new_state.regs.pc += 1;

        new_state
    }
}

pub struct SEL {}
impl Executable for SEL {
    fn execute(&self, _state: &PEState) -> PEState {
        todo!()
    }
}

pub struct CMP {}
impl Executable for CMP {
    fn execute(&self, _state: &PEState) -> PEState {
        todo!()
    }
}
