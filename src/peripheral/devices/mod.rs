use bitflags::bitflags;

pub mod digital_pad;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct DigitalSwitch: u16 {
        const SELECT   = 0x0001;
        const L3       = 0x0002;
        const R3       = 0x0004;
        const START    = 0x0008;
        const UP       = 0x0010;
        const RIGHT    = 0x0020;
        const DOWN     = 0x0040;
        const LEFT     = 0x0080;
        const L2       = 0x0100;
        const R2       = 0x0200;
        const L1       = 0x0400;
        const R1       = 0x0800;
        const TRIANGLE = 0x1000;
        const CIRCLE   = 0x2000;
        const CROSS    = 0x4000;
        const SQUARE   = 0x8000;
    }
}

pub trait Device {
    fn send(&mut self, data: u8);
    fn set_switch(&mut self, switch: DigitalSwitch, released: bool);
    fn transfer_rx(&mut self);
}