use std::{cell::RefCell, collections::VecDeque, hint::unreachable_unchecked, rc::Rc};

use glam::u8vec3;
use modular_bitfield::{bitfield, prelude::*};

use crate::{bus::{interrupt::{Interrupt, IRQ}, timer::Timer}, gpu::primitives::{color::Color, vertex::Vertex}, ram::RAM};

const VRAM_SIZE: usize = 1024 * 1024;

pub mod primitives;
mod commands;

#[bitfield]
struct GPUSTAT {
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
pub struct BlitFields {
    vram_x: u16,
    vram_y: u16,

    width: u16,
    height: u16,

    current_row: u16,
    current_col: u16,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ParametrizedCommand {
    CPU_VRAM_Copy,
    VRAM_CPU_Copy,
    Polygon(u32),
    Line(u32),
    Rectangle(u32),
}

#[derive(PartialEq, Clone, Copy)]
pub enum GP0_State {
    CommandStart,
    ReceivingParameters { 
        idx: usize,
        expected: usize,
        command: ParametrizedCommand,
    },
    ReceivingPolyLineParameters {
        color_word: bool,
        gouraud: bool,
        command: ParametrizedCommand,
    },
    ReceivingData(BlitFields),
    SendingData(BlitFields),
}

pub struct GPU {
    vram: RAM,
    gp0_mode: GP0_State,
    gpu_read_transfer: Option<GP0_State>,
    gp0_parameters: VecDeque<u32>,

    gpu_read: u32,
    gpu_status: GPUSTAT,

    drawing_area: (Vertex, Vertex),
    drawing_offset: Vertex,

    display_range: (Vertex, Vertex),
    display_area_start: Vertex,

    tex_window: u32,

    cycle: usize,
    even_odd_frame: bool,

    interrupt: Rc<RefCell<Interrupt>>,
    timer: Rc<RefCell<Timer>>,
}

impl GPU {
    pub fn new(interface: Rc<RefCell<Interrupt>>, timer: Rc<RefCell<Timer>>) -> Self {
        let vram = RAM::new(VRAM_SIZE);

        Self {
            vram,
            gp0_mode: GP0_State::CommandStart,
            gpu_read_transfer: None,
            gp0_parameters: VecDeque::new(),

            gpu_read: 0,
            gpu_status: GPUSTAT::from_bytes(0x1C00_0000u32.to_le_bytes()),

            drawing_area: (Vertex::default(), Vertex::default()),
            drawing_offset: Vertex::default(),

            display_range: (Vertex::default(), Vertex::default()),
            display_area_start: Vertex::default(),

            tex_window: 0,

            cycle: 566_203 - 516_687,
            even_odd_frame: false,

            interrupt: interface,
            timer,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.cycle += 1;
        if self.cycle == 566_203 {
            self.cycle = 0;

            self.even_odd_frame = !self.even_odd_frame;
            self.interrupt.borrow_mut().request(IRQ::VBLANK);

            return true;
        }

        return false;
    }

    pub fn read_gp0(&mut self) -> u32 {
        if let Some(GP0_State::SendingData(_)) = self.gpu_read_transfer {
            self.process_vram_cpu_copy();
        }
        
        self.gpu_read
    }

    pub fn read_gp1(&mut self) -> u32 {
        self.gpu_status.set_drawing_even_odd_lines_in_interlace_mode((!self.even_odd_frame) as u8 & (1 >> (!self.gpu_status.vertical_interlace() & 1)));
        u32::from_le_bytes(self.gpu_status.bytes)
    }

    pub fn write_gp0(&mut self, word: u32) {
        // println!("GP0 {word:08X}");
        self.gp0_mode = match self.gp0_mode {
            GP0_State::CommandStart => {
                self.gp0_parameters.clear();
                match word >> 29 {
                    1 => self.set_polygon_state(word),
                    2 => self.set_line_state(word),
                    3 => self.set_rectangle_state(word),
                    4 => {
                        println!("VRAM-to-VRAM copy {word:08X}");
                        GP0_State::CommandStart
                    }
                    5 => GP0_State::ReceivingParameters {idx: 1, expected: 2, command: ParametrizedCommand::CPU_VRAM_Copy},
                    6 => GP0_State::ReceivingParameters {idx: 1, expected: 2, command: ParametrizedCommand::VRAM_CPU_Copy},
                    0 | 7 => match word >> 24 {
                        // NOP
                        0x00 | 0x04..=0x1E | 0xE0 | 0xE7..=0xEF =>
                            GP0_State::CommandStart,
                        // Weird NOP that takes up space in the FIFO, no FIFO in this emulator though
                        0x03 => GP0_State::CommandStart,

                        0x1F => self.irq(),
                        0xE1 => self.set_texpage(word),
                        0xE2 => self.set_tex_window(word),
                        0xE3 => self.set_drawing_area_top_left(word),
                        0xE4 => self.set_drawing_area_bottom_right(word),
                        0xE5 => self.set_drawing_offset(word),
                        0xE6 => self.set_mask_bit_setting(word),
                        _ => {
                            // println!("{word:08X}");
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
                        ParametrizedCommand::VRAM_CPU_Copy => self.initialize_vram_cpu_copy(),
                        ParametrizedCommand::Polygon(word) => {
                            match (word >> 24) as u8 {
                                0x20 => self.draw_monochrome_tri(word),
                                0x22 => self.draw_transparent_monochrome_tri(word),
                                0x2A => self.draw_transparent_monochrome_quad(word),
                                0x28 => self.draw_monochrome_quad(word),
                                0x2C => self.draw_modulated_quad(word),
                                0x30 => self.draw_gouraud_tri(word),
                                0x32 => self.draw_transparent_gouraud_tri(word),
                                0x38 => self.draw_gouraud_quad(word),
                                0x3A => self.draw_transparent_gouraud_quad(word),
                                _ => {
                                    println!("Polygon command not implemented: {word:08X}");
                                    GP0_State::CommandStart
                                }
                            }
                        }
                        ParametrizedCommand::Line(word) => {
                            match ((word >> 24) & !((1 << 26) | (1 << 24))) as u8 {
                                0x40 => self.draw_monochrome_line(word),
                                0x42 => self.draw_transparent_monochrome_line(word),
                                0x50 => self.draw_gouraud_line(word),
                                0x52 => self.draw_transparent_gouraud_line(word),
                                _ => {
                                    println!("Line command not implemented: {word:08X}");
                                    GP0_State::CommandStart
                                }
                            }
                        }
                        ParametrizedCommand::Rectangle(word) => {
                            match (word >> 24) as u8 {
                                0x68 => self.draw_single_pixel_monochrome_rect(word),
                                _ => {
                                    println!("Rectangle command not implemented: {word:08X}");
                                    GP0_State::CommandStart
                                }
                            }
                        }
                    }
                } else {
                    GP0_State::ReceivingParameters { idx: idx + 1, expected, command }
                }
            }

            GP0_State::ReceivingData(_) => self.process_cpu_vram_copy(word),
            GP0_State::ReceivingPolyLineParameters { color_word, gouraud, command } => {
                if (color_word || !gouraud) && (word & 0xF000F000 == 0x50005000) {
                    match command {
                        ParametrizedCommand::Line(word) => {
                            match ((word >> 24) & !((1 << 26) | (1 << 24))) as u8 {
                                0x48 => self.draw_monchrome_polyline(word),
                                0x4A => self.draw_transparent_monchrome_polyline(word),
                                0x58 => self.draw_gouraud_polyline(word),
                                0x5A => self.draw_transparent_gouraud_polyline(word),
                                _ => {
                                    println!("Polyline command not implemented: {word:08X}");
                                    GP0_State::CommandStart
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                } else {
                    self.gp0_parameters.push_back(word);

                    GP0_State::ReceivingPolyLineParameters { color_word: !color_word && gouraud, gouraud, command }
                }
            }
            _ => panic!("Unsupported mode for writes"),
        }
    }

    pub fn write_gp1(&mut self, word: u32) {
        // println!("GP1 {word:08X}");
        self.gp1_command(word);
    }

    fn draw_pixel(&mut self, color: u32, coords: u32) {
        let color_halfword = Color::compress_color_depth(color);
        
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr =((y << 10) + x) << 1;
        
        self.vram.write16(vram_addr, color_halfword);
    }

    fn draw_transparent_pixel(&mut self, color: u32, coords: u32) {
        let color_halfword = Color::compress_color_depth(color);
        
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr = ((y << 10) + x) << 1;

        let back: Color = self.vram.read16(vram_addr).into();
        let front: Color = color_halfword.into();

        let blended = Color::compress_color_depth(front.blend(back, self.gpu_status.semi_transparency()).into());
        
        self.vram.write16(vram_addr, blended);
    }

    fn draw_compressed_pixel(&mut self, color: u16, coords: u32) {
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr = ((y << 10) + x) << 1;
        
        self.vram.write16(vram_addr, color);
    }

    pub fn render_vram(&self) -> Box<[Color; 512 * 1024]> {
        let mut output = Box::new([Color::default(); 512 * 1024]);
        if self.gpu_status.display_area_color_depth() == 0 {
            for y in 0..512 {
                for x in 0..1024 {
                    let vram_addr = (((y << 10) + x) << 1) as u32;
                    let pixel = self.vram.read16(vram_addr);

                    output[(y << 10) + x] = Color::from(pixel);
                }
            }
        } else {
            for y in 0..512 {
                let line_start = (y << 11) as u32;
                for x in 0..682 {
                    let vram_addr = line_start + (x * 3) as u32;
                    let r = self.vram.read8(vram_addr);
                    let g = self.vram.read8(vram_addr + 1);
                    let b = self.vram.read8(vram_addr + 2);

                    output[(y << 10) + x] = Color {rgb: u8vec3(r, g, b)};
                }
            }
        }

        output
    }
}