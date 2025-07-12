use std::{cell::RefCell, collections::VecDeque, path::Path, rc::Rc};

use bitflags::bitflags;

use crate::{bus::interrupt::{Interrupt, IRQ}, cd_rom::bin::{sector::Sector, DiskAddress, DiskMap, DiskTrait}};

mod command;
mod bin;

const AVERAGE_IRQ_DELAY: usize = 0xC4E1;

bitflags! {
    pub struct CD_ROM_STATUS: u8 {
        const PLAY     = 0x80;
        const SEEK     = 0x40;
        const READ     = 0x20;
        const SHELL    = 0x10;
        const ID_ERR   = 0x08;
        const SEEK_ERR = 0x04;
        const SPINDLE  = 0x02;
        const ERROR    = 0x01;
    }
}

bitflags! {
    pub struct CD_ROM_MODE: u8 {
        const SPEED       = 0x80;
        const XA_ADPCM    = 0x40;
        const SECTOR_SIZE = 0x20;
        const IGNORE      = 0x10;
        const XA_FILTER   = 0x08;
        const REPORT      = 0x04;
        const AUTO_PAUSE  = 0x02;
        const CDDA        = 0x01;
    }
}

pub struct CD_ROM {
    disk: DiskMap,
    sector_buffer: [Option<Sector>; 2],
    sector_pointer: usize,
    read_func: fn(&mut CD_ROM) -> u8,

    status: CD_ROM_STATUS,
    mode: CD_ROM_MODE,

    registers: [u8; 16],
    current_bank: usize,

    parameters: VecDeque<u8>,

    result_fifo: [u8; 16],
    result_idx: usize,
    result_size: usize,
    result_fifo_empty: bool,
    second_response: SecondResponse,

    irq_delay: usize,
    irq_pending: bool,
    irq: u8,

    seek_target: DiskAddress,
    read_addr: DiskAddress,

    interrupt: Rc<RefCell<Interrupt>>,
}

impl CD_ROM {
    pub fn new<P>(interrupt: Rc<RefCell<Interrupt>>, bin_path: P) -> anyhow::Result<CD_ROM>
    where P: AsRef<Path> {
        Ok(Self {
            disk: DiskMap::from_bin(bin_path)?,
            sector_buffer: [None; 2],
            sector_pointer: 0,
            read_func: CD_ROM::read_0x800,

            status: CD_ROM_STATUS::from_bits_truncate(0),
            mode: CD_ROM_MODE::from_bits_truncate(0),

            registers: [0; 16],
            current_bank: 0,

            parameters: VecDeque::with_capacity(16),
            result_fifo: [0; 16],
            result_idx: 0,
            result_size: 0,
            result_fifo_empty: false,
            second_response: SecondResponse::None,

            irq_delay: AVERAGE_IRQ_DELAY,
            irq_pending: false,
            irq: 0,

            seek_target: DiskAddress::default(),
            read_addr: DiskAddress::default(),
            
            interrupt,
        })
    }

    pub fn tick(&mut self) {
        if self.irq_pending {
            self.irq_delay -= 1;
            if self.irq_delay == 0 {
                self.registers[HINTSTS] = (self.registers[HINTSTS] & !7) | self.irq;
                self.interrupt.borrow_mut().request(IRQ::CDROM);
                self.irq_delay = AVERAGE_IRQ_DELAY;
                self.irq_pending = false;
                // println!("Firing CD-ROM INT{}", self.registers[HINTSTS] & 7);
                match self.second_response {
                    SecondResponse::GetID => self.get_id_second_response(),
                    SecondResponse::SeekL => self.seekL_second_response(),
                    SecondResponse::ReadN => self.readN_second_response(),
                    SecondResponse::Pause => self.pause_second_response(),
                    SecondResponse::Init => self.init_second_response(),
                    SecondResponse::None => {}
                }
            }
        }
    }

    pub fn read8(&mut self, offset: u32) -> u8 {
        let register = READ_BANKS[self.current_bank][offset as usize];
        let value = match register {
            HSTS => {
                // if !self.result_fifo.is_empty() {println!("{:#?}", self.result_fifo)};

                self.registers[HSTS] = (self.registers[HSTS] & !0x08) | 0x08 * (self.parameters.is_empty() as u8);
                self.registers[HSTS] = (self.registers[HSTS] & !0x20) | 0x20 * (!self.result_fifo_empty as u8);

                self.registers[HSTS]
            }
            RESULT => {
                let result = self.result_fifo[self.result_idx];
                
                self.result_idx += 1;
                self.result_fifo_empty = self.result_fifo_empty || !(self.result_idx == self.result_size);
                self.result_idx &= 0xF;

                result
            }
            RDDATA => {
                (self.read_func)(self)
            },
            _ => self.registers[register]
        };
        // println!("CDROM bank {} [{offset}] = {value:02X}", self.current_bank);
        value
    }

    fn read_0x800(&mut self) -> u8 {
        const RDDATA_0X800: [fn(&mut CD_ROM) -> u8; 2] = [CD_ROM::read_0x800, CD_ROM::pad_0x800];

        let byte = self.sector_buffer[0].unwrap()[self.sector_pointer + 12];
        
        self.sector_pointer += 1;
        let pad = self.sector_pointer == 0x800;
        self.read_func = RDDATA_0X800[pad as usize];

        byte
    }

    fn pad_0x800(&mut self) -> u8 {
        self.sector_buffer[0].unwrap()[const {0x800 - 8 + 12}]
    }

    fn read_0x924(&mut self) -> u8 {
        const RDDATA_0X924: [fn(&mut CD_ROM) -> u8; 2] = [CD_ROM::read_0x924, CD_ROM::pad_0x924];

        let byte = self.sector_buffer[0].unwrap()[self.sector_pointer];
        
        self.sector_pointer += 1;
        let pad = self.sector_pointer == 0x924;
        self.read_func = RDDATA_0X924[pad as usize];

        byte
    }

    fn pad_0x924(&mut self) -> u8 {
        self.sector_buffer[0].unwrap()[const {0x924 - 4}]
    }

    pub fn write8(&mut self, offset: u32, value: u8) {
        // println!("CDROM bank {} [{offset}] <- {value:02X}", self.current_bank);
        let register = WRITE_BANKS[self.current_bank][offset as usize];
        match register {
            ADDRESS => self.registers[ADDRESS] = (self.registers[ADDRESS] & !3) | (value & 3),
            PARAMETER => {
                self.parameters.push_back(value);
                self.registers[ADDRESS] |= 0x10;
            }
            COMMAND => self.execute(value),
            HCLRCTL => {
                self.registers[HINTSTS] &= !(value & 0x1F);
                if value & 0x40 != 0 {
                    self.parameters.clear();
                }
            }
            _ => self.registers[register] = value,
        }

        self.current_bank = self.registers[ADDRESS] as usize & 3;
    }

    fn execute(&mut self, command: u8) {
        self.result_idx = 0;
        // println!("CD-ROM command: {command:02X}");
        match command {
            0x01 => self.send_status(3),
            0x02 => self.setloc(),
            0x06 => self.readN(),
            0x09 => self.pause(),
            0x0A => self.init(),
            0x0E => self.setmode(),
            0x15 => self.seekL(),
            0x19 => self.test(),
            0x1A => self.get_id(),
            _ => panic!("CD-ROM command not yet implemented. {command:02X}"),
        }
    }

    fn schedule_int(&mut self, int: u8) {
        self.irq = int;
        
        if self.registers[HINTMSK] & ((self.registers[HINTSTS] & !7) | int) != 0 {
            self.irq_pending = true;
        }
    }
}

const READ_BANKS: [[usize; 4]; 4] = [
    [HSTS, RESULT, RDDATA, HINTMSK],
    [HSTS, RESULT, RDDATA, HINTSTS],
    [HSTS, RESULT, RDDATA, HINTMSK],
    [HSTS, RESULT, RDDATA, HINTSTS],
];

const WRITE_BANKS: [[usize; 4]; 4] = [
    [ADDRESS, COMMAND, PARAMETER, HCHPCTL],
    [ADDRESS, WRDATA,  HINTMSK,   HCLRCTL],
    [ADDRESS, CI,      ATV0,      ATV1   ],
    [ADDRESS, ATV2,    ATV3,      ADPCTL ],
];

const HSTS:      usize = 0;
const RESULT:    usize = 1;
const RDDATA:    usize = 2;
const HINTMSK:   usize = 3;
const HINTSTS:   usize = 4;
const ADDRESS:   usize = HSTS;
const COMMAND:   usize = 5;
const PARAMETER: usize = 6;
const HCHPCTL:   usize = 7;
const WRDATA:    usize = 8;
const HCLRCTL:   usize = 9;
const CI:        usize = 10;
const ATV0:      usize = 11;
const ATV1:      usize = 12;
const ATV2:      usize = 13;
const ATV3:      usize = 14;
const ADPCTL:    usize = 15;

enum SecondResponse {
    None,
    GetID,
    SeekL,
    ReadN,
    Pause,
    Init,
}