use std::{cell::RefCell, path::Path, rc::Rc};

use crate::{bus::interface::Interface, cpu::CPU};

#[allow(non_snake_case)]

pub mod cpu;
pub mod bus;
pub mod bios;
pub mod ram;

fn main() -> Result<(), anyhow::Error> {
    let interface = Rc::new(RefCell::new(Interface::new(Path::new("SCPH1001.bin"))?));
    let mut cpu = CPU::new(interface.clone());

    loop {
        cpu.tick();
    }
}
