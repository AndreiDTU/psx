#![allow(non_snake_case)]

use std::{cell::RefCell, ops::{Index, IndexMut}, path::Path, rc::Rc};

use crate::{bus::{dma::DMA, interface::Interface}, cpu::CPU};

pub mod bus;
pub mod bios;
pub mod cpu;
pub mod ram;

fn main() -> Result<(), anyhow::Error> {
    let interface = Rc::new(RefCell::new(Interface::new(Path::new("SCPH1001.bin"))?));

    let dma_running = Rc::new(RefCell::new(false));
    let dma = Rc::new(RefCell::new(DMA::new(interface.clone(), dma_running.clone())));
    interface.borrow_mut().dma = Rc::downgrade(&dma);
    let mut cpu = CPU::new(interface.clone());
    cpu.dma_running = Rc::downgrade(&dma_running);

    loop {
        cpu.tick();
        dma.borrow_mut().tick();
    }
}

#[derive(Debug, Clone, Copy)]
struct Registers<const N: usize> {
    R: [u32; N],
}

impl<const N: usize> Index<u32> for Registers<N> {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        &self.R[index as usize]
    }
}

impl<const N: usize> IndexMut<u32> for Registers<N> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.R[index as usize]
    }
}