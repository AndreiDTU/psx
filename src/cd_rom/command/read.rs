use crate::cd_rom::{bin::sector::Sector, SecondResponse, CD_ROM, CD_ROM_MODE, CD_ROM_STATUS};

impl CD_ROM {
    pub fn readN(&mut self) {
        self.send_status(3);

        self.second_response = SecondResponse::ReadN;
    }

    pub fn readN_second_response(&mut self) {
        self.sector_pointer = 0;
        self.read_func = RDDATA_READ[self.mode.contains(CD_ROM_MODE::SECTOR_SIZE) as usize];
        match self.disk.get(&self.read_addr) {
            Some(sector) => {
                Self::load_sector(&mut self.sector_buffer, sector);

                self.status.insert(CD_ROM_STATUS::READ);
                self.status.remove(CD_ROM_STATUS::SEEK);
                self.status.remove(CD_ROM_STATUS::PLAY);

                let speed = INT1_RATE[self.mode.contains(CD_ROM_MODE::SPEED) as usize];
                self.irq_delay = speed;
                self.schedule_int(1);

                self.read_addr.increment();
            }
            None => {
                let speed = INT1_RATE[self.mode.contains(CD_ROM_MODE::SPEED) as usize];
                self.irq_delay = speed;
                self.schedule_int(4);
            }
        }
    }

    fn load_sector(sector_buffer: &mut [Option<Sector>; 2], sector: &Sector) {
        match sector_buffer[0] {
            None => sector_buffer[0] = Some(*sector),
            Some(_) => match sector_buffer[1] {
                None => sector_buffer[1] = Some(*sector),
                Some(_) => {
                    sector_buffer[0] = sector_buffer[1];
                    sector_buffer[1] = Some(*sector);
                }
            }
        }
    }
}

const INT1_RATE: [usize; 2] = [0x0006_E1CD, 0x0003_6CD2];
const RDDATA_READ: [fn(&mut CD_ROM) -> u8; 2] = [CD_ROM::read_0x800, CD_ROM::read_0x924];