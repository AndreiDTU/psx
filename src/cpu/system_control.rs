use crate::Registers;

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

    pub fn raise_exception(&mut self, cause: u32, current_pc: u32, delay_slot: bool) -> bool {
        let handler = self.R[12] & (1 << 22) != 0;

        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= (mode << 2) & 0x3F;

        self.R[13] = cause << 2;
        self.R[14] = current_pc;

        if delay_slot {
            self.R[14] = self.R[14].wrapping_sub(4);
            self.R[13] |= 1 << 31;
        }

        handler
    }

    pub fn request_interrupt(&mut self) {
        self.R[13] |= 1 << 10;
    }

    pub fn trigger_interrupt(&self) -> bool {
        self.R[13] & (1 << 10) != 0 && self.R[12] & 0x401 == 0x401
    }

    pub fn rfe(&mut self) {
        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= mode >> 2;
    }
}