//! Model the data memory

use crate::isa::state::PESignals;

#[derive(Default, Debug, Clone)]
pub struct DataMemory {
    pub data: Vec<u8>,
}

impl DataMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn write(&mut self, addr: u64, data: u8) {
        self.data[addr as usize] = data;
    }

    pub fn read(&self, addr: u64) -> u8 {
        self.data[addr as usize]
    }

    pub fn update_with_pe(&mut self, pe: &PESignals) {
        if let Some(addr) = pe.wire_dmem_addr {
            if let Some(data) = pe.wire_dmem_data {
                self.write(addr, data as u8);
            } else {
                self.read(addr) as u64;
            }
        }
    }
}
