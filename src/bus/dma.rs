use std::{cell::RefCell, ops::{Index, IndexMut}, rc::Rc};

use crate::{bus::interface::Interface, Registers};

const CHANNELS: [u8; 7] = [0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60];

pub struct DMA {
    channels: Channels,
    remaining_size: [u16; 7],
    current_addr: [Option<u32>; 7],
    header: u32,

    interface: Rc<RefCell<Interface>>,

    clock: usize,
    running: Rc<RefCell<bool>>,
}

impl DMA {
    pub fn new(interface: Rc<RefCell<Interface>>, running: Rc<RefCell<bool>>) -> Self {
        let mut channels = Channels {channels: [Registers{R: [0; 4]}; 8]};
        channels[0x70] = 0x0765_4321;
        Self {
            channels,
            remaining_size: [0; 7],
            current_addr: [None; 7],
            header: 0x00FF_FFFF,

            interface,
            
            clock: 0,
            running,
        }
    }

    pub fn tick(&mut self) {
        self.clock += 1;
        if self.clock == 120 {
            self.clock = 0;
        }

        if let Some(channel) = self.active_channel() {
            self.running.replace(true);
            match channel >> 4 {
                0 => panic!("MDEC-IN not implemented."),
                1 => panic!("MDEC-OUT not implemented."),
                2 => match self.channels.sync_type(channel) {
                    2 => self.linked_list_transfer(channel),
                    _ => self.block_transfer(channel),
                }
                3 => panic!("CDROM not implemented."),
                4 => panic!("SPU not implemented."),
                5 => panic!("PIO not implemented."),
                6 => self.block_transfer(channel),
                _ => panic!("Unreachable channel: {channel}"),
            }
        }
    }

    pub fn active_channel(&self) -> Option<u32> {
        CHANNELS.iter()
            .filter_map(|channel| {
                let channel = *channel as u32;
                let trigger = self.channels.sync_type(channel) != 0 || self.channels.force_transfer(channel);

                if self.channels.enabled(channel) && trigger {
                    Some((channel, self.channels.priority(channel)))
                } else {None}
            })
            .max_by_key(|(_channel, priority)| {*priority})
            .map(|(channel, _priority)| {channel})
    }

    pub fn read_register(&self, offset: u32) -> u32 {
        // println!("DMA[{:02X}] = {:08X}", offset, self.channels[offset]);
        self.channels[offset]
    }

    pub fn write_register(&mut self, offset: u32, value: u32) {
        // println!("DMA[{:02X}] <- {:08X}", offset, value);
        match offset {
            0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0x50 | 0x60 => {
                self.channels[offset] = value & 0x00FF_FFFF
            }
            0x74 => {
                let interrupt_register = &mut self.channels[0x74];
                *interrupt_register = value;
                let mask = value & 0x7F00_0000;
                *interrupt_register &= !mask;
            }
            _ => self.channels[offset] = value,
        }
    }

    pub fn bus_error(&self) -> bool {
        self.channels[0x74] & (1 << 15) != 0
    }

    pub fn master_interrupt_enabled(&self) -> bool {
        self.channels[0x74] & (1 << 23) != 0
    }

    pub fn master_interrupt(&self) -> bool {
        self.channels[0x74] & (1 << 31) != 0
    }

    fn block_transfer(&mut self, index: u32) {
        let channel = (index >> 4) as usize;
        let increment = self.channels.increment_size(index);

        self.current_addr[channel] = Some(self.current_addr[channel].unwrap_or_else(|| self.channels.base_address(index)));
        let addr = unsafe { self.current_addr[channel].unwrap_unchecked() };

        if self.remaining_size[channel] == 0 {
            self.remaining_size[channel] = match self.channels.sync_type(index) {
                0 => self.channels.word_num(index),
                1 => self.channels.block_amount(index) * self.channels.block_size(index),
                _ => panic!("Unknown block sync mode"),
            }
            // println!("Remaining size: {}", self.remaining_size[channel]);
        }
        let mut remaining_size = self.remaining_size[channel];

        if remaining_size > 0 {
            if self.channels.transfer_direction(index) {
                match channel {
                    2 => {
                        let value = self.interface.borrow_mut().read32(addr & 0x001F_FFFC);
                        self.interface.borrow_mut().write32(0x1F80_1810, value);
                    }
                    _ => panic!("Unhandled DMA channel {channel} RAM -> Device"),
                };
            } else {
                let value = match channel {
                    6 => match remaining_size {
                        1 => 0x00FF_FFFF,
                        _ => addr.wrapping_sub(4) & 0x001F_FFFF,
                    }
                    _ => panic!("Unhandled DMA channel {channel} Device -> RAM"),
                };

                // println!("DMA: [{:08X}] <- [{:08X}]", addr, value);
                self.interface.borrow_mut().write32(addr & 0x001F_FFFC, value);
            }

            remaining_size -= 1;
            self.remaining_size[channel] = remaining_size;
            if remaining_size == 0 {
                self.current_addr[channel] = None;
                self.channels.done(index);
                if self.active_channel().is_none() {self.running.replace(false);}
            } else {
                self.current_addr[channel] = Some(addr.wrapping_add(increment));
            }
        }
    }

    fn linked_list_transfer(&mut self, index: u32) {
        if !self.channels.transfer_direction(index) {panic!("Linked list mode cannot transfer to RAM")}

        let channel = (index >> 4) as usize;
        if channel != 2 {panic!("Attempting linked-list DMA on non-GPU channel: {}", channel)}

        let mut remaining_size = self.remaining_size[channel];
        if self.remaining_size[channel] == 0 {
            if self.current_addr[channel] == None {
                let first_header_addr = self.channels.base_address(index);
                self.current_addr[channel] = Some(first_header_addr);
                // println!("base: {:08X}", first_header_addr);
                self.header = self.interface.borrow_mut().read32(first_header_addr);
            }

            remaining_size = (self.header >> 24) as u16;
            self.remaining_size[channel] = remaining_size;
        }
        let mut addr = (self.current_addr[channel].unwrap() + 4) & 0x001F_FFFC;

        if remaining_size > 0 {
            let command = self.interface.borrow_mut().read32(addr);
            self.interface.borrow_mut().write32(0x1F80_1810, command);
        }

        remaining_size = remaining_size.saturating_sub(1);
        self.remaining_size[channel] = remaining_size;
        if remaining_size == 0 {
            if self.header & 0x0080_0000 != 0 {
                self.current_addr[channel] = None;
                self.channels.done(index);
                self.running.replace(false);
                return;
            }
            addr = self.header & 0x00FF_FFFF;
            self.header = self.interface.borrow_mut().read32(addr);
            // println!("Header: {:08X} addr: {:08X}", self.header, addr);
        }

        self.current_addr[channel] = Some(addr);
    }
}

struct Channels {
    channels: [Registers<4>; 8],
}

impl Channels {
    pub fn done(&mut self, index: u32) {
        let channel = index >> 4;
        self.channels[channel as usize][2] &= !((1 << 28) | (1 << 24));

        let bit = ((channel + 1) * 4) - 1;
        self.channels[7][0] &= !((1 << bit));
        if self.sync_type(index) == 1 {
            let blocks = self.block_amount(index).saturating_sub(1) as u32;
            let block_control = &mut self.channels[channel as usize][1];
            *block_control = (*block_control & 0x0000_FFFF) | (blocks << 16)
        }
    }

    pub fn enabled(&self, index: u32) -> bool {
        let channel = index >> 4;
        let bit = ((channel + 1) * 4) - 1;
        self.channels[7][0] & (1 << bit) != 0 && self.start_transfer(index)
    }

    pub fn priority(&self, index: u32) -> u8 {
        let channel = index >> 4;
        (self.channels[7][0] & (0x7 << channel)) as u8
    }

    pub fn interrupt_mode(&self, index: u32) -> bool {
        let channel = index >> 4;
        self.channels[7][1] & (1 << channel) != 0
    }

    pub fn interrupt_mask(&self, index: u32) -> bool {
        let channel = index >> 4;
        self.channels[7][1] & (1 << (channel + 16)) != 0
    }

    pub fn interrupt_flag(&self, index: u32) -> bool {
        let channel = index >> 4;
        self.channels[7][1] & (1 << (channel + 24)) != 0
    }

    pub fn word_num(&self, index: u32) -> u16 {
        self.block_control(index) as u16
    }

    pub fn block_size(&self, index: u32) -> u16 {
        self.block_control(index) as u16
    }

    pub fn block_amount(&self, index: u32) -> u16 {
        (self.block_control(index) >> 16) as u16
    }

    pub fn transfer_direction(&self, index: u32) -> bool {
        self.channel_control(index) & (1 << 0) != 0
    }

    pub fn increment_size(&self, index: u32) -> u32 {
        if self.channel_control(index) & (1 << 1) != 0 {4u32.wrapping_neg()} else {4}
    }

    pub fn chopping(&self, index: u32) -> bool {
        self.channel_control(index) & (1 << 8) != 0
    }

    pub fn sync_type(&self, index: u32) -> u8 {
        ((self.channel_control(index) >> 9) & 3) as u8
    }

    pub fn chopping_dma_window(&self, index: u32) -> u8 {
        ((self.channel_control(index) >> 16) & 7) as u8
    }

    pub fn chopping_cpu_window(&self, index: u32) -> u8 {
        ((self.channel_control(index) >> 20) & 7) as u8
    }

    pub fn start_transfer(&self, index: u32) -> bool {
        self.channel_control(index) & (1 << 24) != 0
    }

    pub fn force_transfer(&self, index: u32) -> bool {
        self.channel_control(index) & (1 << 28) != 0
    }

    pub fn base_address(&self, index: u32) -> u32 {
        let channel = index >> 4;
        self.channels[channel as usize][0]
    }

    fn block_control(&self, index: u32) -> u32 {
        let channel = index >> 4;
        self.channels[channel as usize][1]
    }

    fn channel_control(&self, index: u32) -> u32 {
        let channel = index >> 4;
        self.channels[channel as usize][2]
    }
}

impl Index<u32> for Channels {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        let channel = index >> 4;
        let register = (index & 0xF) >> 2;
        // println!("Reading DMA channel {channel}, register {register}, index {:02X}", index);
        &self.channels[channel as usize][register]
    }
}

impl IndexMut<u32> for Channels {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let channel = index >> 4;
        let register = (index & 0xF) >> 2;
        // println!("Writing DMA channel {channel}, register {register}, index {:02X}", index);
        &mut self.channels[channel as usize][register]
    }
}