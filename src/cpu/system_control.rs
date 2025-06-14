use crate::cpu::Registers;

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
}