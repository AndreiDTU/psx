use crate::gpu::{GP0_State, BlitFields, GPU};

impl GPU {
    pub fn initialize_cpu_vram_copy(&mut self) -> GP0_State {
        let coords = self.gp0_parameters.pop_front().unwrap();
        let size = self.gp0_parameters.pop_front().unwrap();

        let vram_x = (coords & 0x3FF) as u16;
        let vram_y = ((coords >> 16) & 0x1FF) as u16;

        let mut width = (size & 0x3FF) as u16;
        if width == 0 {width = 1024}

        let mut height = ((size >> 16) & 0x1FF) as u16;
        if height == 0 {height = 512}

        GP0_State::ReceivingData(
            BlitFields { vram_x: 
                vram_x, vram_y,
                width, height, 
                
                current_row: 0, current_col: 0
            }
        )
    }

    pub fn process_cpu_vram_copy(&mut self, word: u32) -> GP0_State {
        let GP0_State::ReceivingData(mut fields) = self.gp0_mode else {unreachable!()};

        for i in 0..=1 {
            let halfword = (word >> (16 * i)) as u16;

            let vram_row = ((fields.vram_y + fields.current_row) & 0x1FF) as u32;
            let vram_col = ((fields.vram_x + fields.current_col) & 0x3FF) as u32;

            let vram_addr = 2 * (1024 * vram_row + vram_col);

            self.vram.write16(vram_addr, halfword);

            fields.current_col += 1;
            if fields.current_col == fields.width {
                fields.current_col = 0;
                fields.current_row += 1;

                if fields.current_row == fields.height {
                    return GP0_State::CommandStart;
                }
            }
        }

        GP0_State::ReceivingData(fields)
    }

    pub fn initialize_vram_cpu_copy(&mut self) -> GP0_State {
        let coords = self.gp0_parameters.pop_front().unwrap();
        let size = self.gp0_parameters.pop_front().unwrap();

        let vram_x = (coords & 0x3FF) as u16;
        let vram_y = ((coords >> 16) & 0x1FF) as u16;

        let mut width = (size & 0x3FF) as u16;
        if width == 0 {width = 1024}

        let mut height = ((size >> 16) & 0x1FF) as u16;
        if height == 0 {height = 512}

        self.gpu_read_transfer = Some(GP0_State::SendingData(
            BlitFields { vram_x: 
                vram_x, vram_y,
                width, height, 
                
                current_row: 0, current_col: 0
            }
        ));

        GP0_State::CommandStart
    }

    pub fn process_vram_cpu_copy(&mut self) {
        let Some(GP0_State::SendingData(mut fields)) = self.gpu_read_transfer else {unreachable!()};

        let vram_row = ((fields.vram_y + fields.current_row) & 0x1FF) as u32;
        let vram_col = ((fields.vram_x + fields.current_col) & 0x3FF) as u32;

        let vram_addr = 2 * (1024 * vram_row + vram_col);

        self.gpu_read = self.vram.read32(vram_addr);

        fields.current_col += 1;
        if fields.current_col == fields.width {
            fields.current_col = 0;
            fields.current_row += 1;

            if fields.current_row == fields.height {
                self.gpu_read_transfer = None;
                return;
            }
        }

        self.gpu_read_transfer = Some(GP0_State::SendingData(fields));
    }
}