use crate::cd_rom::{AVERAGE_IRQ_DELAY, CD_ROM_INT};
#[allow(unused)]
use crate::cd_rom::{CD_ROM, CD_ROM_STATUS};

impl CD_ROM {
    pub fn send_status(&mut self, int: u8, delay: Option<usize>, func: Option<fn(&mut CD_ROM)>) {
        self.result_fifo[self.result_idx] = self.status.bits();
        self.result_size = 0;
        self.result_fifo_empty = false;

        println!("CD-ROM stat {:02X}", self.result_fifo[self.result_idx]);

        self.int_queue.push_back(CD_ROM_INT {
            num: int,
            delay: delay.unwrap_or(AVERAGE_IRQ_DELAY),
            func: func,
        });
    }

    pub fn get_id(&mut self) {
        self.send_status(3, None, Some(Self::get_id_second_response));
    }

    pub fn get_id_second_response(&mut self) {
        self.result_idx = 0;
        self.result_size = 8;
        *self.result_fifo[self.result_idx..].first_chunk_mut().unwrap() = DISK_MODE1;
        // self.status.insert(CD_ROM_STATUS::SHELL);
        
        self.int_queue.push_back(CD_ROM_INT {
            num: 2,
            delay: ID_SECOND_DELAY,
            func: None
        });
    }
}

const ID_SECOND_DELAY: usize = 0x4A00;

#[allow(unused)]
const NO_DISK: [u8; 8] = [0x08, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
#[allow(unused)]
const DISK_MODE1: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x53, 0x43, 0x45, 0x41];