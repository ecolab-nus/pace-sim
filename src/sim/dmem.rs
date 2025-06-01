//! Model the data memory

use crate::isa::pe::{DMemInterface, DMemMode};

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

    pub fn write8(&mut self, addr: u64, data: u8) {
        self.data[addr as usize] = data;
    }

    pub fn read8(&self, addr: u64) -> u8 {
        self.data[addr as usize]
    }

    pub fn write16(&mut self, addr: u64, data: u16) {
        self.data[addr as usize] = data as u8;
        self.data[addr as usize + 1] = (data >> 8) as u8;
    }

    pub fn read16(&self, addr: u64) -> u16 {
        self.data[addr as usize] as u16 | (self.data[addr as usize + 1] as u16) << 8
    }

    // pub fn write32(&mut self, addr: u64, data: u32) {
    //     self.data[addr as usize] = data as u8;
    //     self.data[addr as usize + 1] = (data >> 8) as u8;
    //     self.data[addr as usize + 2] = (data >> 16) as u8;
    //     self.data[addr as usize + 3] = (data >> 24) as u8;
    // }

    // pub fn read32(&self, addr: u64) -> u32 {
    //     self.data[addr as usize] as u32
    //         | (self.data[addr as usize + 1] as u32) << 8
    //         | (self.data[addr as usize + 2] as u32) << 16
    //         | (self.data[addr as usize + 3] as u32) << 24
    // }

    pub fn write64(&mut self, addr: u64, data: u64) {
        self.data[addr as usize] = data as u8;
        self.data[addr as usize + 1] = (data >> 8) as u8;
        self.data[addr as usize + 2] = (data >> 16) as u8;
        self.data[addr as usize + 3] = (data >> 24) as u8;
        self.data[addr as usize + 4] = (data >> 32) as u8;
    }

    pub fn read64(&self, addr: u64) -> u64 {
        self.data[addr as usize] as u64
            | (self.data[addr as usize + 1] as u64) << 8
            | (self.data[addr as usize + 2] as u64) << 16
            | (self.data[addr as usize + 3] as u64) << 24
            | (self.data[addr as usize + 4] as u64) << 32
    }

    pub fn update_interface(&mut self, interface: &mut DMemInterface) {
        match interface.mode {
            DMemMode::Read8 => {
                interface.reg_dmem_data =
                    Some(self.read8(interface.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read16 => {
                interface.reg_dmem_data =
                    Some(self.read16(interface.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read64 => {
                interface.reg_dmem_data = Some(self.read64(interface.wire_dmem_addr.unwrap()));
            }
            DMemMode::Write8 => {
                self.write8(
                    interface.wire_dmem_addr.unwrap(),
                    interface.reg_dmem_data.unwrap() as u8,
                );
            }
            DMemMode::Write16 => {
                self.write16(
                    interface.wire_dmem_addr.unwrap(),
                    interface.reg_dmem_data.unwrap() as u16,
                );
            }
            DMemMode::Write64 => {
                self.write64(
                    interface.wire_dmem_addr.unwrap(),
                    interface.reg_dmem_data.unwrap() as u64,
                );
            }
            DMemMode::NOP => {}
        }
    }
}
