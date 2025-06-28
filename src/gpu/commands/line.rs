use crate::gpu::{primitives::{color::Color, vertex::Vertex}, GP0_State, ParametrizedCommand, GPU};

impl GPU {
    pub fn set_line_state(&mut self, word: u32) -> GP0_State {
        let gouraud = word & (1 << 28) != 0;
        let polyline = word & (1 << 27) != 0;

        if polyline {
            GP0_State::ReceivingPolyLineParameters {
                color_word: false,
                gouraud,
                command: ParametrizedCommand::Line(word)
            }
        } else {
            GP0_State::ReceivingParameters {
                idx: 1,
                expected: 2 + gouraud as usize,
                command: ParametrizedCommand::Line(word),
            }
        }
    }

    pub fn draw_monochrome_line(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_monochrome_line(v0, v1, word);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_monochrome_line(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_monochrome_line(v0, v1, word);

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_line(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_gouraud_line(v0, v1, c0, c1);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_line(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_gouraud_line(v0, v1, c0, c1);

        GP0_State::CommandStart
    }

    pub fn draw_monchrome_polyline(&mut self, word: u32) -> GP0_State {
        let vertices: Vec<u32> = self.gp0_parameters.drain(..).collect();
        vertices
            .windows(2)
            .map(|w| (&w[0], &w[1]))
            .for_each(|(v0, v1)| {self.write_monochrome_line((*v0).into(), (*v1).into(), word);});

        GP0_State::CommandStart
    }

    pub fn draw_transparent_monchrome_polyline(&mut self, word: u32) -> GP0_State {
        let vertices: Vec<u32> = self.gp0_parameters.drain(..).collect();
        vertices
            .windows(2)
            .map(|w| (&w[0], &w[1]))
            .for_each(|(v0, v1)| {self.write_transparent_monochrome_line((*v0).into(), (*v1).into(), word);});

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_polyline(&mut self, word: u32) -> GP0_State {
        self.gp0_parameters.push_front(word);
        let raw: Vec<u32> = self.gp0_parameters.drain(..).collect();

        let vertices: Vec<(Color, Vertex)> = raw
            .chunks_exact(2)
            .map(|chunk| {
                let color = Color::compress_color_depth(chunk[0]).into();
                let coords = chunk[1].into();

                (color, coords)
            })
            .collect();

        vertices
            .windows(2)
            .map(|w| (&w[0], &w[1]))
            .for_each(|((c0, v0), (c1, v1))| {self.write_gouraud_line((*v0).into(), (*v1).into(), *c0, *c1);});

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_polyline(&mut self, word: u32) -> GP0_State {
        self.gp0_parameters.push_front(word);
        let raw: Vec<u32> = self.gp0_parameters.drain(..).collect();

        let vertices: Vec<(Color, Vertex)> = raw
            .chunks_exact(2)
            .map(|chunk| {
                let color = Color::compress_color_depth(chunk[0]).into();
                let coords = chunk[1].into();

                (color, coords)
            })
            .collect();

        vertices
            .windows(2)
            .map(|w| (&w[0], &w[1]))
            .for_each(|((c0, v0), (c1, v1))| {self.write_transparent_gouraud_line((*v0).into(), (*v1).into(), *c0, *c1);});

        GP0_State::CommandStart
    }

    fn write_monochrome_line(&mut self, v0: Vertex, v1: Vertex, color: u32) {
        for pixel in v0.bresenham_line(v1) {
            let coords = pixel.translate(self.drawing_offset).into();

            self.draw_pixel(color, coords);
        }
    }

    fn write_transparent_monochrome_line(&mut self, v0: Vertex, v1: Vertex, color: u32) {
        for pixel in v0.bresenham_line(v1) {
            let coords = pixel.translate(self.drawing_offset).into();

            self.draw_transparent_pixel(color, coords);
        }
    }

    fn write_gouraud_line(&mut self, v0: Vertex, v1: Vertex, c0: Color, c1: Color) {
        for (pixel, mut color) in v0.bresenham_line_gouraud(v1, c0, c1) {
            let coords = pixel.translate(self.drawing_offset).into();
            color.apply_dithering(pixel);
            
            self.draw_pixel(color.into(), coords);
        }
    }

    fn write_transparent_gouraud_line(&mut self, v0: Vertex, v1: Vertex, c0: Color, c1: Color) {
        for (pixel, mut color) in v0.bresenham_line_gouraud(v1, c0, c1) {
            let coords = pixel.translate(self.drawing_offset).into();
            color.apply_dithering(pixel);
            
            self.draw_transparent_pixel(color.into(), coords);
        }
    }
}