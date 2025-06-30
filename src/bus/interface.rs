use std::{cell::RefCell, path::Path, rc::{Rc, Weak}};

use crate::{bios::BIOS, bus::{dma::DMA, interrupt::Interrupt, timer::Timer}, cd_rom::CD_ROM, gpu::GPU, ram::RAM};

const DRAM_SIZE: usize = 2 * 1024 * 1024;
const DRAM_START: u32 = 0x0000_0000;
const DRAM_END: u32 = DRAM_START + DRAM_SIZE as u32 * 4;

const EXPANSION_1_START: u32 = 0x1F00_0000;
const EXPANSION_1_END: u32 = EXPANSION_1_START + 0x80000;

const SCRATCHPAD_SIZE: usize = 0x400;
const SCRATCHPAD_START: u32 = 0x1F80_0000;
const SCRATCHPAD_END: u32 = SCRATCHPAD_START + SCRATCHPAD_SIZE as u32;

const MEM_CTRL_START: u32 = 0x1F80_1000;
const MEM_CTRL_END: u32 = MEM_CTRL_START + 0x24;

const PERIPHERAL_START: u32 = 0x1F801040;
const PERIPHERAL_END: u32 = PERIPHERAL_START + 0x20;

const MEM_CTRL_2_START: u32 = 0x1F80_1060;
const MEM_CTRL_2_END: u32 = MEM_CTRL_2_START + 4;

const IQR_START: u32 = 0x1F80_1070;
const IRQ_END: u32 = IQR_START + 8;

const DMA_START: u32 = 0x1F801080;
const DMA_END: u32 = DMA_START + 0x80;

const TIMER_START: u32 = 0x1F801100;
const TIMER_END: u32 = TIMER_START + 0x30;

const CD_ROM_START: u32 = 0x1F801800;
const CD_ROM_END: u32 = CD_ROM_START + 4;

const GPU_START: u32 = 0x1F801810;
const GPU_END: u32 = GPU_START + 8;

const VOICE_START: u32 = 0x1F801C00;
const VOICE_END: u32 = VOICE_START + 24 * 0x10;

const SPU_START: u32 = 0x1F801D80;
const SPU_END: u32 = SPU_START + 0x40;

const REVERB_START: u32 = 0x1F801DC0;
const REVERB_END: u32 = REVERB_START + 0x40;

const EXPANSION_2_START: u32 = 0x1F802000;
const EXPANSION_2_END: u32 = EXPANSION_2_START + 66;

const BIOS_START: u32 = 0x1FC0_0000;
const BIOS_END: u32 = BIOS_START + (512 * 1024);

const CACHE_CONTROL_START: u32 = 0xFFFE_0130;
const CACHE_CONTROL_END: u32 = 0xFFFF_FFFF;

pub struct Interface {
    bios: BIOS,
    pub(crate) dma: Weak<RefCell<DMA>>,
    pub dram: RAM,
    pub scratchpad: RAM,
    pub gpu: GPU,
    pub interrupt: Rc<RefCell<Interrupt>>,
    cd_rom: Rc<RefCell<CD_ROM>>,
    timer: Rc<RefCell<Timer>>,
}

impl Interface {
    pub fn new(path: &Path, interrupt: Rc<RefCell<Interrupt>>, cd_rom: Rc<RefCell<CD_ROM>>) -> Result<Self, anyhow::Error> {
        let bios = BIOS::new(path)?;
        let dram = RAM::new(DRAM_SIZE);
        let scratchpad = RAM::new(SCRATCHPAD_SIZE);
        let timer = Rc::new(RefCell::new(Timer::new(interrupt.clone())));
        let gpu = GPU::new(interrupt.clone(), timer.clone());

        Ok(Self { bios, dma: Weak::new(), dram, scratchpad, gpu, interrupt, timer, cd_rom })
    }

    pub fn read32(&mut self, addr: u32) -> u32 {
        if addr & 0b11 != 0 {panic!("Unaligned read at {:08X}", addr)}
        
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read32((addr - DRAM_START) & 0x3FFFFFF),
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.read32(addr - SCRATCHPAD_START),
            BIOS_START..BIOS_END => self.bios.read32(addr - BIOS_START),
            MEM_CTRL_START..MEM_CTRL_END => 0,
            PERIPHERAL_START..PERIPHERAL_END => 0,
            MEM_CTRL_2_START..MEM_CTRL_2_END => 0,
            TIMER_START..TIMER_END => self.timer.borrow_mut().read32(addr - TIMER_START),
            CD_ROM_START..CD_ROM_END => 0,
            IQR_START..IRQ_END => {
                let offset = addr - IQR_START;
                match offset {
                    0 => self.interrupt.borrow_mut().read_status32(),
                    4 => self.interrupt.borrow_mut().read_mask32(),
                    _ => unreachable!(),
                }
            }
            DMA_START..DMA_END => self.dma.upgrade().unwrap().borrow().read_register(addr - DMA_START),
            GPU_START..GPU_END => {
                let offset = addr - GPU_START;
                match offset {
                    0 => self.gpu.read_gp0(),
                    4 => self.gpu.read_gp1(),
                    _ => unreachable!(),
                }
            }
            VOICE_START..VOICE_END => 0,
            SPU_START..SPU_END => 0,
            REVERB_START..REVERB_END => 0,
            CACHE_CONTROL_START..=CACHE_CONTROL_END => 0,
            _ => panic!("Read access at unmapped address: {:08X}", addr),
        }
    }

    pub fn read16(&mut self, addr: u32) -> u16 {
        if addr & 0b1 != 0 {panic!("Unaligned read at {:08X}", addr)}
        
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read16(addr - DRAM_START),
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.read16(addr - SCRATCHPAD_START),
            MEM_CTRL_START..MEM_CTRL_END => 0,
            PERIPHERAL_START..PERIPHERAL_END => 0,
            MEM_CTRL_2_START..MEM_CTRL_2_END => 0,
            TIMER_START..TIMER_END => self.timer.borrow_mut().read16(addr - TIMER_START),
            CD_ROM_START..CD_ROM_END => 0,
            IQR_START..IRQ_END => {
                let offset = addr - IQR_START;
                match offset {
                    0 => self.interrupt.borrow_mut().read_status16(),
                    4 => self.interrupt.borrow_mut().read_mask16(),
                    _ => unreachable!(),
                }
            }
            VOICE_START..VOICE_END => 0,
            SPU_START..SPU_END => 0,
            REVERB_START..REVERB_END => 0,
            _ => panic!("Read 16-bit access at unmapped address: {:08X}", addr),
        }
    }

    pub fn read8(&mut self, addr: u32) -> u8 {
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.read8(addr - DRAM_START),
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.read8(addr - SCRATCHPAD_START),
            EXPANSION_1_START..EXPANSION_1_END => 0xFF,
            BIOS_START..BIOS_END => self.bios.read8(addr - BIOS_START),
            MEM_CTRL_START..MEM_CTRL_END => 0,
            PERIPHERAL_START..PERIPHERAL_END => 0,
            MEM_CTRL_2_START..MEM_CTRL_2_END => 0,
            CD_ROM_START..CD_ROM_END => self.cd_rom.borrow_mut().read8(addr - CD_ROM_START),
            VOICE_START..VOICE_END => 0,
            SPU_START..SPU_END => 0,
            REVERB_START..REVERB_END => 0,
            _ => panic!("Read 8-bit access at unmapped address: {:08X}", addr),
        }
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        if addr & 0b11 != 0 {panic!("Unaligned write at {:08X}", addr)}

        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.write32(addr - DRAM_START, value),
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.write32(addr - SCRATCHPAD_START, value),
            MEM_CTRL_START..MEM_CTRL_END => {},
            PERIPHERAL_START..PERIPHERAL_END => {},
            MEM_CTRL_2_START..MEM_CTRL_2_END => {},
            TIMER_START..TIMER_END => self.timer.borrow_mut().write32(addr - TIMER_START, value),
            CD_ROM_START..CD_ROM_END => {},
            IQR_START..IRQ_END => {
                let offset = addr - IQR_START;
                match offset {
                    0 => self.interrupt.borrow_mut().acknowledge32(value),
                    4 => self.interrupt.borrow_mut().write_mask32(value),
                    _ => unreachable!(),
                }
            }
            DMA_START..DMA_END => self.dma.upgrade().unwrap().borrow_mut().write_register(addr - DMA_START, value),
            GPU_START..GPU_END => {
                let offset = addr - GPU_START;
                match offset {
                    0 => self.gpu.write_gp0(value),
                    4 => self.gpu.write_gp1(value),
                    _ => unreachable!(),
                }
            }
            VOICE_START..VOICE_END => {},
            SPU_START..SPU_END => {},
            REVERB_START..REVERB_END => {},
            CACHE_CONTROL_START..=CACHE_CONTROL_END => {
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
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.write16(addr - SCRATCHPAD_START, value),
            MEM_CTRL_START..MEM_CTRL_END => {},
            PERIPHERAL_START..PERIPHERAL_END => {},
            MEM_CTRL_2_START..MEM_CTRL_2_END => {},
            TIMER_START..TIMER_END => self.timer.borrow_mut().write16(addr - TIMER_START, value),
            CD_ROM_START..CD_ROM_END => {},
            IQR_START..IRQ_END => {
                let offset = addr - IQR_START;
                match offset {
                    0 => self.interrupt.borrow_mut().acknowledge16(value),
                    4 => self.interrupt.borrow_mut().write_mask16(value),
                    _ => unreachable!(),
                }
            }
            VOICE_START..VOICE_END => {},
            SPU_START..SPU_END => {},
            REVERB_START..REVERB_END => {},
            _ => panic!("Write 16-bit access at unmapped address: {:08X}", addr),
        }
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        let addr = mask_region(addr);
        match addr {
            DRAM_START..DRAM_END => self.dram.write8(addr - DRAM_START, value),
            SCRATCHPAD_START..SCRATCHPAD_END => self.scratchpad.write8(addr - SCRATCHPAD_START, value),
            MEM_CTRL_START..MEM_CTRL_END => {},
            PERIPHERAL_START..PERIPHERAL_END => {},
            MEM_CTRL_2_START..MEM_CTRL_2_END => {},
            CD_ROM_START..CD_ROM_END => self.cd_rom.borrow_mut().write8(addr - CD_ROM_START, value),
            VOICE_START..VOICE_END => {},
            SPU_START..SPU_END => {},
            REVERB_START..REVERB_END => {},
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