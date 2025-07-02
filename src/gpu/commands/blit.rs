use crate::gpu::{GP0_State, BlitFields, GPU};

impl GPU {
    pub fn initialize_vram_vram_copy(&mut self) -> GP0_State {
        let source_coords = self.gp0_parameters.pop_front().unwrap();
        let dest_coords = self.gp0_parameters.pop_front().unwrap();
        let size = self.gp0_parameters.pop_front().unwrap();

        let source_x = source_coords & 0x3FF;
        let source_y = (source_coords >> 16) & 0x1FF;

        let dest_x = dest_coords & 0x3FF;
        let dest_y = (dest_coords >> 16) & 0x1FF;

        let mut width = size & 0x3FF;
        if width == 0 {width = 1024}

        let mut height = (size >> 16) & 0x1FF;
        if height == 0 {height = 512}

        for row_offset in 0..height {
            for col_offset in 0..width {
                let sx = source_x + col_offset;
                let sy = source_y + row_offset;
                let dx = dest_x + col_offset;
                let dy = dest_y + row_offset;

                let src_addr = ((sy << 10) + sx) << 1;
                let dst_addr = ((dy << 10) + dx) << 1;

                let value = self.vram.read32(src_addr);
                self.vram.write32(dst_addr, value);
            }
        }

        GP0_State::CommandStart
    }

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

        self.gpu_status.set_ready_to_send_VRAM_to_CPU(1);

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
                self.gpu_status.set_ready_to_send_VRAM_to_CPU(0);
                return;
            }
        }

        self.gpu_read_transfer = Some(GP0_State::SendingData(fields));
    }
}