use std::collections::VecDeque;

use modular_bitfield::{bitfield, prelude::*};

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

pub struct GPU {
    gpu_status: GPUSTAT,
    gpu_read: u32,

    display_area_start: u32,
    x1: u32, x2: u32,
    y1: u32, y2: u32,
    x_offset: u32, y_offset: u32,

    texture_window_mask_x: u32,
    texture_window_mask_y: u32,
    texture_window_offset_x: u32,
    texture_window_offset_y: u32,
    
    textured_rectangle_x_flip: bool,
    textured_rectangle_y_flip: bool,

    command_buffer: VecDeque<u32>,
    parameters: Option<usize>,
}

impl GPU {
    pub fn new() -> Self {
        Self {
            gpu_status: GPUSTAT::from_bytes(0x1C00_0000u32.to_le_bytes()),
            gpu_read: 0,

            display_area_start: 0,
            x1: 0, x2: 0,
            y1: 0, y2: 0,
            x_offset: 0, y_offset: 0,
    
            texture_window_mask_x: 0,
            texture_window_mask_y: 0,
            texture_window_offset_x: 0,
            texture_window_offset_y: 0,

            textured_rectangle_x_flip: false,
            textured_rectangle_y_flip: false,

            command_buffer: VecDeque::with_capacity(16),
            parameters: None,
        }
    }

    pub fn tick(&mut self) {
        if let Some(parameter_count) = self.parameters {
            if self.command_buffer.len() > parameter_count {
                let command = self.command_buffer.pop_front().unwrap();
                self.execute_gp0(command);
            }
        }
    }

    pub fn read_gp0(&self) -> u32 {
        println!("GPUREAD = {:08X}", self.gpu_read);
        self.gpu_read
    }

    pub fn read_gp1(&self) -> u32 {
        println!("GPUSTAT = {:08X}", u32::from_le_bytes(self.gpu_status.bytes));
        u32::from_le_bytes(self.gpu_status.bytes) & !(1 << 19)
    }

    pub fn write_gp0(&mut self, command: u32) {
        println!("GP0: {:08X}", command);
        self.set_params(command);
    }

    pub fn write_gp1(&mut self, command: u32) {
        println!("GP1: {:08X}", command);
        self.execute_gp1(command);
    }

    fn set_params(&mut self, command: u32) {
        let command_number = command >> 24;
        if self.parameters == None {
            let parameter_count = match command_number {
                0x00 => return,
                0x28 => 4,
                0xE1 => 0,
                0xE2 => 0,
                0xE3 => 0,
                0xE4 => 0,
                0xE5 => 0,
                0xE6 => 0,
                _ => panic!("Unsupported GPU command {:08X}", command)
            };

            self.parameters = Some(parameter_count);
        }

        self.command_buffer.push_back(command);
    }

    fn execute_gp0(&mut self, command: u32) {
        let command_number = command >> 24;
        match command_number {
            0x00 => {}
            0x28 => self.draw_monochrome_quad(command),
            0xE1 => self.draw_mode_setting(command),
            0xE2 => self.texture_window_setting(command),
            0xE3 => self.drawing_area_top_left(command),
            0xE4 => self.drawing_area_bottom_right(command),
            0xE5 => self.drawing_offset(command),
            0xE6 => self.mask_bit_setting(command),
            _ => panic!("Unsupported GPU command {:08X}", command)
        }
        self.parameters = None;
    }

    fn draw_monochrome_quad(&mut self, command: u32) {
        let points: Vec<(u32, u32)> = self.command_buffer
            .drain(..)
            .map(|w| {
                ((w >> 16), (w & 0x0000_FFFF))
            })
            .collect();

        let color = command & 0x00FF_FFFF;

        println!(
            "Quad: ({}, {}) ({}, {}) ({}, {}) ({}, {}) RGB24: {:06X}",
            points.get(0).unwrap().0, points.get(0).unwrap().1,
            points.get(1).unwrap().0, points.get(1).unwrap().1,
            points.get(2).unwrap().0, points.get(2).unwrap().1,
            points.get(3).unwrap().0, points.get(3).unwrap().1,
            color
        )
    }

    fn draw_mode_setting(&mut self, command: u32) {
        self.gpu_status.set_texture_page_x_base((command & 0xF) as u8);
        self.gpu_status.set_texture_page_y_base_1(((command >> 4) & 1) as u8);
        self.gpu_status.set_semi_transparency(((command >> 5) & 3) as u8);
        self.gpu_status.set_texture_page_colors(((command >> 7) & 3) as u8);
        self.gpu_status.set_dither_24bit_to_15bit(((command >> 9) & 1) as u8);
        self.gpu_status.set_drawing_to_display_area(((command >> 10) & 1) as u8);
        self.gpu_status.set_texture_page_y_base_2(((command >> 11) & 1) as u8);
        self.textured_rectangle_x_flip = ((command >> 12) & 1) != 0;
        self.textured_rectangle_y_flip = ((command >> 13) & 1) != 0;
    }

    fn texture_window_setting(&mut self, command: u32) {
        self.texture_window_mask_x = command & 0x1F;
        self.texture_window_mask_y = (command >> 5) & 0x1F;
        self.texture_window_offset_x = (command >> 10) & 0x1F;
        self.texture_window_offset_y = (command >> 15) & 0x1F;
    }

    fn drawing_area_top_left(&mut self, command: u32) {
        self.x1 = command & 0x0000_03FF;
        self.y1 = (command >> 10) & 0x0000_01FF;
    }

    fn drawing_area_bottom_right(&mut self, command: u32) {
        self.x2 = command & 0x0000_03FF;
        self.y2 = (command >> 10) & 0x0000_01FF;
    }

    fn drawing_offset(&mut self, command: u32) {
        self.x_offset = command & 0x0000_07FF;
        self.y_offset = (command >> 11) & 0x0000_07FF;
    }

    fn mask_bit_setting(&mut self, command: u32) {
        self.gpu_status.set_set_mask_bit((command & 1) as u8);
        self.gpu_status.set_check_mask(((command >> 1) & 1) as u8);
    }

    fn execute_gp1(&mut self, command: u32) {
        let command = command & 0x3FFF_FFFF;
        let command_number = command >> 24;
        match command_number {
            0x00 => self.reset_gpu(),
            0x01 => self.reset_command_buffer(),
            0x02 => self.acknowledge_gpu_interrupt(),
            0x03 => self.display_enable(command),
            0x04 => self.set_dma_direction(command),
            0x05 => self.set_display_area_start(command),
            0x06 => self.set_horizontal_display_range(command),
            0x07 => self.set_vertical_display_range(command),
            0x08 => self.set_display_mode(command),
            _ => println!("Unsupported GP1 instruction {:02X}", command_number),
        }
    }

    fn reset_gpu(&mut self) {
        self.reset_command_buffer();
        self.acknowledge_gpu_interrupt();
        self.display_enable(1);
        self.set_dma_direction(0);
        self.set_display_area_start(0);
        self.set_horizontal_display_range(0);
        self.set_vertical_display_range(0);
        self.set_display_mode(0);
    }

    fn reset_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    fn acknowledge_gpu_interrupt(&mut self) {
        println!("IRQ1: GPU interrupt not implemented.")
    }

    fn display_enable(&mut self, command: u32) {
        self.gpu_status.set_display_disable((command & 1) as u8);
    }

    fn set_dma_direction(&mut self, command: u32) {
        self.gpu_status.set_dma_direction((command & 3) as u8);
    }

    fn set_display_area_start(&mut self, command: u32) {
        self.display_area_start = command & 0x0007_FFFF;
    }

    fn set_horizontal_display_range(&mut self, command: u32) {
        self.x1 = command & 0x0000_0FFF;
        self.x2 = (command >> 12) & 0x0000_0FFF;
    }

    fn set_vertical_display_range(&mut self, command: u32) {
        self.y1 = command & 0x0000_03FF;
        self.y2 = (command >> 10) & 0x0000_03FF;
    }

    fn set_display_mode(&mut self, command: u32) {
        self.gpu_status.set_horizontal_resolution_1((command & 3) as u8);
        self.gpu_status.set_vertical_resolution(((command >> 2) & 1) as u8);
        self.gpu_status.set_video_mode(((command >> 3) & 1) as u8);
        self.gpu_status.set_display_area_color_depth(((command >> 4) & 1) as u8);
        self.gpu_status.set_vertical_interlace(((command >> 5) & 1) as u8);
        self.gpu_status.set_horizontal_resolution_2(((command >> 6) & 1) as u8);
        self.gpu_status.set_flip_screen_horizontally(((command >> 7) & 1) as u8);
    }
}
