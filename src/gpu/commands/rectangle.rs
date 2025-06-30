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

    pub fn draw_single_pixel_monochrome_rect(&mut self, word: u32) -> GP0_State {
        let coords: u32 = Vertex::from(self.gp0_parameters.pop_front().unwrap()).translate(self.drawing_offset).into();
        self.draw_pixel(word, coords);

        GP0_State::CommandStart
    }
}