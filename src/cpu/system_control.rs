use crate::{cpu::decoder::Cause, Registers};

pub struct SystemControl {
    R: Registers<64>
}

impl SystemControl {
    pub fn new() -> Self {
        Self { R: Registers {R: [0; 64]} }
    }

    pub fn write_register(&mut self, register: u32, value: u32) {
        match register {
            3 | 5 | 7 | 9 | 11 | 12 =>
                self.R[register] = value,
            _ => {}
        }
    }

    pub fn read_register(&self, register: u32) -> u32 {
        self.R[register]
    }

    pub fn raise_exception(&mut self, cause: u32, current_pc: u32, pc: u32, delay_slot: bool) -> bool {
        let handler = self.R[12] & 0x40_0000 != 0;

        let pc = if cause == Cause::INT as u32 {pc} else {current_pc};

        let old = self.R[13] & 0x300;
        self.R[13] = cause << 2;
        self.R[13] |= old;

        if delay_slot {
            self.R[13] |= 1 << 31;
            self.R[14] = pc.wrapping_sub(4);
        } else {
            self.R[13] &= !(1 << 31);
            self.R[14] = pc;
        }

        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= (mode << 2) & 0x3F;

        handler
    }

    pub fn clear_interrupt(&mut self) {
        self.R[13] &= !(1 << 10);
    }

    pub fn request_interrupt(&mut self) {
        self.R[13] |= 1 << 10;
    }

    pub fn trigger_interrupt(&self) -> bool {
        let ip = (self.R[13] >> 8) & 0xFF;
        let im = (self.R[12] >> 8) & 0xFF;

        self.R[13] & 1 != 0 && ip & im != 0
    }

    pub fn rfe(&mut self) {
        let mode = self.R[12] & 0x3F;
        let old = self.R[12] & 0x30;
        self.R[12] &= !0x3F;
        self.R[12] |= mode >> 2;
        self.R[12] |= old;
    }
}