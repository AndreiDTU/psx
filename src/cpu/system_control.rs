use crate::cpu::{decoder::Cause, Registers};

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

    pub fn raise_exception(&mut self, cause: Cause, current_pc: u32) -> bool {
        let handler = self.R[12] & (1 << 22) != 0;

        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= (mode << 2) & 0x3F;

        self.R[13] = (cause as u32) << 2;
        self.R[14] = current_pc;

        handler
    }

    pub fn rfe(&mut self) {
        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= mode >> 2;
    }
}