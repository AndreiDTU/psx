use crate::gpu::{primitives::{color::Color, vertex::Vertex}, GP0_State, ParametrizedCommand, GPU};

impl GPU {
    pub fn set_rectangle_state(&mut self, word: u32) -> GP0_State {
        let mut expected = 1;
        expected += (word & (1 << 26) != 0) as usize;
        expected += (word & 0x1C00_0000 == 0) as usize;

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
                self.draw_transparent_pixel(word, coords);
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
        self.draw_transparent_pixel(word, coords);

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
                self.draw_transparent_pixel(word, coords);
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
                self.draw_transparent_pixel(word, coords);
            }
        }

        GP0_State::CommandStart
    }
}