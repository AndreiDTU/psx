use crate::cd_rom::{bin::sector::Sector, CD_ROM, CD_ROM_INT, CD_ROM_MODE, CD_ROM_STATUS};

impl CD_ROM {
    pub fn readN(&mut self) {
        self.sector_pointer = 0;
        self.read_addr = self.seek_target;
        self.send_status(3, None, Some(Self::readN_second_response));
    }

    pub fn readN_second_response(&mut self) {
        println!("{:#?}", self.read_addr);
        match self.disk.get(&self.read_addr) {
            Some(sector) => {
                self.load_sector(*sector);

                self.status.insert(CD_ROM_STATUS::READ);
                self.status.remove(CD_ROM_STATUS::SEEK);
                self.status.remove(CD_ROM_STATUS::PLAY);

                let speed = INT1_RATE[self.mode.contains(CD_ROM_MODE::SPEED) as usize];
                self.send_status(1, Some(speed), Some(Self::readN_second_response));

                self.read_addr.increment();
            }
            None => {
                let speed = INT1_RATE[self.mode.contains(CD_ROM_MODE::SPEED) as usize];
                self.int_queue.push_back(CD_ROM_INT {
                    num: 4,
                    delay: speed,
                    func: None,
                });
            }
        }
    }

    fn load_sector(&mut self, sector: Sector) {
        let read_func = RDDATA_READ[self.mode.contains(CD_ROM_MODE::SECTOR_SIZE) as usize];
        self.sector_buffer[0] = self.sector_buffer[1];
        self.sector_buffer[1] = Some((sector, read_func));
        
        if self.sector_buffer[0].is_none() {
            self.sector_buffer[0] = self.sector_buffer[1];
        }
    }
}

const INT1_RATE: [usize; 2] = [0x0006_E1CD, 0x0003_6CD2];
const RDDATA_READ: [fn(&mut CD_ROM) -> u8; 2] = [CD_ROM::read_0x800, CD_ROM::read_0x924];