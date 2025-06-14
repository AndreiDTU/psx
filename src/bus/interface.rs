use std::path::Path;

use crate::{bios::BIOS, ram::RAM};

const IO_START: u32 = 0x1F801000;
const IO_END: u32 = IO_START + (4 * 1024);

const DRAM_SIZE: usize = 2 * 1024 * 1024;
const DRAM_START: u32 = 0xA000_0000;
const DRAM_END: u32 = DRAM_START + DRAM_SIZE as u32;

const BIOS_START: u32 = 0xBFC0_0000;
const BIOS_END: u32 = BIOS_START + (512 * 1024);

const CACHE_CONTROL: u32 = 0xFFFE0130;
const CACHE_CONTROL_END: u32 = CACHE_CONTROL + 4;

pub struct Interface {
    bios: BIOS,
    dram: RAM,
}

impl Interface {
    pub fn new(path: &Path) -> Result<Self, anyhow::Error> {
        let bios = BIOS::new(path)?;
        let dram = RAM::new(DRAM_SIZE);

        Ok(Self { bios, dram })
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        if addr & 0b11 != 0 {panic!("Unaligned write at {:08X}", addr)}

        match addr {
            IO_START..IO_END => {}
            DRAM_START..DRAM_END => self.dram.write32(addr - DRAM_START, value),
            CACHE_CONTROL..CACHE_CONTROL_END => {println!("Write to CACHE_CONTROL")}
            _ => panic!("Write access at unmapped address: {:08X}", addr),
        }
    }

    pub fn read32(&self, addr: u32) -> u32 {
        if addr & 0b11 != 0 {panic!("Unaligned read at {:08X}", addr)}

        match addr {
            IO_START..IO_END => 0,
            DRAM_START..DRAM_END => self.dram.read32(addr - DRAM_START),
            BIOS_START..BIOS_END => self.bios.read32(addr - BIOS_START),
            _ => panic!("Read access at unmapped address: {:08X}", addr),
        }
    }
}