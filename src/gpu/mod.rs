use std::{collections::VecDeque, hint::unreachable_unchecked};

use modular_bitfield::{bitfield, prelude::*};

use crate::{gpu::primitives::{color::Color, interpolate_uv_coords, vertex::Vertex}, ram::RAM};

const VRAM_SIZE: usize = 1024 * 1024;

pub mod primitives;

#[bitfield]
pub struct GPUSTAT {
    texture_page_x_base: B4,
    texture_page_y_base_1: B1,
    semi_transparency: B2,
    texture_page_colors: B2,
    dither_24bit_to_15bit: B1,
    drawing_to_display_area: B1,
    set_mask_bit: B1,
    check_mask: B1,
    interlace_field: B1,
    flip_screen_horizontally: B1,
    texture_page_y_base_2: B1,
    horizontal_resolution_2: B1,
    horizontal_resolution_1: B2,
    vertical_resolution: B1,
    video_mode: B1,
    display_area_color_depth: B1,
    vertical_interlace: B1,
    display_disable: B1,
    interrupt_request: B1,
    dma_request: B1,
    ready_to_receive_cmd: B1,
    ready_to_send_VRAM_to_CPU: B1,
    ready_to_receive_dma_block: B1,
    dma_direction: B2,
    drawing_even_odd_lines_in_interlace_mode: B1,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct VramCopyFields {
    vram_x: u16,
    vram_y: u16,

    width: u16,
    height: u16,

    current_row: u16,
    current_col: u16,
}

#[derive(PartialEq, Clone, Copy)]
enum ParametrizedCommand {
    CPU_VRAM_Copy,
    Polygon(u32),
}

#[derive(PartialEq)]
enum GP0_State {
    CommandStart,
    ReceivingParameters { 
        idx: usize,
        expected: usize,
        command: ParametrizedCommand,
    },
    ReceivingData(VramCopyFields),
}

pub struct GPU {
    vram: RAM,
    gp0_mode: GP0_State,
    gp0_parameters: VecDeque<u32>,

    drawing_area: (Vertex, Vertex),
    drawing_offset: Vertex,

    cycles: usize,
}

impl GPU {
    pub fn new() -> Self {
        let vram = RAM::new(VRAM_SIZE);

        Self {
            vram,
            gp0_mode: GP0_State::CommandStart,
            gp0_parameters: VecDeque::new(),

            drawing_area: (Vertex::default(), Vertex::default()),
            drawing_offset: Vertex::default(),

            cycles: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.cycles += 1;
        if self.cycles == 564_480 {
            self.cycles = 0;
            return true;
        }

        return false;
    }

    pub fn read_gp0(&mut self) -> u32 {
        0x00
    }

    pub fn read_gp1(&mut self) -> u32 {
        0x1C00_0000
    }

    pub fn write_gp0(&mut self, word: u32) {
        println!("GP0 {word:08X}");
        self.gp0_mode = match self.gp0_mode {
            GP0_State::CommandStart => {
                self.gp0_parameters.clear();
                match word >> 29 {
                    1 => self.set_polygon_state(word),
                    2 => {
                        println!("draw line {word:08X}");
                        GP0_State::CommandStart
                    }
                    3 => {
                        println!("draw rectangle {word:08X}");
                        GP0_State::CommandStart
                    }
                    4 => {
                        println!("VRAM-to-VRAM copy {word:08X}");
                        GP0_State::CommandStart
                    }
                    5 => GP0_State::ReceivingParameters {idx: 1, expected: 2, command: ParametrizedCommand::CPU_VRAM_Copy},
                    6 => {
                        println!("VRAM-to-CPU copy {word:08X}");
                        GP0_State::CommandStart
                    }
                    0 | 7 => match word >> 24 {
                        0xE3 => self.set_drawing_area_top_left(word),
                        0xE4 => self.set_drawing_area_bottom_right(word),
                        0xE5 => self.set_drawing_offset(word),
                        _ => {
                            println!("{word:08X}");
                            GP0_State::CommandStart
                        }
                    }
                    _ => unsafe { unreachable_unchecked() }
                }
            }

            GP0_State::ReceivingParameters {idx, expected, command} => {
                self.gp0_parameters.push_back(word);

                if idx == expected {
                    match command {
                        ParametrizedCommand::CPU_VRAM_Copy => self.initialize_cpu_vram_copy(),
                        ParametrizedCommand::Polygon(word) => {
                            let polygon_type = (word >> 24) as u8; match polygon_type {
                                0x28 => self.draw_monochrome_quad(word),
                                0x2C => self.draw_modulated_quad(word),
                                0x30 => self.draw_gouraud_tri(word),
                                0x38 => self.draw_gouraud_quad(word),
                                _ => {
                                    println!("Polygon command not implemented: {word:08X}");
                                    GP0_State::CommandStart
                                }
                            }
                        }
                    }
                } else {
                    GP0_State::ReceivingParameters { idx: idx + 1, expected, command }
                }
            }

            GP0_State::ReceivingData(_) => self.process_cpu_vram_copy(word)
        }
    }
    
    fn set_polygon_state(&mut self, word: u32) -> GP0_State {
        println!("draw polygon {word:08X}");

        let vertices = 3 + ((word >> 27) & 1);
        let color_words = (vertices - 1) * ((word >> 28) & 1);
        let texture_words = vertices * ((word >> 26) & 1);

        GP0_State::ReceivingParameters {
            idx: 1,
            expected: (vertices + color_words + texture_words) as usize,
            command: ParametrizedCommand::Polygon(word)
        }
    }

    fn draw_monochrome_quad(&mut self, word: u32) -> GP0_State {
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let v3: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_monochrome_tri(v0, v1, v2, word);
        self.write_monochrome_tri(v1, v2, v3, word);

        GP0_State::CommandStart
    }

    fn draw_modulated_quad(&mut self, word: u32) -> GP0_State {
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

    fn draw_gouraud_tri(&mut self, word: u32) -> GP0_State {
        let c0: Color = Color::compress_color_depth(word).into();
        let v0: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c1: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v1: Vertex = self.gp0_parameters.pop_front().unwrap().into();
        let c2: Color = Color::compress_color_depth(self.gp0_parameters.pop_front().unwrap()).into();
        let v2: Vertex = self.gp0_parameters.pop_front().unwrap().into();

        self.write_gouraud_tri(v0, v1, v2, c0, c1, c2);

        GP0_State::CommandStart
    }

    fn draw_gouraud_quad(&mut self, word: u32) -> GP0_State {
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

    fn set_drawing_area_top_left(&mut self, word: u32) -> GP0_State {
        let x = (word & 0x3FF) as i32;
        let y = ((word >> 10) & 0x1FF) as i32;

        self.drawing_area.0 = Vertex {x, y};

        GP0_State::CommandStart
    }

    fn set_drawing_area_bottom_right(&mut self, word: u32) -> GP0_State {
        let x = (word & 0x3FF) as i32;
        let y = ((word >> 10) & 0x1FF) as i32;

        self.drawing_area.1 = Vertex {x, y};

        GP0_State::CommandStart
    }

    fn set_drawing_offset(&mut self, word: u32) -> GP0_State {
        let mut x = word & 0x7FF;
        let mut y = (word >> 11) & 0x7FF;

        if x & (1 << 10) != 0 {x |= 0xFFFF_F800}
        if y & (1 << 10) != 0 {y |= 0xFFFF_F800}
        
        self.drawing_offset = Vertex {x: x as i32, y: y as i32};

        GP0_State::CommandStart
    }

    fn write_monochrome_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, color: u32) {
        let mut v0 = v0;
        let mut v1 = v1;

        Vertex::ensure_vertex_order(&mut v0, &mut v1, v2);

        let (min_x, max_x, min_y, max_y) = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1);

        for x in min_x..max_x {
            for y in min_y..max_y {
                let pixel = Vertex {x, y};
                if pixel.is_inside_triangle(v0, v1, v2) {
                    let coords = pixel.translate(self.drawing_offset).into();

                    self.draw_pixel(color, coords);
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

        let (min_x, max_x, min_y, max_y) = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1);

        for x in min_x..max_x {
            for y in min_y..max_y {
                let pixel = Vertex {x, y};
                if pixel.is_inside_triangle(v0, v1, v2) {
                    let coords = pixel.translate(self.drawing_offset).into();
                    let barycentric_coords = pixel.compute_barycentric_coordinates(v0, v1, v2);
                    let color = Color::interpolate_color(barycentric_coords, [c0, c1, c2]).apply_dithering(pixel).into();

                    self.draw_pixel(color, coords);
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
        let base_x = (page & 0xF) * 64;
        let base_y = ((page >> 4) & 1) * 256;
        let semi_transparency = (page >> 5) & 3;
        let tex_page_color_depth = (page >> 7) & 3;

        let clut_x = clut & 0x3F;
        let clut_y = (clut >> 6) & 0x1FF;

        let clut_addr = ((clut_y << 10) | (clut_x << 4)) << 1;

        let mut v0 = v0;
        let mut v1 = v1;

        let mut uv0 = uv0;
        let mut uv1 = uv1;
        let mut uv2 = uv2;

        if Vertex::ensure_vertex_order(&mut v0, &mut v1, v2) {
            std::mem::swap(&mut uv0, &mut uv1);
        }

        uv0 = (uv0.0 + base_x, uv0.1 + base_y);
        uv1 = (uv1.0 + base_x, uv1.1 + base_y);
        uv2 = (uv2.0 + base_x, uv2.1 + base_y);

        let (min_x, max_x, min_y, max_y) = Vertex::triangle_bounding_box(v0, v1, v2, self.drawing_area.0, self.drawing_area.1);

        for y in min_y..max_y {
            match tex_page_color_depth {
                0 => {
                    for x in (min_x..max_x).step_by(4) {
                        let pixel = Vertex {x, y};
                        if pixel.is_inside_triangle(v0, v1, v2) {
                            let pixel = pixel.translate(self.drawing_offset);

                            let tex_pixel = Vertex { x: min_x + ((x - min_x) >> 2), y: y};

                            let barycentric_coords = tex_pixel.compute_barycentric_coordinates(v0, v1, v2);
                            let uv = interpolate_uv_coords(barycentric_coords, [uv0, uv1, uv2]);
                            let tex_color = self.vram.read16(2 * (1024 * uv.1 + uv.0));

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
                                        let pixel = Vertex { x: pixel.x + i as i32, y: pixel.y };
                                        let coords: u32 = pixel.into();
                                        self.draw_pixel_compressed(*color, coords);
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

    fn draw_pixel(&mut self, color: u32, coords: u32) {
        let color_halfword = Color::compress_color_depth(color);
        
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr = 2 * (1024 * y + x);
        
        self.vram.write16(vram_addr, color_halfword);
    }

    fn draw_pixel_compressed(&mut self, color: u16, coords: u32) {
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr = 2 * (1024 * y + x);
        
        self.vram.write16(vram_addr, color);
    }

    pub fn write_gp1(&mut self, word: u32) {
        println!("GP1 {word:08X}");
    }

    fn initialize_cpu_vram_copy(&mut self) -> GP0_State {
        let coords = self.gp0_parameters.pop_front().unwrap();
        let size = self.gp0_parameters.pop_front().unwrap();

        let vram_x = (coords & 0x3FF) as u16;
        let vram_y = ((coords >> 16) & 0x1FF) as u16;

        let mut width = (size & 0x3FF) as u16;
        if width == 0 {width = 1024}

        let mut height = ((size >> 16) & 0x1FF) as u16;
        if height == 0 {height = 512}

        GP0_State::ReceivingData(
            VramCopyFields { vram_x: 
                vram_x, vram_y,
                width, height, 
                
                current_row: 0, current_col: 0
            }
        )
    }

    fn process_cpu_vram_copy(&mut self, word: u32) -> GP0_State {
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

    pub fn render_vram(&self) -> Box<[Color; 512 * 1024]> {
        let mut output = Box::new([Color::default(); 512 * 1024]);
        for y in 0..512 {
            for x in 0..1024 {
                let vram_addr = (2 * (1024 * y + x)) as u32;
                let pixel = self.vram.read16(vram_addr);

                output[1024 * y + x] = Color::from(pixel);
            }
        }

        output
    }
}