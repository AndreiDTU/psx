use crate::cd_rom::{SecondResponse, CD_ROM, CD_ROM_MODE, CD_ROM_STATUS, RDDATA};

impl CD_ROM {
    pub fn readN(&mut self) {
        self.send_status(3);
        self.current_sector = *self.disk.get(&self.read_addr).unwrap();

        self.status.insert(CD_ROM_STATUS::READ);
        self.status.remove(CD_ROM_STATUS::SEEK);
        self.status.remove(CD_ROM_STATUS::PLAY);

        self.second_response = SecondResponse::ReadN;
    }

    pub fn readN_second_response(&mut self) {
        self.registers[RDDATA] = self.current_sector[self.sector_pointer];
        self.sector_pointer += 1;

        let size = 0x800 + (self.mode.contains(CD_ROM_MODE::SECTOR_SIZE) as usize) * 0x124;

        if self.sector_pointer == size {
            self.sector_pointer = 0;
            self.read_addr.increment();
            self.current_sector = *self.disk.get(&self.read_addr).unwrap();
        }

        let speed = INT1_RATE[self.mode.contains(CD_ROM_MODE::SPEED) as usize];
        self.irq_delay = speed;
        self.schedule_int(1);
    }
}

const INT1_RATE: [usize; 2] = [0x0006_E1CD, 0x0003_6CD2];