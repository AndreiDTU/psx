use crate::cd_rom::{SecondResponse, CD_ROM, CD_ROM_MODE, CD_ROM_STATUS};

impl CD_ROM {
    pub fn setmode(&mut self) {
        self.mode = CD_ROM_MODE::from_bits_truncate(self.parameters.pop_front().unwrap());

        self.send_status(3);
    }

    pub fn init(&mut self) {
        const INIT_FIRST_DELAY: usize = 0x0001_3CCE;

        self.mode = CD_ROM_MODE::from_bits_truncate(0x20);

        self.send_status(3);

        self.second_response = SecondResponse::Init;
        self.irq_delay = INIT_FIRST_DELAY;
    }

    pub fn init_second_response(&mut self) {
        const INIT_SECOND_DELAY: usize = 33_000_000 / 75;
        
        self.irq_delay = INIT_SECOND_DELAY;
        self.send_status(2);
        self.second_response = SecondResponse::None;
    }

    pub fn pause(&mut self) {
        self.send_status(3);
        self.second_response = SecondResponse::Pause;
    }

    pub fn pause_second_response(&mut self) {
        let paused = (!self.status.intersects(const {CD_ROM_STATUS::from_bits_truncate(0xE0)}) as usize) << 1;
        let speed = self.mode.contains(CD_ROM_MODE::SPEED) as usize;

        self.status.remove(CD_ROM_STATUS::PLAY);
        self.status.remove(CD_ROM_STATUS::SEEK);
        self.status.remove(CD_ROM_STATUS::READ);

        self.irq_delay = PAUSE_SECOND_DELAY[paused + speed];
        self.send_status(2);
        self.second_response = SecondResponse::None;
    }
}

const PAUSE_SECOND_DELAY: [usize; 4] = [
    0x0021_181C,
    0x0010_BD93,
    0x0000_1DF2,
    0x0000_1DF2,
];