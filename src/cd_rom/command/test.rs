use crate::cd_rom::{AVERAGE_IRQ_DELAY, CD_ROM, CD_ROM_INT};

impl CD_ROM {
    pub fn test(&mut self) {
        let sub_op = self.parameters.pop_front().unwrap();
        // println!("CD-ROM test sub-op: {sub_op:02X}");
        match sub_op {
            0x20 => self.test_version(),
            _ => panic!("CD-ROM test sub-op not yet implemented. {sub_op:02X}"),
        }
    }

    pub fn test_version(&mut self) {
        *self.result_fifo[self.result_idx..].first_chunk_mut().unwrap() = VERSION;
        self.result_size = 3;
        self.result_fifo_empty = false;

        self.int_queue.push_back(CD_ROM_INT {
            num: 3,
            delay: AVERAGE_IRQ_DELAY,
            func: None,
        });
    }
}

const VERSION: [u8; 4] = [0x94, 0x09, 0x19, 0xC0];