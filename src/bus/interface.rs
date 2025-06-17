use std::{cell::RefCell, path::Path, rc::Weak};

use crate::{bios::BIOS, bus::dma::DMA, gpu::GPU, ram::RAM};

const DRAM_SIZE: usize = 2 * 1024 * 1024;
const DRAM_START: u32 = 0x0000_0000;
const DRAM_END: u32 = DRAM_START + DRAM_SIZE as u32;

const EXPANSION_1_START: u32 = 0x1F00_0000;
const EXPANSION_1_END: u32 = EXPANSION_1_START + 0x80000;

const BIOS_START: u32 = 0x1FC0_0000;
const BIOS_END: u32 = BIOS_START + (512 * 1024);

const IO_START: u32 = 0x1F801000;
const IO_END: u32 = IO_START + (4 * 1024);

const DMA_START: u32 = 0x1F801080;
const DMA_END: u32 = DMA_START + 0x80;

const GPU_START: u32 = 0x1F801810;
const GPU_END: u32 = GPU_START + 8;

const EXPANSION_2_START: u32 = 0x1F802000;
const EXPANSION_2_END: u32 = EXPANSION_2_START + 66;

const CACHE_CONTROL: u32 = 0xFFFE0130;
const CACHE_CONTROL_END: u32 = CACHE_CONTROL + 4;

pub struct Interface {
    bios: BIOS,
    pub(crate) dma: Weak<RefCell<DMA>>,
    pub dram: RAM,
    pub gpu: GPU,
}

impl Interface {
    pub fn new(path: &Path) -> Result<Self, anyhow::Error> {
        let bios = BIOS::new(path)?;
        let dram = RAM::new(DRAM_SIZE);
        let gpu = GPU::new();

        Ok(Self { bios, dma: Weak::new(), dram, gpu })
    }

    pub fn read32(&self, addr: u32) -> u32 {
        if addr & 0b11 != 0 {panic!("Unaligned read at {:08X}", addr)}
        
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read32(addr - DRAM_START),
            BIOS_START..BIOS_END => self.bios.read32(addr - BIOS_START),
            DMA_START..DMA_END => self.dma.upgrade().unwrap().borrow().read_register(addr - DMA_START),
            GPU_START..GPU_END => {
                let offset = addr - GPU_START;
                match offset {
                    0 => self.gpu.read_gp0(),
                    4 => self.gpu.read_gp1(),
                    _ => unreachable!(),
                }
            }
            IO_START..IO_END => 0,
            _ => panic!("Read access at unmapped address: {:08X}", addr),
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        if addr & 0b1 != 0 {panic!("Unaligned read at {:08X}", addr)}
        
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read16(addr - DRAM_START),
            IO_START..IO_END => 0,
            _ => panic!("Read access at unmapped address: {:08X}", addr),
        }
    }

    pub fn read8(&self, addr: u32) -> u8 {
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read8(addr - DRAM_START),
            EXPANSION_1_START..EXPANSION_1_END => 0xFF,
            BIOS_START..BIOS_END => self.bios.read8(addr - BIOS_START),
            _ => panic!("Read 8-bit access at unmapped address: {:08X}", addr),
        }
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        if addr & 0b11 != 0 {panic!("Unaligned write at {:08X}", addr)}

        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.write32(addr - DRAM_START, value),
            DMA_START..DMA_END => self.dma.upgrade().unwrap().borrow_mut().write_register(addr - DMA_START, value),
            GPU_START..GPU_END => {
                let offset = addr - GPU_START;
                match offset {
                    0 => self.gpu.write_gp0(value),
                    4 => self.gpu.write_gp1(value),
                    _ => unreachable!(),
                }
            }
            IO_START..IO_END => {}
            CACHE_CONTROL..CACHE_CONTROL_END => {
                // println!("Write to CACHE_CONTROL")
            }
            _ => panic!("Write access at unmapped address: {:08X}", addr),
        }
    }

    pub fn write16(&mut self, addr: u32, value: u16) {
        if addr & 1 != 0 {panic!("Unaligned 16-bit write at: {:08X}", addr)}

        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.write16(addr - DRAM_START, value),
            IO_START..IO_END => {}
            _ => panic!("Write 16-bit access at unmapped address: {:08X}", addr),
        }
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.write8(addr - DRAM_START, value),
            EXPANSION_2_START..EXPANSION_2_END => {}
            _ => panic!("Write 8-bit access at unmapped address: {:08X}", addr),
        }
    }
}

const REGION_MASK: [u32; 8] = [
    // KUSEG: 2048MB
    0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF,
    // KSEG0: 512MB
    0x7FFF_FFFF,
    // KSEG1: 512MB
    0x1FFF_FFFF,
    // KSEG2: 1024MB
    0xFFFF_FFFF, 0xFFFF_FFFF,
];

pub fn mask_region(addr: u32) -> u32 {
    let index = (addr >> 29) as usize;

    addr & REGION_MASK[index]
}