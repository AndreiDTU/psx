use std::{cell::RefCell, hint::unreachable_unchecked, rc::Rc};

use crate::bus::interrupt::Interrupt;

pub struct Timer {
    counter: [u32; 3],
    mode: [u32; 3],
    target: [u32; 3],

    irq_enabled: [bool; 3],

    interrupt: Rc<RefCell<Interrupt>>,
}

impl Timer {
    pub fn new(interrupt: Rc<RefCell<Interrupt>>) -> Self {
        Self {
            counter: [0; 3],
            mode: [0; 3],
            target: [0; 3],

            irq_enabled: [true; 3],

            interrupt
        }
    }

    pub fn tick(&mut self) {

    }

    fn tick_counter_0(&mut self, cycle: usize, resolution: u8) {
        let counter = &mut self.counter[0];
        let mode = self.mode[0];
        let target = self.target[0];

        if mode & 1 != 0 {
            match (mode >> 1) & 3 {
                0 => {}
                _ => unsafe { unreachable_unchecked() }
            }
        } else {
            *counter += 1;
            if *counter == 0xFFFF {
                *counter = 0;
                if mode & 0x20 != 0 && self.irq_enabled[0] {
                    self.interrupt.borrow_mut().request(super::interrupt::IRQ::TMR0);
                    self.irq_enabled[0] = mode & 0x40 != 0;
                }
            }
        }
    }

    pub fn read32(&mut self, offset: u32) -> u32 {
        let timer_idx = ((offset & 0x10) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx],
            0x4 => {
                let mode = self.mode[timer_idx];
                self.mode[timer_idx] &= !0x00C0;
                mode
            }
            0x8 => self.target[timer_idx],
            0xC => 0,
            _ => unreachable!()
        }
    }

    pub fn read16(&mut self, offset: u32) -> u16 {
        let timer_idx = ((offset & 0x10) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx] as u16,
            0x4 => {
                let mode = self.mode[timer_idx];
                self.mode[timer_idx] &= !0x00C0;
                mode as u16
            }
            0x8 => self.target[timer_idx] as u16,
            0xC => 0,
            _ => unreachable!()
        }
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        let timer_idx = ((offset & 0x10) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx] = value & 0xFFFF,
            0x4 => {
                self.irq_enabled = [true; 3];
                self.mode[timer_idx] = (value & 0x2FF) | 0x400;
                self.counter[timer_idx] = 0;
            }
            0x8 => self.target[timer_idx] = value & 0xFFFF,
            0xC => {},
            _ => unreachable!()
        }
    }

    pub fn write16(&mut self, offset: u32, value: u16) {
        let timer_idx = ((offset & 0x10) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx] = value as u32,
            0x4 => {
                self.irq_enabled = [true; 3];
                self.mode[timer_idx] = value as u32 & 0x2FF;
                self.counter[timer_idx] = 0;
            }
            0x8 => self.target[timer_idx] = value as u32,
            0xC => {},
            _ => unreachable!()
        }
    }
}