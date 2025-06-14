use std::{cell::RefCell, ops::{Index, IndexMut}, rc::Rc};

use crate::{bus::interface::Interface, cpu::{decoder::Instruction, system_control::SystemControl}};

pub mod decoder;
pub mod system_control;
mod op_codes;

pub struct CPU {
    R: Registers<32>,
    pc: u32,
    hi: u32,
    lo: u32,

    next_instruction: u32,
    pending_writes: [Option<(u32, u32)>; 2],

    system_control: SystemControl,

    interface: Rc<RefCell<Interface>>
}

impl CPU {
    pub fn new(interface: Rc<RefCell<Interface>>) -> Self {
        let R = Registers {R: [0; 32]};
        let pc = 0xBFC0_0000;
        let (hi, lo) = (0, 0);

        CPU {
            R,
            pc,
            hi,
            lo,

            next_instruction: 0,
            pending_writes: [None; 2],

            system_control: SystemControl::new(),

            interface,
        }
    }

    pub fn tick(&mut self) {
        let instruction = self.next_instruction;
        self.next_instruction = self.read32(self.pc);

        self.pc = self.pc.wrapping_add(4);

        self.execute(instruction);
        self.commit_writes();
    }

    pub fn execute(&mut self, instruction: u32) {
        println!("instruction: {:08X}, pc: {:08X}, R31: {:08X}", instruction, self.pc, self.R[31]);
        let op = instruction.op();
        match op {
            0b000000 => {
                let funct = instruction.funct();
                match funct {
                    0b000000 => self.sll(instruction),
                    0b000011 => self.sra(instruction),
                    0b001000 => self.jr(instruction),
                    0b001001 => self.jalr(instruction),
                    0b100000 => self.add(instruction),
                    0b100001 => self.addu(instruction),
                    0b100011 => self.subu(instruction),
                    0b100100 => self.and(instruction),
                    0b100101 => self.or(instruction),
                    0b101011 => self.sltu(instruction),
                    _ => panic!("Unsupported funct: {:06b}..{:06b}", op, funct),
                }
            },
            0b000001 => self.bxx(instruction),
            0b000010 => self.j(instruction),
            0b000011 => self.jal(instruction),
            0b000100 => self.beq(instruction),
            0b000101 => self.bne(instruction),
            0b000110 => self.blez(instruction),
            0b000111 => self.bgtz(instruction),
            0b001000 => self.addi(instruction),
            0b001001 => self.addiu(instruction),
            0b001010 => self.slti(instruction),
            0b001100 => self.andi(instruction),
            0b001101 => self.ori(instruction),
            0b001111 => self.lui(instruction),
            0b010000 => {
                let cop_instruction = instruction.rs();
                match cop_instruction {
                    0b00000 => self.mfc0(instruction),
                    0b00100 => self.mtc0(instruction),
                    _ => panic!("Unsupported cop0 op: {:06b}..{:06b}", op, cop_instruction)
                }
            }
            0b100000 => self.lb(instruction),
            0b100011 => self.lw(instruction),
            0b100100 => self.lbu(instruction),
            0b101000 => self.sb(instruction),
            0b101001 => self.sh(instruction),
            0b101011 => self.sw(instruction),
            _ => panic!("Unsupported op: {:06b}", op),
        }
    }

    fn write32(&mut self, addr: u32, value: u32) {
        if self.system_control.read_register(12) & 0x10000 != 0 {
            println!("Cache not implemented");
            return;
        }

        self.interface.borrow_mut().write32(addr, value);
    }

    fn read32(&self, addr: u32) -> u32 {
        self.interface.borrow().read32(addr)
    }

    fn read8(&self, addr: u32) -> u8 {
        self.interface.borrow().read8(addr)
    }

    fn write16(&mut self, addr: u32, value: u16) {
        if self.system_control.read_register(12) & 0x10000 != 0 {
            println!("Cache not implemented");
            return;
        }
        self.interface.borrow_mut().write16(addr, value);
    }

    fn write8(&mut self, addr: u32, value: u8) {
        if self.system_control.read_register(12) & 0x10000 != 0 {
            println!("Cache not implemented");
            return;
        }
        self.interface.borrow_mut().write8(addr, value);
    }

    fn write_register(&mut self, register: u32, value: u32) {
        self.R[register] = value;
        self.R[0] = 0;
    }

    fn schedule_write(&mut self, register: u32, value: u32) {
        self.pending_writes[1] = Some((register, value))
    }

    fn commit_writes(&mut self) {
        let pending_write = self.pending_writes[0];
        self.pending_writes[0] = self.pending_writes[1];
        self.pending_writes[1] = None;
        if let Some((register, value)) = pending_write {
            self.write_register(register, value);
        }
    }

    fn raise_exception(&mut self) {
        todo!()
    }
}

#[derive(Debug)]
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