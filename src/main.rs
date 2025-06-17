#![allow(non_snake_case)]

use std::{cell::RefCell, ops::{Index, IndexMut}, path::Path, rc::Rc};

use crate::{bus::{dma::DMA, interface::Interface}, cpu::CPU};

pub mod bus;
pub mod bios;
pub mod cpu;
pub mod gpu;
pub mod ram;

fn main() -> Result<(), anyhow::Error> {
    // let exe_binding = std::fs::read("psxtest_cpu.exe").unwrap();
    // let exe = exe_binding.as_slice();

    let interface = Rc::new(RefCell::new(Interface::new(Path::new("SCPH1001.bin"))?));
    let dma_running = Rc::new(RefCell::new(false));
    let dma = Rc::new(RefCell::new(DMA::new(interface.clone(), dma_running.clone())));
    interface.borrow_mut().dma = Rc::downgrade(&dma);
    let mut cpu = CPU::new(interface.clone(), dma_running.clone());

    loop {
        // sideload_exe(&mut cpu, interface.clone(), exe);
        cpu.tick();
        dma.borrow_mut().tick();
        interface.borrow_mut().gpu.tick();
    }
}

#[allow(unused)]
fn sideload_exe(cpu: &mut CPU, interface: Rc<RefCell<Interface>>, exe: &[u8]) {
    if cpu.pc != 0x80030000 {return}

    let initial_pc = u32::from_le_bytes(*exe[0x10..].first_chunk().unwrap());
    let initial_r28 = u32::from_le_bytes(*exe[0x14..].first_chunk().unwrap());
    let exe_ram_addr = u32::from_le_bytes(*exe[0x18..].first_chunk().unwrap()) & 0x001F_FFFF;
    let exe_size_2kb = u32::from_le_bytes(*exe[0x1C..].first_chunk().unwrap());
    let initial_sp = u32::from_le_bytes(*exe[0x30..].first_chunk().unwrap());

    let exe_size = exe_size_2kb;
    interface.borrow_mut().dram.data[exe_ram_addr as usize..(exe_ram_addr + exe_size) as usize]
        .copy_from_slice(&exe[2048..2048 + exe_size as usize]);

    cpu.R[28] = initial_r28;
    if initial_sp != 0 {
        cpu.R[29] = initial_sp;
        cpu.R[30] = initial_sp;
    }

    cpu.next_pc = initial_pc;
}

#[derive(Debug, Clone, Copy)]
pub struct Registers<const N: usize> {
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