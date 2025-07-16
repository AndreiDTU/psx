use std::hint::unreachable_unchecked;

use crate::gpu::{primitives::vertex::Vertex, GP0_State, ParametrizedCommand, GPU};

impl GPU {
    pub fn set_rectangle_state(&mut self, word: u32) -> GP0_State {
        let mut expected = 1;
        expected += (word & (1 << 26) != 0) as usize;
        expected += (word & 0x1800_0000 == 0) as usize;

        GP0_State::ReceivingParameters {
            idx: 1,
            expected: expected,
            command: ParametrizedCommand::Rectangle(word),
        }
    }

    pub fn draw_variable_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);
        let size = self.gp0_parameters.pop_front().unwrap();

        let size_y = (size >> 16) & 0x1FF;
        let size_x = size & 0x3FF;

        for x in top_left.coords.x..(top_left.coords.x + size_x as i32) {
            for y in top_left.coords.y..(top_left.coords.y + size_y as i32) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_pixel(word, coords);
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_transparent_variable_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);
        let size = self.gp0_parameters.pop_front().unwrap();

        let size_y = (size >> 16) & 0x1FF;
        let size_x = size & 0x3FF;

        for x in top_left.coords.x..(top_left.coords.x + size_x as i32) {
            for y in top_left.coords.y..(top_left.coords.y + size_y as i32) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_transparent_pixel(word, coords, self.gpu_status.semi_transparency());
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_variable_textured_rect(&mut self) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);
        let clut_uv = self.gp0_parameters.pop_front().unwrap();
        let size = self.gp0_parameters.pop_front().unwrap();

        let size_y = (size >> 16) & 0x1FF;
        let size_x = size & 0x3FF;

        let base_x = (self.gpu_status.texture_page_x_base() as u32) << 6;
        let base_y = (self.gpu_status.texture_page_y_base_1() as u32) << 8;
        let tex_page_color_depth = ((self.gpu_status.texture_page_colors() as u32) >> 7) & 3;

        let clut = clut_uv >> 16;
        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let (base_u, base_v) = ((clut_uv & 0xFF) + base_x, ((clut_uv >> 8) & 0xFF) + base_y);

        for y in top_left.coords.y..(top_left.coords.y + size_y as i32) {
            match tex_page_color_depth {
                0 => {
                    for x in (top_left.coords.x..(top_left.coords.x + size_x as i32)).step_by(4) {
                        let pixel = Vertex::from((x, y));
                        let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                        let (u, v) = (base_u + (((x - top_left.coords.x) as u32) >> 2), base_v + ((y - top_left.coords.y) as u32));
                        let tex_color = self.vram.read16(((v << 10) + u) << 1);

                        let px_idx = [
                            tex_color,
                            tex_color >> 4,
                            tex_color >> 8,
                            tex_color >> 12,
                        ].map(|idx| {idx & 0xF});

                        let color = px_idx.map(|idx| {self.vram.read16(clut_addr + ((idx as u32) << 1))});
                        
                        color.iter()
                            .enumerate()
                            .for_each(|(i, color)| {
                                if *color != 0 {
                                    let pixel: Vertex = (px + i as i32, py).into();
                                    let coords: u32 = pixel.into();
                                    self.draw_compressed_pixel(*color, coords);
                                }
                            });
                    }
                },
                1 => todo!(),
                2 => todo!(),
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_textured_8x8_rect(&mut self) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);
        let clut_uv = self.gp0_parameters.pop_front().unwrap();

        let base_x = (self.gpu_status.texture_page_x_base() as u32) << 6;
        let base_y = (self.gpu_status.texture_page_y_base_1() as u32) << 8;
        let tex_page_color_depth = ((self.gpu_status.texture_page_colors() as u32) >> 7) & 3;

        let clut = clut_uv >> 16;
        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let (base_u, base_v) = ((clut_uv & 0xFF) + base_x, ((clut_uv >> 8) & 0xFF) + base_y);

        for y in top_left.coords.y..(top_left.coords.y + 8) {
            match tex_page_color_depth {
                0 => {
                    for x in top_left.coords.x..(top_left.coords.x + 8) {
                        let pixel = Vertex::from((x, y));
                        let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                        let (u, v) = (base_u + (((x - top_left.coords.x) as u32) >> 2), base_v + ((y - top_left.coords.y) as u32));
                        let tex_color = self.vram.read16(((v << 10) + u) << 1);

                        let px_idx = [
                            tex_color,
                            tex_color >> 4,
                            tex_color >> 8,
                            tex_color >> 12,
                        ].map(|idx| {idx & 0xF});

                        let color = px_idx.map(|idx| {self.vram.read16(clut_addr + ((idx as u32) << 1))});
                        
                        color.iter()
                            .enumerate()
                            .for_each(|(i, color)| {
                                if *color != 0 {
                                    let pixel: Vertex = (px + i as i32, py).into();
                                    let coords: u32 = pixel.into();
                                    self.draw_compressed_pixel(*color, coords);
                                }
                            });
                    }
                }
                1 => todo!(),
                2 => todo!(),
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_single_pixel_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let coords: u32 = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset).into();
        self.draw_pixel(word, coords);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_single_pixel_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let coords: u32 = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset).into();
        self.draw_transparent_pixel(word, coords, self.gpu_status.semi_transparency());

        GP0_State::CommandStart
    }

    pub fn draw_8x8_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);

        for x in top_left.coords.x..(top_left.coords.x + 8) {
            for y in top_left.coords.y..(top_left.coords.y + 8) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_pixel(word, coords);
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_transparent_8x8_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);

        for x in top_left.coords.x..(top_left.coords.x + 8) {
            for y in top_left.coords.y..(top_left.coords.y + 8) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_transparent_pixel(word, coords, self.gpu_status.semi_transparency());
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_16x16_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);

        for x in top_left.coords.x..(top_left.coords.x + 16) {
            for y in top_left.coords.y..(top_left.coords.y + 16) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_pixel(word, coords);
            }
        }

        GP0_State::CommandStart
    }

    pub fn draw_transparent_16x16_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset);

        for x in top_left.coords.x..(top_left.coords.x + 16) {
            for y in top_left.coords.y..(top_left.coords.y + 16) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_transparent_pixel(word, coords, self.gpu_status.semi_transparency());
            }
        }

        GP0_State::CommandStart
    }
}