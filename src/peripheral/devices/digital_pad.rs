use std::{cell::RefCell, rc::Weak};

use crate::peripheral::{devices::{Device, DigitalSwitch}, ports::sio0::SIO0};

const DIGITAL_CONTROLLER_INFO: u16 = 0x5A41;

pub struct DigitalPad {
    switches: DigitalSwitch,
    step: usize,
    addressing_controller: bool,
    
    rx: Option<u8>,
    sio: Weak<RefCell<SIO0>>,
}

impl DigitalPad {
    pub fn new(sio: Weak<RefCell<SIO0>>) -> Self {
        Self {
            switches: DigitalSwitch::from_bits_truncate(0xFFFF),
            step: 0,
            addressing_controller: false,
            
            rx: None,
            sio,
        }
    }
}

impl Device for DigitalPad {
    fn send(&mut self, _data: u8) {        
        self.rx = Some(match self.step {
            0 => 0xFF,
            1 => DIGITAL_CONTROLLER_INFO as u8,
            2 => (DIGITAL_CONTROLLER_INFO >> 8) as u8,
            3 => self.switches.bits() as u8,
            4 => {
                self.addressing_controller = false;
                (self.switches.bits() >> 8) as u8
            },
            _ => unreachable!()
        });

        self.step += 1;
        self.step &= 0xF * ((self.step <= 4) as usize); // Reset step counter if above 4
    }

    fn set_switch(&mut self, switch: DigitalSwitch, released: bool) {
        self.switches.set(switch, released);
    }

    fn transfer_rx(&mut self) {
        if let Some(rx) = self.rx {
            self.sio.upgrade().unwrap().borrow_mut().rx_receive(rx);
            self.rx = None;
        }
    }
}