use crate::cd_rom::{DiskAddress, CD_ROM, CD_ROM_STATUS};

impl CD_ROM {
    pub fn setloc(&mut self) {
        self.seek_target = DiskAddress::from_bytes(&[
            self.parameters.pop_front().unwrap(),
            self.parameters.pop_front().unwrap(),
            self.parameters.pop_front().unwrap()
        ]);

        self.send_status(3, None, None);
    }

    pub fn seekL(&mut self) {
        self.read_addr = self.seek_target;

        self.status.insert(CD_ROM_STATUS::SEEK);
        self.status.remove(CD_ROM_STATUS::READ);
        self.status.remove(CD_ROM_STATUS::PLAY);

        self.send_status(3, None, Some(Self::seekL_second_response));
    }

    pub fn seekL_second_response(&mut self) {
        self.status.remove(CD_ROM_STATUS::READ);
        self.status.remove(CD_ROM_STATUS::SEEK);
        self.status.remove(CD_ROM_STATUS::PLAY);

        self.send_status(2, Some(SEEKL_SECOND_DELAY), None);
    }
}

const SEEKL_SECOND_DELAY: usize = 440000;