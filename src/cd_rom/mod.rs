use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use bitflags::bitflags;

use crate::bus::interrupt::{Interrupt, IRQ};

const AVERAGE_IRQ_DELAY: usize = 50000;
const ID_SECOND_DELAY: usize = 0x4A00;

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

pub struct CD_ROM {
    status: CD_ROM_STATUS,
    registers: [u8; 16],
    current_bank: usize,

    parameters: VecDeque<u8>,

    result_fifo: [u8; 16],
    result_idx: usize,
    result_size: usize,
    result_fifo_empty: bool,
    second_response: SecondResponse,

    irq_delay: usize,
    irq: bool,

    interrupt: Rc<RefCell<Interrupt>>,
}

impl CD_ROM {
    pub fn new(interrupt: Rc<RefCell<Interrupt>>) -> Self {
        Self {
            status: CD_ROM_STATUS::from_bits_truncate(0),
            registers: [0; 16],
            current_bank: 0,

            parameters: VecDeque::with_capacity(16),
            result_fifo: [0; 16],
            result_idx: 0,
            result_size: 0,
            result_fifo_empty: false,
            second_response: SecondResponse::None,

            irq_delay: AVERAGE_IRQ_DELAY,
            irq: false,
            
            interrupt,
        }
    }

    pub fn tick(&mut self) {
        if self.irq {
            self.irq_delay -= 1;
            if self.irq_delay == 0 {
                self.interrupt.borrow_mut().request(IRQ::CDROM);
                self.irq_delay = AVERAGE_IRQ_DELAY;
                self.irq = false;
                match self.second_response {
                    SecondResponse::GetID => self.get_id_second_response(),
                    _ => {}
                }
                self.second_response = SecondResponse::None;
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
            HINTMSK => self.registers[HINTMSK],
            HINTSTS => self.registers[HINTSTS],
            _ => self.registers[register]
        };
        // println!("CDROM bank {} [{offset}] = {value:02X}", self.current_bank);
        value
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
        println!("CD-ROM command: {command:02X}");
        match command {
            0x01 => self.send_status(),
            0x1A => self.get_id(),
            0x19 => self.test(),
            _ => panic!("CD-ROM command not yet implemented. {command:02X}"),
        }
    }

    fn send_status(&mut self) {
        self.result_fifo[self.result_idx] = self.status.bits();
        self.result_size = 0;
        self.result_fifo_empty = false;

        self.schedule_int(3);
    }

    fn get_id(&mut self) {
        self.status.insert(CD_ROM_STATUS::SHELL);
        self.send_status();
        self.second_response = SecondResponse::GetID;
    }

    fn get_id_second_response(&mut self) {
        self.result_idx = 0;
        self.result_size = 8;
        *self.result_fifo[self.result_idx..].first_chunk_mut().unwrap() = NO_DISK;
        self.status.insert(CD_ROM_STATUS::SHELL);
        self.schedule_int(5);
        self.irq_delay = ID_SECOND_DELAY;
    }

    fn test(&mut self) {
        let sub_op = self.parameters.pop_front().unwrap();
        println!("CD-ROM test sub-op: {sub_op:02X}");
        match sub_op {
            0x20 => self.test_version(),
            _ => panic!("CD-ROM test sub-op not yet implemented. {sub_op:02X}"),
        }
    }

    fn test_version(&mut self) {
        *self.result_fifo[self.result_idx..].first_chunk_mut().unwrap() = VERSION;
        self.result_size = 3;
        self.result_fifo_empty = false;

        self.schedule_int(3);
    }

    fn schedule_int(&mut self, int: u8) {
        self.registers[HINTSTS] = (self.registers[HINTSTS] & !7) | int;
        
        if self.registers[HINTMSK] & self.registers[HINTSTS] != 0 {
            self.irq = true;
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

const VERSION: [u8; 4] = [0x94, 0x09, 0x19, 0xC0];
const NO_DISK: [u8; 8] = [0x08, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

enum SecondResponse {
    None,
    GetID,
}