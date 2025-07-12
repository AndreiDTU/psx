use crate::{cpu::gte::command::GTE_Command, Registers};

mod command;
mod register;

pub struct GTE {
    UNR_TABLE: [u8; 0x101],

    R: Registers<64>,

    cycles: usize,
}

impl GTE {
    pub fn new() -> Self {
        let UNR_TABLE: [u8; 0x101] = std::array::from_fn(|i| {
            let i = i as i32;
            std::cmp::max(0, (0x40000 / (i + 0x100) + 1) / 2 - 0x101) as u8
        });

        Self {
            UNR_TABLE,

            R: Registers { R: [0; 64] },
        
            cycles: 0,
        }
    }

    pub fn issue_command(&mut self, command: u32) {
        // println!("GTE command: {command:08X}");
        self.R[63] = 0;
        self.cycles = match command.num() {
            0x01 => self.rtps(command),
            0x06 => self.nclip(command),
            0x0C => self.op(command),
            0x10 => self.dpcs(command),
            0x11 => self.intpl(command),
            0x12 => self.mvmva(command),
            0x13 => self.ncds(command),
            0x14 => self.cdp(command),
            0x16 => self.ncdt(command),
            0x1B => self.nccs(command),
            0x1C => self.cc(command),
            0x1E => self.ncs(command),
            0x20 => self.nct(command),
            0x28 => self.sqr(command),
            0x29 => self.dcpl(command),
            0x2A => self.dpct(command),
            0x2D => self.avsz3(command),
            0x2E => self.avsz4(command),
            0x30 => self.rtpt(command),
            0x3D => self.gpf(command),
            0x3E => self.gpl(command),
            0x3F => self.ncct(command),
            _ => panic!("GTE command not implemented {command:08X}"),
        }
    }

    pub fn read_data_register(&self, register: u32) -> u32 {
        match register {
            1 | 3 | 5 | 8..=11 => self.R[register] as i16 as u32,
            7 | 16..=19 => self.R[register] as u16 as u32,
            15 => self.read_data_register(14),
            28 | 29 => self.R[28] & 0x7FFF,
            31 => self.lzcr(),
            _ => self.R[register],
        }
    }

    pub fn read_ctrl_register(&mut self, register: u32) -> u32 {
        let ctrl_reg = register + 32;
        match ctrl_reg {
            36 | 44 | 52 | 58 | 59 | 61 | 62 => self.R[ctrl_reg] as i16 as u32,
            63 => {
                let flag = &mut self.R[63];
                *flag &= 0x7FFF_FFFF;
                *flag |= 0x8000_0000 * ((*flag & 0x7F87_E000 != 0) as u32);                

                *flag
            }
            _ => self.R[ctrl_reg],
        }
    }

    pub fn write_data_register(&mut self, register: u32, value: u32) {
        match register {
            7 | 16..=19 => self.R[register] = value as u16 as u32,
            9 => {
                self.R[register] = value as u16 as u32;
                self.update_irgb_red(value);
            }
            10 => {
                self.R[register] = value as u16 as u32;
                self.update_irgb_green(value);
            }
            11 => {
                self.R[register] = value as u16 as u32;
                self.update_irgb_blue(value);
            }
            15 => {
                self.R[12] = self.R[13];
                self.R[13] = self.R[14];
                self.R[14] = value;
            }
            28 => {
                self.R[register] = value & 0x7FFF;
                self.R[9] = (value & 0x1F) << 7;
                self.R[10] = ((value >> 5) & 0x1F) << 7;
                self.R[11] = ((value >> 10) & 0x1F) << 7;
            }
            29 | 31 => {},
            _ => self.R[register] = value,
        }
        self.R[register] = value;
    }

    pub fn write_ctrl_register(&mut self, register: u32, value: u32) {
        let ctrl_reg = register + 32;
        // println!("Write {value:08X} to cop2r{register}; GTE R[{ctrl_reg}] = {:08X}", self.R[ctrl_reg]);
        match ctrl_reg {
            58 => self.R[ctrl_reg] = value as u16 as u32,
            63 => self.R[ctrl_reg] = (self.R[ctrl_reg] & 0x8000_0FFF) | (value & !0x8000_0FFF),
            _ => self.R[ctrl_reg] = value,
        }
    }

    fn update_irgb_red(&mut self, value: u32) {
        let mut red = value as u16;
        if red & 0x8000 != 0 {red = 0} else if red >= 0x1000 {red = 0x1F} else {red >>= 7}
        self.R[28] = (self.R[28] & !0x1F) | red as u16 as u32;
        self.R[28] &= 0x7FFF;
    }

    fn update_irgb_green(&mut self, value: u32) {
        let mut green = value as u16;
        if green & 0x8000 != 0 {green = 0} else if green >= 0x1000 {green = 0x1F} else {green >>= 7}
        self.R[28] = (self.R[28] & !(0x1F << 5)) | ((green as u16 as u32) << 5);
        self.R[28] &= 0x7FFF;
    }

    fn update_irgb_blue(&mut self, value: u32) {
        let mut blue = value as u16;
        if blue & 0x8000 != 0 {blue = 0} else if blue >= 0x1000 {blue = 0x1F} else {blue >>= 7}
        self.R[28] = (self.R[28] & !(0x1F << 10)) | ((blue as u16 as u32) << 10);
        self.R[28] &= 0x7FFF;
    }
}