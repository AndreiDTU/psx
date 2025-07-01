use std::{cell::RefCell, rc::Rc};

use crate::bus::interrupt::{Interrupt, IRQ};

pub struct Timer {
    counter: [u32; 3],
    mode: [u32; 3],
    target: [u32; 3],

    irq_enabled: [bool; 3],

    sysclock_8: usize,

    interrupt: Rc<RefCell<Interrupt>>,
}

impl Timer {
    pub fn new(interrupt: Rc<RefCell<Interrupt>>) -> Self {
        Self {
            counter: [0; 3],
            mode: [0; 3],
            target: [0; 3],

            irq_enabled: [true; 3],

            sysclock_8: 0,

            interrupt
        }
    }

    pub fn tick(&mut self) {
        self.tick_counter_0();
        self.tick_counter_1();
        self.tick_counter_2();

        // println!("Timers [{:04X}, {:04X}, {:04X}]", self.counter[0], self.counter[1], self.counter[2]);
    }

    fn tick_counter_0(&mut self) {
        let counter = &mut self.counter[0];
        let mode = self.mode[0];
        let target = self.target[0];
        let target_enabled = mode & 0x08 != 0;
        let enabled = &mut self.irq_enabled[0];

        if mode & 1 != 0 {
            panic!("Timer 0 sync modes not implemented")
        } else {
            *counter += 1;
            if *counter == 0xFFFF {
                *counter = 0;
                if mode & 0x20 != 0 && *enabled {
                    self.interrupt.borrow_mut().request(IRQ::TMR0);
                    *enabled = mode & 0x40 != 0;
                }
            } else if target_enabled && *counter == target {
                *counter = 0;
                if mode & 0x10 != 0 && *enabled {
                    self.interrupt.borrow_mut().request(IRQ::TMR0);
                    *enabled = mode & 0x40 != 0;
                }
            }
        }
    }

    fn tick_counter_1(&mut self) {
        let counter = &mut self.counter[1];
        let mode = self.mode[1];
        let target = self.target[1];
        let target_enabled = mode & 0x08 != 0;
        let enabled = &mut self.irq_enabled[1];

        if mode & 1 != 0 {
            panic!("Timer 1 sync modes not implemented")
        } else {
            *counter += 1;
            if *counter == 0xFFFF {
                *counter = 0;
                if mode & 0x20 != 0 && *enabled {
                    self.interrupt.borrow_mut().request(IRQ::TMR1);
                    *enabled = mode & 0x40 != 0;
                }
            } else if target_enabled && *counter == target {
                *counter = 0;
                if mode & 0x10 != 0 && *enabled {
                    self.interrupt.borrow_mut().request(IRQ::TMR1);
                    *enabled = mode & 0x40 != 0;
                }
            }
        }
    }

    fn tick_counter_2(&mut self) {
        let counter = &mut self.counter[2];
        let mode = &mut self.mode[2];
        let target = self.target[2];
        let target_enabled = *mode & 0x08 != 0;
        let enabled = &mut self.irq_enabled[2];
        let source = *mode & 0x200 != 0;

        if *mode & 1 != 0 {
            panic!("Timer 2 sync modes not implemented")
        } else if !source || self.sysclock_8 == 0 {
            self.sysclock_8 = 8;
            *counter += 1;
            if *counter == 0xFFFF {
                *counter = 0;
                if *mode & 0x20 != 0 && *enabled {
                    // self.interrupt.borrow_mut().request(IRQ::TMR2);
                    *enabled = *mode & 0x40 != 0;
                }
            } else if target_enabled && *counter == target {
                *counter = 0;
                if *mode & 0x10 != 0 && *enabled {
                    // self.interrupt.borrow_mut().request(IRQ::TMR2);
                    *enabled =* mode & 0x40 != 0;
                }
            }
        } else {
            self.sysclock_8 -= 1;
        }
    }

    pub fn read32(&mut self, offset: u32) -> u32 {
        let timer_idx = ((offset & 0x30) >> 4) as usize;
        match offset & 0xF {
            0x0 => {
                // println!("Timer {timer_idx}: {:04X}", self.counter[timer_idx]);
                self.counter[timer_idx]
            }
            0x4 => {
                let mode = self.mode[timer_idx];
                self.mode[timer_idx] &= !0x1800;
                mode
            }
            0x8 => self.target[timer_idx],
            0xC => 0,
            _ => unreachable!()
        }
    }

    pub fn read16(&mut self, offset: u32) -> u16 {
        let timer_idx = (offset >> 4) as usize;
        match offset & 0xF {
            0x0 => {
                println!("Timer {timer_idx}: {:04X}", self.counter[timer_idx] as u16);
                self.counter[timer_idx] as u16
            }
            0x4 => {
                let mode = self.mode[timer_idx];
                self.mode[timer_idx] &= !0x1800;
                mode as u16
            }
            0x8 => self.target[timer_idx] as u16,
            0xC => 0,
            _ => unreachable!()
        }
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        let timer_idx = ((offset & 0x30) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx] = value & 0xFFFF,
            0x4 => {
                self.irq_enabled = [true; 3];
                self.mode[timer_idx] = (value & 0x3FF) | 0x400;
                self.counter[timer_idx] = 0;
            }
            0x8 => self.target[timer_idx] = value & 0xFFFF,
            0xC => {},
            _ => unreachable!()
        }
    }

    pub fn write16(&mut self, offset: u32, value: u16) {
        let timer_idx = ((offset & 0x30) >> 4) as usize;
        match offset & 0xF {
            0x0 => self.counter[timer_idx] = value as u32,
            0x4 => {
                self.irq_enabled = [true; 3];
                self.mode[timer_idx] = (value as u32 & 0x3FF) | 0x400;
                self.counter[timer_idx] = 0;
            }
            0x8 => self.target[timer_idx] = value as u32,
            0xC => {},
            _ => unreachable!()
        }
    }
}