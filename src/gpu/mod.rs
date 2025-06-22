use std::{collections::VecDeque, hint::unreachable_unchecked};

use modular_bitfield::{bitfield, prelude::*};

use crate::{gpu::primitives::color::Color, ram::RAM};

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
    Flat_Polygon(u32),
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

    cycles: usize,
}

impl GPU {
    pub fn new() -> Self {
        let vram = RAM::new(VRAM_SIZE);

        Self {
            vram,
            gp0_mode: GP0_State::CommandStart,
            gp0_parameters: VecDeque::new(),

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
        if self.gp0_mode == GP0_State::CommandStart {println!("GP0 {word:08X}")}
        self.gp0_mode = match self.gp0_mode {
            GP0_State::CommandStart => match word >> 29 {
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
                    _ => {
                        println!("{word:08X}");
                        GP0_State::CommandStart
                    }
                }
                _ => unsafe { unreachable_unchecked() }
            }

            GP0_State::ReceivingParameters {idx, expected, command} => {
                self.gp0_parameters.push_back(word);

                if idx == expected {
                    match command {
                        ParametrizedCommand::CPU_VRAM_Copy => self.initialize_cpu_vram_copy(),
                        ParametrizedCommand::Flat_Polygon(word) => {
                            let polygon_type = (word >> 24) as u8; match polygon_type {
                                0x28 => self.draw_flat_quad(word),
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
            command: ParametrizedCommand::Flat_Polygon(word)
        }
    }

    fn draw_flat_quad(&mut self, word: u32) -> GP0_State {


        GP0_State::CommandStart
    }

    pub fn write_gp1(&mut self, word: u32) {
        println!("GP1 {word:08X}");
    }

    fn initialize_cpu_vram_copy(&mut self) -> GP0_State {
        let (size, coords) = (self.gp0_parameters.pop_front().unwrap(), self.gp0_parameters.pop_front().unwrap());

        let vram_x = (coords & 0x3FF) as u16;
        let vram_y = ((coords >> 16) & 0x1FF) as u16;

        let mut width = (size & 0x3FF) as u16;
        if width == 0 {width = 1024}

        let mut height = ((size >> 16) & 0x3FF) as u16;
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

            println!("[{vram_addr:08X}] <- {halfword:04X}");
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

    fn draw_pixel(&mut self, color: u32, coords: u32) {
        let color_halfword: u16 = Color::from(color).into();
        
        let x = coords & 0x3FF;
        let y = (coords >> 16) & 0x1FF;

        let vram_addr = 2 * (1024 * y + x);
        
        self.vram.write16(vram_addr, color_halfword);
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