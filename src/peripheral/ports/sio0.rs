use std::{cell::RefCell, rc::Rc};

use crate::{bus::interrupt::{Interrupt, IRQ}, peripheral::{devices::Device, ports::{SIO_CTRL, SIO_MODE, SIO_STAT}}};
const ACK_IRQ_DELAY: usize = 1088;

pub struct SIO0 {
    TX_DATA: u32,
    RX_DATA: u32,

    SIO_STAT: SIO_STAT,
    SIO_CTRL: SIO_CTRL,
    SIO_MODE: SIO_MODE,

    SIO_BAUD: u16,

    devices: [Option<Rc<RefCell<Box<dyn Device>>>>; 2],

    interrupt: Rc<RefCell<Interrupt>>,
    bits_sent: usize,
    irq_delay: usize,
    irq_pending: bool,
}

impl SIO0 {
    pub fn new(devices: [Option<Rc<RefCell<Box<dyn Device>>>>; 2], interrupt: Rc<RefCell<Interrupt>>) -> Self {
        Self {
            TX_DATA: 0,
            RX_DATA: 0,
            
            SIO_STAT: SIO_STAT::from_bytes(0x0000_0005_u32.to_le_bytes()),
            SIO_CTRL: SIO_CTRL::new(),
            SIO_MODE: SIO_MODE::new(),
            
            SIO_BAUD: 0,
            
            devices,
            
            interrupt,
            bits_sent: 0,
            irq_delay: 0,
            irq_pending: false,
        }
    }

    pub fn tick(&mut self) {
        let current_timer = self.SIO_STAT.baudrate_timer();
        if current_timer != 0 {
            self.SIO_STAT.set_baudrate_timer(current_timer - 1);
        } else {
            if self.SIO_STAT.tx_fifo_not_full() == 0 {
                self.bits_sent += 1;
                if self.bits_sent == 8 {
                    if let Some(device) = &mut self.devices[self.SIO_CTRL.sio0_port_select() as usize] {
                        device.borrow_mut().send(self.TX_DATA as u8);
                    }
                    self.irq_pending = true;
                    self.irq_delay = ACK_IRQ_DELAY;
                    self.TX_DATA = 0;
                    self.SIO_STAT.set_tx_fifo_not_full(1);
                    self.SIO_STAT.set_tx_idle(1);
                    // println!("TX FIFO empty");
                    self.bits_sent = 0;
                }
                // println!("SIO0 bits_sent {}", self.bits_sent);
            }

            self.reload_timer();
        }

        if self.irq_pending {
            self.irq_delay -= 1;
            if self.irq_delay == 0 {
                // println!("byte received");
                self.irq_pending = false;
                self.interrupt.borrow_mut().request(IRQ::BYTE_RECEIVED);
                self.SIO_STAT.set_irq(1);
                self.SIO_STAT.set_rx_fifo_not_empty(1);
            }
        }
    }

    pub fn connect_device(&mut self, device: Rc<RefCell<Box<dyn Device>>>, port: usize) {
        self.devices[port] = Some(device);
    }

    pub fn read32(&mut self, offset: u32) -> u32 {
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                let data = self.RX_DATA;

                self.RX_DATA >>= 4;
                if self.RX_DATA == 0 {self.SIO_STAT.set_rx_fifo_not_empty(0);}

                data
            }
            0x04 => u32::from_le_bytes(self.SIO_STAT.into_bytes()),
            0x08 => u16::from_le_bytes(self.SIO_MODE.into_bytes()) as u32 | ((u16::from_le_bytes(self.SIO_CTRL.into_bytes()) as u32) << 16),
            0x0C => (self.SIO_BAUD as u32) << 16,
            _ => unreachable!()
        }
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                self.TX_DATA = value & 0xFF;
                self.SIO_STAT.set_tx_fifo_not_full(0);
                self.SIO_STAT.set_tx_idle(0);
            }
            0x04 => self.SIO_MODE = SIO_MODE::from_bytes(((value >> 16) as u16 & 0x013F).to_le_bytes()),
            0x08 => {
                self.SIO_MODE = SIO_MODE::from_bytes((value as u16 & 0x013F).to_le_bytes());
                self.SIO_CTRL = SIO_CTRL::from_bytes(((value >> 16) as u16 & 0x3F7F).to_le_bytes());
                if self.SIO_CTRL.acknowledge() != 0 {
                    // println!("Clearing IRQ");
                    self.SIO_STAT.set_irq(0);
                }
            }
            0x0C => {
                self.SIO_BAUD = (value >> 16) as u16;
                self.reload_timer();
            }
            _ => unreachable!()
        }
    }

    pub fn read16(&mut self, offset: u32) -> u16 {
        // println!("SI0 read 16-bit offset: {offset:02X}, SIO0_STAT: {:08X}", u32::from_le_bytes(self.SIO_STAT.into_bytes()));
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                let data = self.RX_DATA as u16;

                self.RX_DATA >>= 4;
                if self.RX_DATA == 0 {self.SIO_STAT.set_rx_fifo_not_empty(0);}

                data
            }
            0x02 => (self.RX_DATA >> 16) as u16,
            0x04 => u32::from_le_bytes(self.SIO_STAT.into_bytes()) as u16,
            // 0x04 => 0xFF,
            0x06 => (u32::from_le_bytes(self.SIO_STAT.into_bytes()) >> 16) as u16,
            0x08 => u16::from_le_bytes(self.SIO_MODE.into_bytes()),
            0x0A => u16::from_le_bytes(self.SIO_CTRL.into_bytes()),
            0x0C => 0,
            0x0E => self.SIO_BAUD,
            _ => unreachable!()
        }
    }

    pub fn write16(&mut self, offset: u32, value: u16) {
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                self.TX_DATA = value as u32 & 0xFF;
                self.SIO_STAT.set_tx_fifo_not_full(0);
                self.SIO_STAT.set_tx_idle(0);
            }
            0x02 => {}
            0x04 => {}
            0x06 => {}
            0x08 => self.SIO_MODE = SIO_MODE::from_bytes((value & 0x013F).to_le_bytes()),
            0x0A => self.SIO_CTRL = SIO_CTRL::from_bytes((value & 0x3F7F).to_le_bytes()),
            0x0C => {}
            0x0E => {
                self.SIO_BAUD = value;
                self.reload_timer();
            }
            _ => unreachable!()
        }
    }

    pub fn read8(&mut self, offset: u32) -> u8 {
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                let data = self.RX_DATA as u8;

                self.RX_DATA >>= 4;
                if self.RX_DATA == 0 {self.SIO_STAT.set_rx_fifo_not_empty(0);}

                data
            }
            0x01 => (self.RX_DATA >> 8) as u8,
            0x02 => (self.RX_DATA >> 16) as u8,
            0x03 => (self.RX_DATA >> 24) as u8,
            0x04 => u32::from_le_bytes(self.SIO_STAT.into_bytes()) as u8,
            0x05 => (u32::from_le_bytes(self.SIO_STAT.into_bytes()) >> 8) as u8,
            0x06 => (u32::from_le_bytes(self.SIO_STAT.into_bytes()) >> 16) as u8,
            0x07 => (u32::from_le_bytes(self.SIO_STAT.into_bytes()) >> 24) as u8,
            0x08 => u16::from_le_bytes(self.SIO_MODE.into_bytes()) as u8,
            0x09 => (u16::from_le_bytes(self.SIO_MODE.into_bytes()) >> 8) as u8,
            0x0A => u16::from_le_bytes(self.SIO_CTRL.into_bytes()) as u8,
            0x0B => (u16::from_le_bytes(self.SIO_CTRL.into_bytes()) >> 8) as u8,
            0x0C => 0,
            0x0D => 0,
            0x0E => self.SIO_BAUD as u8,
            0x0F => (self.SIO_BAUD >> 8) as u8,
            _ => unreachable!()
        }
    }

    pub fn write8(&mut self, offset: u32, value: u8) {
        let offset = offset & 0xF;
        match offset {
            0x00 => {
                self.TX_DATA = value as u32 & 0xFF;
                self.SIO_STAT.set_tx_fifo_not_full(0);
                self.SIO_STAT.set_tx_idle(0);
            }
            0x01..=0x03 => {}
            0x04..=0x07 => {}
            0x08 => self.SIO_MODE = SIO_MODE::from_bytes((((self.SIO_MODE.sio0_clock_polarity() as u16) << 8) | value as u16).to_le_bytes()),
            0x09 => self.SIO_MODE.set_sio0_clock_polarity(value & 1),
            0x0A => self.SIO_CTRL = SIO_CTRL::from_bytes(((u16::from_le_bytes(self.SIO_CTRL.into_bytes()) & 0xFF00) | (value as u16)).to_le_bytes()),
            0x0B => self.SIO_CTRL = SIO_CTRL::from_bytes(((u16::from_le_bytes(self.SIO_CTRL.into_bytes()) & 0x00FF) | ((value as u16) << 8)).to_le_bytes()),
            0x0C..=0x0D => {}
            0x0E => {
                self.SIO_BAUD = (self.SIO_BAUD & 0xFF00) | (value as u16);
                self.reload_timer();
            }
            0x0F => {
                self.SIO_BAUD = (self.SIO_BAUD & 0x00FF) | ((value as u16) << 8);
                self.reload_timer();
            }
            _ => unreachable!()
        }
    }

    fn reload_timer(&mut self) {
        let baud_factor: u32 = match self.SIO_MODE.baudrate_reload_factor() {
            0 | 1 => 0, // MUL1
            2 => 4,     // MUL16
            3 => 6,     // MUL64
            _ => unreachable!()
        };

        let reload = ((self.SIO_BAUD as u32) << baud_factor) >> 1;
        self.SIO_STAT.set_baudrate_timer(reload);
    }

    pub fn rx_receive(&mut self, data: u8) {
        self.RX_DATA = data as u32;
    }
}