use crate::cd_rom::{CD_ROM, CD_ROM_MODE};

impl CD_ROM {
    pub fn setmode(&mut self) {
        self.mode = CD_ROM_MODE::from_bits_truncate(self.parameters.pop_front().unwrap());

        self.send_status(3);
    }
}