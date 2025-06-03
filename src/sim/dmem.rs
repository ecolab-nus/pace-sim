//! Model the data memory

use crate::isa::pe::{DMemInterface, DMemMode};

#[derive(Default, Debug, Clone)]
pub struct DataMemory {
    pub data: Vec<u8>,
    pub interface: DMemInterface,
}

impl DataMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            interface: DMemInterface::default(),
        }
    }

    /// Load the data memory content from binary string
    /// The file format is :
    /// 64 bits per line, one bit per character
    /// From left to right is from most significant bit to least significant bit
    pub fn from_binary_str(s: &str) -> Self {
        let mut data = Vec::new();
        for line in s.lines() {
            let mut byte = 0;
            for (i, c) in line.chars().enumerate() {
                if c == '1' {
                    byte |= 1 << (63 - i);
                }
            }
            data.push(byte);
        }
        Self {
            data,
            interface: DMemInterface::default(),
        }
    }

    pub fn to_binary_str(&self) -> String {
        let mut result = String::new();
        for byte in self.data.iter() {
            result.push_str(&format!("{:064b}\n", byte));
        }
        result
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

    pub fn update_interface(&mut self) {
        match self.interface.mode {
            DMemMode::Read8 => {
                self.interface.reg_dmem_data =
                    Some(self.read8(self.interface.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read16 => {
                self.interface.reg_dmem_data =
                    Some(self.read16(self.interface.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read64 => {
                self.interface.reg_dmem_data =
                    Some(self.read64(self.interface.wire_dmem_addr.unwrap()));
            }
            DMemMode::Write8 => {
                self.write8(
                    self.interface.wire_dmem_addr.unwrap(),
                    self.interface.reg_dmem_data.unwrap() as u8,
                );
            }
            DMemMode::Write16 => {
                self.write16(
                    self.interface.wire_dmem_addr.unwrap(),
                    self.interface.reg_dmem_data.unwrap() as u16,
                );
            }
            DMemMode::Write64 => {
                self.write64(
                    self.interface.wire_dmem_addr.unwrap(),
                    self.interface.reg_dmem_data.unwrap() as u64,
                );
            }
            DMemMode::NOP => {}
        }
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        for (i, chunk) in self.data.chunks(32).enumerate() {
            if i > 0 {
                result.push('\n');
            }
            for (j, block) in chunk.chunks(8).enumerate() {
                if j > 0 {
                    result.push_str(" | ");
                }
                for (k, &byte) in block.iter().enumerate() {
                    if k > 0 {
                        result.push(' ');
                    }
                    result.push_str(&format!("{:02x}", byte));
                }
            }
        }
        result
    }
}

mod tests {

    #[test]
    fn test_dmem_dump() {
        use super::*;
        // create a 8KB data memory
        let mut dmem = DataMemory::new(8192);
        for i in 0..8192 {
            dmem.write8(i as u64, i as u8);
        }

        let dump = dmem.to_binary_str();
        // load from the file and compare
        let dmem_loaded = DataMemory::from_binary_str(&dump);
        assert_eq!(dmem.data, dmem_loaded.data);
    }
}
