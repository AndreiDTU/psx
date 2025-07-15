use crate::cd_rom::CD_ROM;

impl CD_ROM {
    pub fn demute(&mut self) {
        self.mute = false;

        self.send_status(3, None, None);
    }
}