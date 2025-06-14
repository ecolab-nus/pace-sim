use crate::sim::dmem::DMemInterface;

use super::instruction::{InstMode, Instruction};

#[derive(Debug, Clone)]
pub struct AGUState {
    pub pc: u32,
    pub cm: Vec<Instruction>,
    pub arf: Vec<u16>,
}

impl AGUState {
    pub fn new(cm: Vec<Instruction>, arf: Vec<u16>) -> Self {
        Self { pc: 0, cm, arf }
    }

    fn update(&mut self) {
        let inst = &self.cm[self.pc as usize];
        let pc = self.pc as usize;
        let addr = self.arf[pc];

        match inst.inst_mode {
            InstMode::STRIDED => {
                self.arf[pc] = addr + inst.stride as u16;
            }
            InstMode::CONST => {
                // do nothing
            }
        }
    }

    pub fn next(&mut self) {
        self.pc = (self.pc + 1) % self.cm.len() as u32;
    }

    pub fn update_interface(&mut self, dmem: &mut DMemInterface) {
        dmem.wire_dmem_addr = Some(self.arf[self.pc as usize] as u64);
        self.update();
        self.next();
    }
}
