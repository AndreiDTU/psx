use std::hint::unreachable_unchecked;

use crate::gpu::{primitives::vertex::Vertex, GPU};

impl GPU {
    #[inline(always)]
    pub fn gp1_command(&mut self, word: u32) {
        match word >> 24 {
            0x00 => self.reset_gpu(),
            0x01 => self.gp0_parameters.clear(),
            0x02 => self.gpu_status.set_interrupt_request(0),
            0x03 => self.gpu_status.set_display_disable(word as u8 & 1),
            0x04 => self.gpu_status.set_dma_direction(word as u8 & 3),
            0x05 => self.set_display_area_start(word),
            0x06 => self.set_horizontal_display_range(word),
            0x07 => self.set_vertical_display_range(word),
            0x08 => self.set_display_mode(word),
            0x10..=0x1F => self.read_internal_register(word),
            _ => panic!("GP1 command not yet implemented! {word:08X}"),
        }
    }

    fn reset_gpu(&mut self) {
        self.gp0_parameters.clear();
        self.gpu_status.set_interrupt_request(0);
        self.gpu_status.set_display_disable(1);
        self.gpu_status.set_dma_direction(0);
        self.set_display_area_start(0);
        self.set_horizontal_display_range(0);
        self.set_vertical_display_range(0);
        self.set_display_mode(0);

        self.set_texpage(0);
        self.set_tex_window(0);
        self.set_drawing_area_top_left(0);
        self.set_drawing_area_bottom_right(0);
        self.set_drawing_offset(0);
        self.set_mask_bit_setting(0);
    }

    fn set_display_area_start(&mut self, word: u32) {
        let x = word & 0x3FF;
        let y = (word >> 10) & 0x1FF;

        self.display_area_start = (x, y).into();
    }

    fn set_horizontal_display_range(&mut self, word: u32) {
        let x0 = word & 0xFFF;
        let x1 = (word >> 12) & 0xFFF;

        self.display_range.0 = Vertex { coords: self.display_range.0.coords.with_x(x0 as i32) };
        self.display_range.1 = Vertex { coords: self.display_range.1.coords.with_x(x1 as i32) };
    }

    fn set_vertical_display_range(&mut self, word: u32) {
        let y0 = word & 0x3FF;
        let y1 = word & 0x3FF;

        self.display_range.0 = Vertex { coords: self.display_range.0.coords.with_y(y0 as i32) };
        self.display_range.1 = Vertex { coords: self.display_range.1.coords.with_y(y1 as i32) };
    }

    fn set_display_mode(&mut self, word: u32) {
        self.gpu_status.set_horizontal_resolution_1(((word >> 0) & 3) as u8);
        self.gpu_status.set_vertical_resolution(((word >> 2) & 1) as u8);
        self.gpu_status.set_video_mode(((word >> 3) & 1) as u8);
        self.gpu_status.set_display_area_color_depth(((word >> 4) & 1) as u8);
        self.gpu_status.set_vertical_interlace(((word >> 5) & 1) as u8);
        self.gpu_status.set_horizontal_resolution_2(((word >> 6) & 1) as u8);
        self.gpu_status.set_flip_screen_horizontally(((word >> 7) & 1) as u8);
    }

    fn read_internal_register(&mut self, word: u32) {
        self.gpu_read |= match word & 7 {
            0x00 | 0x01 | 0x06 | 0x07 => 0,
            0x02 => self.tex_window & 0x000F_FFFF,
            0x03 => u32::from(self.drawing_area.0) & 0x0007_FFFF_u32,
            0x04 => u32::from(self.drawing_area.1) & 0x0007_FFFF_u32,
            0x05 => u32::from(self.drawing_offset) & 0x003F_FFFF_u32,
            _ => unsafe { unreachable_unchecked() }
        }
    }
}