use std::{cell::RefCell, rc::Rc};

use crate::cpu::system_control::SystemControl;

pub struct Interrupt {
    I_STAT: u32,
    I_MASK: u32,

    system_control: Rc<RefCell<SystemControl>>,
}

impl Interrupt {
    pub fn new(system_control: Rc<RefCell<SystemControl>>) -> Self {
        Self { I_STAT: 0, I_MASK: 0, system_control }
    }

    pub fn read_status32(&self) -> u32 {
        self.I_STAT
    }

    pub fn read_mask32(&self) -> u32 {
        self.I_MASK & 0x7FF
    }

    pub fn acknowledge32(&mut self, value: u32) {
        self.I_STAT &= (value & 0x7FF) | 0xF800;
        if self.I_STAT & self.I_MASK == 0 {
            // println!("Clearing interrupt!");
            self.system_control.borrow_mut().clear_interrupt();
        }
    }

    pub fn write_mask32(&mut self, value: u32) {
        self.I_MASK = value & 0x7FF;
        println!("interrupt mask: {:08X}", self.I_MASK);
    }

    pub fn read_status16(&self) -> u16 {
        self.I_STAT as u16
    }

    pub fn read_mask16(&self) -> u16 {
        self.I_MASK as u16 & 0x7FF
    }

    pub fn acknowledge16(&mut self, value: u16) {
        self.I_STAT &= ((value & 0x7FF) | 0xF800) as u32;
        if self.I_STAT & self.I_MASK == 0 {
            // println!("Clearing interrupt!");
            self.system_control.borrow_mut().clear_interrupt();
        }
    }

    pub fn write_mask16(&mut self, value: u16) {
        self.I_MASK = value as u32;
        println!("interrupt mask: {:08X}", self.I_MASK);
    }

    pub fn request(&mut self, irq: IRQ) {
        self.I_STAT |= irq as u32;
        if (self.I_STAT & self.I_MASK) & 0x7FF != 0 {
            // println!("IRQ: {irq:#?}");
            self.system_control.borrow_mut().request_interrupt();
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IRQ {
    VBLANK        = 0x001,
    GPU           = 0x002,
    CDROM         = 0x004,
    DMA           = 0x008,
    TMR0          = 0x010,
    TMR1          = 0x020,
    TMR2          = 0x040,
    BYTE_RECEIVED = 0x080,
    SIO           = 0x100,
    SPU           = 0x200,
    LIGHTPEN      = 0x400,
}