use std::hint::unreachable_unchecked;

use glam::{U8Vec3, UVec2};

use crate::gpu::{primitives::{color::Color, interpolate_uv_coords, vertex::Vertex}, GP0_State, ParametrizedCommand, GPU};

impl GPU {
    pub fn set_polygon_state(&mut self, word: u32) -> GP0_State {
        // println!("draw polygon {word:08X}");

        let vertices = 3 + ((word >> 27) & 1);
        let color_words = (vertices - 1) * ((word >> 28) & 1);
        let texture_words = vertices * ((word >> 26) & 1);

        GP0_State::ReceivingParameters {
            idx: 1,
            expected: (vertices + color_words + texture_words) as usize,
            command: ParametrizedCommand::Polygon(word)
        }
    }

    pub fn draw_monochrome_tri(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_monochrome_tri(v0, v1, v2, word);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_monochrome_tri(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_monochrome_tri(v0, v1, v2, word);

        GP0_State::CommandStart
    }

    pub fn draw_modulated_tri(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_modulated_tri(v0, v1, v2, word, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_textured_tri(&mut self) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_textured_tri(v0, v1, v2, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_modulated_tri(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_transparent_modulated_tri(v0, v1, v2, word, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_textured_tri(&mut self) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_transparent_textured_tri(v0, v1, v2, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_monochrome_quad(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_monochrome_tri(v0, v1, v2, word);
        self.write_transparent_monochrome_tri(v1, v2, v3, word);

        GP0_State::CommandStart
    }

    pub fn draw_monochrome_quad(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_monochrome_tri(v0, v1, v2, word);
        self.write_monochrome_tri(v1, v2, v3, word);

        GP0_State::CommandStart
    }

    pub fn draw_modulated_quad(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_modulated_tri(v0, v1, v2, word, uv0, uv1, uv2, clut, page);
        self.write_modulated_tri(v1, v2, v3, word, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_textured_quad(&mut self) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_textured_tri(v0, v1, v2, uv0, uv1, uv2, clut, page);
        self.write_textured_tri(v1, v2, v3, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_modulated_quad(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_transparent_modulated_tri(v0, v1, v2, word, uv0, uv1, uv2, clut, page);
        self.write_transparent_modulated_tri(v1, v2, v3, word, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_textured_quad(&mut self) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_transparent_textured_tri(v0, v1, v2, uv0, uv1, uv2, clut, page);
        self.write_transparent_textured_tri(v1, v2, v3, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_tri(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_gouraud_tri(v0, v1, v2, c0, c1, c2);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_tri(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_gouraud_tri(v0, v1, v2, c0, c1, c2);

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_modulated_tri(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_gouraud_modulated_tri(v0, v1, v2, c0, c1, c2, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_modulated_tri(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);

        self.write_transparent_gouraud_modulated_tri(v0, v1, v2, c0, c1, c2, uv0, uv1, uv2, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_quad(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c3: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_gouraud_tri(v0, v1, v2, c0, c1, c2);
        self.write_gouraud_tri(v1, v2, v3, c1, c2, c3);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_quad(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c3: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_transparent_gouraud_tri(v0, v1, v2, c0, c1, c2);
        self.write_transparent_gouraud_tri(v1, v2, v3, c1, c2, c3);

        GP0_State::CommandStart
    }

    pub fn draw_gouraud_modulated_quad(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let c3: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_gouraud_modulated_tri(v0, v1, v2, c0, c1, c2, uv0, uv1, uv2, clut, page);
        self.write_gouraud_modulated_tri(v1, v2, v3, c1, c2, c3, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    pub fn draw_transparent_gouraud_modulated_quad(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t0 = self.gp0_parameters.pop_front().unwrap();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t1 = self.gp0_parameters.pop_front().unwrap();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t2 = self.gp0_parameters.pop_front().unwrap();
        let c3: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let t3 = self.gp0_parameters.pop_front().unwrap();

        let clut = t0 >> 16;
        let page = t1 >> 16;

        let uv0 = (t0 & 0xFF, (t0 >> 8) & 0xFF);
        let uv1 = (t1 & 0xFF, (t1 >> 8) & 0xFF);
        let uv2 = (t2 & 0xFF, (t2 >> 8) & 0xFF);
        let uv3 = (t3 & 0xFF, (t3 >> 8) & 0xFF);

        self.write_transparent_gouraud_modulated_tri(v0, v1, v2, c0, c1, c2, uv0, uv1, uv2, clut, page);
        self.write_transparent_gouraud_modulated_tri(v1, v2, v3, c1, c2, c3, uv1, uv2, uv3, clut, page);

        GP0_State::CommandStart
    }

    fn write_monochrome_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, color: u32) {
        let mut v0 = v0;
        let mut v1 = v1;

        Vertex::ensure_vertex_order(&mut v0, &mut v1, v2);

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for x in min_x..max_x {
            for y in min_y..max_y {
                let pixel: Vertex = (x, y).into();
                if pixel.is_inside_triangle(v0, v1, v2) {
                    let coords = pixel.translate(self.drawing_offset).into();

                    self.draw_pixel(color, coords);
                }
            }
        }
    }

    fn write_transparent_monochrome_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, color: u32) {
        let mut v0 = v0;
        let mut v1 = v1;

        Vertex::ensure_vertex_order(&mut v0, &mut v1, v2);

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for x in min_x..max_x {
            for y in min_y..max_y {
                let pixel: Vertex = (x, y).into();
                if pixel.is_inside_triangle(v0, v1, v2) {
                    let coords = pixel.translate(self.drawing_offset).into();

                    self.draw_transparent_pixel(color, coords, self.gpu_status.semi_transparency());
                }
            }
        }
    }

    fn write_gouraud_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, c0: Color, c1: Color, c2: Color) {
        let mut v0 = v0;
        let mut v1 = v1;

        let mut c0 = c0;
        let mut c1 = c1;
        
        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut c0, &mut c1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        if self.gpu_status.dither_24bit_to_15bit() != 0 {
            for x in min_x..max_x {
                for y in min_y..max_y {
                    let pixel: Vertex = (x, y).into();
                    if pixel.is_inside_triangle(v0, v1, v2) {
                        let coords = pixel.translate(self.drawing_offset).into();
                        let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                        let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).apply_dithering(pixel).into();

                        self.draw_pixel(color, coords);
                    }
                }
            }
        } else {
            for x in min_x..max_x {
                for y in min_y..max_y {
                    let pixel: Vertex = (x, y).into();
                    if pixel.is_inside_triangle(v0, v1, v2) {
                        let coords = pixel.translate(self.drawing_offset).into();
                        let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                        let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).into();

                        self.draw_pixel(color, coords);
                    }
                }
            }
        }
    }

    fn write_transparent_gouraud_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, c0: Color, c1: Color, c2: Color) {
        let mut v0 = v0;
        let mut v1 = v1;

        let mut c0 = c0;
        let mut c1 = c1;
        
        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut c0, &mut c1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        if self.gpu_status.dither_24bit_to_15bit() != 0 {
            for x in min_x..max_x {
                for y in min_y..max_y {
                    let pixel: Vertex = (x, y).into();
                    if pixel.is_inside_triangle(v0, v1, v2) {
                        let coords = pixel.translate(self.drawing_offset).into();
                        let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                        let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).apply_dithering(pixel).into();

                        self.draw_transparent_pixel(color, coords, self.gpu_status.semi_transparency());
                    }
                }
            }
        } else {
            for x in min_x..max_x {
                for y in min_y..max_y {
                    let pixel: Vertex = (x, y).into();
                    if pixel.is_inside_triangle(v0, v1, v2) {
                        let coords = pixel.translate(self.drawing_offset).into();
                        let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                        let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).into();

                        self.draw_transparent_pixel(color, coords, self.gpu_status.semi_transparency());
                    }
                }
            }
        }
    }

    fn write_modulated_tri(
        &mut self, 
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        color: u32, 
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            let px_idx = [
                                tex_color,
                                tex_color >> 4,
                                tex_color >> 8,
                                tex_color >> 12,
                            ].map(|idx| {idx & 0xF});

                            let color = px_idx.map(|idx| {
                                Color::from(self.vram.read16(clut_addr + ((idx as u32) << 1))).modulate(color.into())
                            });
                            
                            color.iter()
                                .enumerate()
                                .for_each(|(i, color)| {
                                    if color.rgb != U8Vec3::splat(0) {
                                        let pixel: Vertex = (px + i as i32, py).into();
                                        let coords: u32 = pixel.into();
                                        self.draw_pixel((*color).into(), coords);
                                    }
                                });
                        }
                    }
                }
                1 => todo!(),
                2 => todo!(),
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }

    fn write_textured_tri(
        &mut self, 
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
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
                }
                1 => todo!(),
                2 => {
                    for x in min_x..max_x {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            if tex_color != 0 {
                                self.draw_compressed_pixel(tex_color, pixel.into());
                            }
                        }
                    }
                },
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }

    fn write_transparent_modulated_tri(
        &mut self, 
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        color: u32, 
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let semi_transparency = ((page >> 5) & 3) as u8;
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            let px_idx = [
                                tex_color,
                                tex_color >> 4,
                                tex_color >> 8,
                                tex_color >> 12,
                            ].map(|idx| {idx & 0xF});

                            let color = px_idx.map(|idx| {
                                Color::from(self.vram.read16(clut_addr + ((idx as u32) << 1))).modulate(color.into())
                            });
                            
                            color.iter()
                                .enumerate()
                                .for_each(|(i, color)| {
                                    if color.rgb != U8Vec3::splat(0) {
                                        let pixel: Vertex = (px + i as i32, py).into();
                                        let coords: u32 = pixel.into();
                                        self.draw_transparent_pixel((*color).into(), coords, semi_transparency);
                                    }
                                });
                        }
                    }
                }
                1 => todo!(),
                2 => todo!(),
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }

    fn write_transparent_textured_tri(
        &mut self, 
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let semi_transparency = ((page >> 5) & 3) as u8;
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
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
                                        self.draw_compressed_transparent_pixel(*color, coords, semi_transparency);
                                    }
                                });
                        }
                    }
                }
                1 => todo!(),
                2 => {
                    for x in min_x..max_x {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            if tex_color != 0 {
                                self.draw_compressed_transparent_pixel(tex_color, pixel.into(), semi_transparency);
                            }
                        }
                    }
                },
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }

    fn write_gouraud_modulated_tri(
        &mut self,
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        c0: Color,
        c1: Color,
        c2: Color,
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        let mut c0 = c0;
        let mut c1 = c1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
            std::mem::swap(&mut c0, &mut c1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();
                            let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let barycentric_uv_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_uv_coords, [uv0, uv1, uv2]) + base).to_array();

                            let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).apply_dithering(pixel);
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            let px_idx = [
                                tex_color,
                                tex_color >> 4,
                                tex_color >> 8,
                                tex_color >> 12,
                            ].map(|idx| {idx & 0xF});

                            let color = px_idx.map(|idx| {
                                Color::from(self.vram.read16(clut_addr + ((idx as u32) << 1))).modulate(color.into())
                            });
                            
                            color.iter()
                                .enumerate()
                                .for_each(|(i, color)| {
                                    if color.rgb != U8Vec3::splat(0) {
                                        let pixel: Vertex = (px + i as i32, py).into();
                                        let coords: u32 = pixel.into();
                                        self.draw_pixel((*color).into(), coords);
                                    }
                                });
                        }
                    }
                }
                1 => todo!(),
                2 => {
                    for x in min_x..max_x {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();
                            let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let barycentric_uv_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_uv_coords, [uv0, uv1, uv2]) + base).to_array();

                            let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).apply_dithering(pixel);
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            if tex_color != 0 {
                                self.draw_pixel(Color::from(tex_color).modulate(color.into()).into(), pixel.into());
                            }
                        }
                    }
                }
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }

    fn write_transparent_gouraud_modulated_tri(
        &mut self,
        v0: Vertex,
        v1: Vertex,
        v2: Vertex,
        c0: Color,
        c1: Color,
        c2: Color,
        uv0: (u32, u32),
        uv1: (u32, u32),
        uv2: (u32, u32),
        clut: u32,
        page: u32,
    ) {
        let base_x = (page & 0xF) << 6;
        let base_y = ((page >> 4) & 1) << 8;
        let base = UVec2::from((base_x, base_y));
        let semi_transparency = ((page >> 5) & 3) as u8;
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        let [min_x, max_x, min_y, max_y] = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1).to_array();

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel: Vertex = (x, y).into();
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let [px, py] = pixel.translate(self.drawing_offset).coords.to_array();

                            let tex_pixel: Vertex = (min_x + ((x - min_x) >> 2), y).into();

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let [u, v] = (interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]) + base).to_array();
                            let tex_color = self.vram.read16(((v << 10) + u) << 1);

                            let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]);

                            let px_idx = [
                                tex_color,
                                tex_color >> 4,
                                tex_color >> 8,
                                tex_color >> 12,
                            ].map(|idx| {idx & 0xF});

                            let color = px_idx.map(|idx| {
                                Color::from(self.vram.read16(clut_addr + ((idx as u32) << 1))).modulate(color.into())
                            });
                            
                            color.iter()
                                .enumerate()
                                .for_each(|(i, color)| {
                                    if color.rgb != U8Vec3::splat(0) {
                                        let pixel: Vertex = (px + i as i32, py).into();
                                        let coords: u32 = pixel.into();
                                        self.draw_transparent_pixel((*color).into(), coords, semi_transparency);
                                    }
                                });
                        }
                    }
                }
                1 => todo!(),
                2 => todo!(),
                3 => panic!("Reserved color depth"),
                _ => unsafe { unreachable_unchecked() }
            };
        }
    }
}