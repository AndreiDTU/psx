use crate::gpu::{GP0_State, GPU, GPUSTAT};

impl GPU {
    pub fn set_texpage(&mut self, word: u32) -> GP0_State {
        let mut bytes = self.gpu_status.bytes;
        bytes[0] = word as u8;
        bytes[1] = (bytes[1] & 0xF8) | (((word >> 8) as u8) & 7);
        self.gpu_status = GPUSTAT::from_bytes(bytes);

        GP0_State::CommandStart
    }

    pub fn set_tex_window(&mut self, word: u32) -> GP0_State {
        self.tex_window = word;

        GP0_State::CommandStart
    }

    pub fn set_drawing_area_top_left(&mut self, word: u32) -> GP0_State {
        let x = (word & 0x3FF) as i32;
        let y = ((word >> 10) & 0x1FF) as i32;

        self.drawing_area.0 = (x, y).into();

        GP0_State::CommandStart
    }

    pub fn set_drawing_area_bottom_right(&mut self, word: u32) -> GP0_State {
        let x = (word & 0x3FF) as i32;
        let y = ((word >> 10) & 0x1FF) as i32;

        self.drawing_area.1 = (x, y).into();

        GP0_State::CommandStart
    }

    pub fn set_drawing_offset(&mut self, word: u32) -> GP0_State {
        let mut x = word & 0x7FF;
        let mut y = (word >> 11) & 0x7FF;

        if x & (1 << 10) != 0 {x |= 0xFFFF_F800}
        if y & (1 << 10) != 0 {y |= 0xFFFF_F800}
        
        self.drawing_offset = (x, y).into();

        GP0_State::CommandStart
    }

    pub fn set_mask_bit_setting(&mut self, word: u32) -> GP0_State {
        self.gpu_status.set_set_mask_bit(((word >> 0) & 1) as u8);
        self.gpu_status.set_check_mask(((word >> 1) & 1) as u8);

        GP0_State::CommandStart
    }
}