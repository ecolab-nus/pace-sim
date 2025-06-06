use crate::sim::dmem::DMemInterface;

use super::instruction::{InstMode, Instruction};

pub struct AGUState {
    pub pc: u32,
    pub cm: Vec<Instruction>,
    pub arf: Vec<u16>,
}

impl AGUState {
    pub fn new(cm: Vec<Instruction>, arf: Vec<u16>) -> Self {
        Self { pc: 0, cm, arf }
    }

    pub fn step(&mut self, dmem: &mut DMemInterface) {
        let inst = &self.cm[self.pc as usize];
        let pc = self.pc as usize;
        let addr = self.arf[pc];
        dmem.wire_dmem_addr = Some(addr as u64);
        match inst.inst_mode {
            InstMode::STRIDED => {
                self.arf[pc] = addr + inst.stride as u16;
            }
            InstMode::CONST => {
                // do nothing
            }
        }
    }
}
