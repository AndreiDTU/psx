#![allow(non_snake_case, non_camel_case_types)]

#[allow(unused_imports)]
use std::{cell::RefCell, ops::{Index, IndexMut}, path::Path, rc::Rc, time::{Duration, Instant}};

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

use crate::{bus::{dma::DMA, interface::Interface, interrupt::Interrupt, timer::Timer}, cd_rom::CD_ROM, cpu::{system_control::SystemControl, CPU}, peripheral::{devices::{digital_pad::DigitalPad, Device}, ports::sio0::SIO0}};

mod bus;
mod bios;
mod cpu;
mod gpu;
mod ram;
mod cd_rom;
mod peripheral;

const VRAM_WIDTH: u32 = 1024;
const VRAM_HEIGHT: u32 = 512;

const NTSC_FRAME_TIME: Duration = Duration::from_nanos(16_866_250);

fn main() -> Result<(), anyhow::Error> {
    // let exe_binding = std::fs::read("psxtest_gte.exe").unwrap();
    // let exe = exe_binding.as_slice();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("PSX", VRAM_WIDTH, VRAM_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, VRAM_WIDTH, VRAM_HEIGHT)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let system_control = Rc::new(RefCell::new(SystemControl::new()));
    let interrupt = Rc::new(RefCell::new(Interrupt::new(system_control.clone())));
    let sio0 = Rc::new(RefCell::new(SIO0::new([const { None }; 2], interrupt.clone())));
    let pad = Rc::new(RefCell::new(Box::new(DigitalPad::new(Rc::downgrade(&sio0))) as Box<dyn Device>));
    sio0.borrow_mut().connect_device(pad.clone(), 0);
    let timer = Rc::new(RefCell::new(Timer::new(interrupt.clone())));
    let cd_rom = Rc::new(RefCell::new(CD_ROM::new(interrupt.clone())));
    let interface = Rc::new(RefCell::new(Interface::new(Path::new("SCPH1001.bin"), interrupt, cd_rom.clone(), timer.clone(), sio0.clone())?));
    let dma_running = Rc::new(RefCell::new(false));
    let dma = Rc::new(RefCell::new(DMA::new(interface.clone(), interface.borrow_mut().interrupt.clone(), dma_running.clone())));
    interface.borrow_mut().dma = Rc::downgrade(&dma);
    let mut cpu = CPU::new(interface.clone(), dma_running, system_control);

    let mut instruction = true;

    let mut frame_start = Instant::now();

    loop {
        if instruction {
            // sideload_exe(&mut cpu, interface.clone(), exe);
            cpu.tick();
        }
        timer.borrow_mut().tick();
        dma.borrow_mut().tick();
        cd_rom.borrow_mut().tick();
        sio0.borrow_mut().tick();
        pad.borrow_mut().transfer_rx();
        if interface.borrow_mut().gpu.tick() {
            let frame: Vec<_> = interface.borrow().gpu.render_vram().iter().flat_map(|color| color.rgb.to_array()).collect();
            texture.update(None, &frame[..], VRAM_WIDTH as usize * 3)?;

            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => return Ok(()),
                    _ => {}
                }
            }

            let frame_time = frame_start.elapsed();
            std::thread::sleep(NTSC_FRAME_TIME.saturating_sub(frame_time));
            frame_start = Instant::now();
        }
        instruction = !instruction;
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