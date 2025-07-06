use crate::Registers;

#[derive(Debug)]
pub struct SystemControl {
    R: Registers<64>
}

impl SystemControl {
    pub fn new() -> Self {
        Self { R: Registers {R: [0; 64]} }
    }

    pub fn write_register(&mut self, register: u32, value: u32) {
        self.R[15] = 2;
        match register {
            3 | 5 | 7 | 9 | 11 | 12 =>
                self.R[register] = value,
            13 => self.R[register] |= value & 0x300,
            _ => {}
        }
    }

    pub fn read_register(&self, register: u32) -> u32 {
        self.R[register]
    }

    pub fn raise_exception(&mut self, cause: u32, current_pc: u32, next_pc: u32, delay_slot: bool) -> bool {
        let handler = self.R[12] & 0x40_0000 != 0;

        self.R[13] = cause << 2;

        if delay_slot {
            // println!("{:02X} in delay slot", cause);
            self.R[13] |= 0xC000_0000;
            self.R[6] = next_pc;
            self.R[14] = current_pc.wrapping_sub(4);
        } else {
            // println!("{:02X} not in delay slot", cause);
            self.R[13] &= !0xC000_0000;
            self.R[14] = current_pc;
        }

        let mode = self.R[12] & 0x3F;
        self.R[12] &= !0x3F;
        self.R[12] |= (mode << 2) & 0x3F;

        handler
    }

    pub fn clear_interrupt(&mut self) {
        // println!("Clearing interrupt!");
        self.R[13] &= !(1 << 10);
    }

    pub fn request_interrupt(&mut self) {
        self.R[13] |= 1 << 10;
    }

    pub fn trigger_interrupt(&self) -> bool {
        let ip = (self.R[13] >> 10) & 0xFF;
        let im = (self.R[12] >> 10) & 0xFF;

        (self.R[12] & 1) == 1 && (ip & im) != 0
    }

    pub fn rfe(&mut self) {
        // println!("RFE! cop0 {:08X}", self.R[12]);
        let mode = self.R[12] & 0x3F;
        let old = self.R[12] & 0x30;
        self.R[12] &= !0x3F;
        self.R[12] |= mode >> 2;
        self.R[12] |= old;
    }
}