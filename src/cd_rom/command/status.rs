use crate::cd_rom::{SecondResponse, CD_ROM};

impl CD_ROM {
    pub fn send_status(&mut self, int: u8) {
        self.result_fifo[self.result_idx] = self.status.bits();
        self.result_size = 0;
        self.result_fifo_empty = false;

        self.schedule_int(int);
    }

    pub fn get_id(&mut self) {
        self.send_status(3);
        self.second_response = SecondResponse::GetID;
    }

    pub fn get_id_second_response(&mut self) {
        self.result_idx = 0;
        self.result_size = 8;
        *self.result_fifo[self.result_idx..].first_chunk_mut().unwrap() = DISK_MODE1;
        // self.status.insert(CD_ROM_STATUS::SHELL);
        self.schedule_int(2);
        self.irq_delay = ID_SECOND_DELAY;
        self.second_response = SecondResponse::None;
    }
}

const ID_SECOND_DELAY: usize = 0x4A00;

#[allow(unused)]
const NO_DISK: [u8; 8] = [0x08, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const DISK_MODE1: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x53, 0x43, 0x45, 0x41];