use crate::cd_rom::{CD_ROM, CD_ROM_MODE, CD_ROM_STATUS};

impl CD_ROM {
    pub fn setmode(&mut self) {
        self.mode = CD_ROM_MODE::from_bits_truncate(self.parameters.pop_front().unwrap());

        self.send_status(3, None, None);
    }

    pub fn init(&mut self) {
        const INIT_FIRST_DELAY: usize = 0x0001_3CCE;

        self.mode = CD_ROM_MODE::from_bits_truncate(0x20);
        
        self.int_queue.clear();
        self.pending_int = None;
        self.send_status(3, Some(INIT_FIRST_DELAY), Some(Self::init_second_response));
    }

    pub fn init_second_response(&mut self) {
        const INIT_SECOND_DELAY: usize = 33_000_000 / 75;

        self.send_status(2, Some(INIT_SECOND_DELAY), None);
    }

    pub fn pause(&mut self) {
        self.int_queue = self.int_queue.iter()
            .filter(|int| {int.num != 1})
            .map(|int| *int)
            .collect();

        if let Some(int) = self.pending_int {
            if int.num == 1 {self.pending_int = None}
        }

        self.send_status(3, None, Some(Self::pause_second_response));
    }

    pub fn pause_second_response(&mut self) {
        let paused = (!self.status.intersects(const {CD_ROM_STATUS::from_bits_truncate(0xE0)}) as usize) << 1;
        let speed = self.mode.contains(CD_ROM_MODE::SPEED) as usize;

        self.sector_buffer = [None; 2];

        self.status.remove(CD_ROM_STATUS::PLAY);
        self.status.remove(CD_ROM_STATUS::SEEK);
        self.status.remove(CD_ROM_STATUS::READ);

        self.send_status(2, Some(PAUSE_SECOND_DELAY[paused + speed]), None);
    }
}

const PAUSE_SECOND_DELAY: [usize; 4] = [
    0x0021_181C,
    0x0010_BD93,
    0x0000_1DF2,
    0x0000_1DF2,
];