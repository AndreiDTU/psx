use std::{cell::RefCell, ops::{Index, IndexMut}, rc::Rc};

use crate::{bus::interface::Interface, cpu::{decoder::{Cause, Instruction}, system_control::SystemControl}};

pub mod decoder;
pub mod system_control;
mod op_codes;

pub struct CPU {
    R: Registers<32>,
    pc: u32,
    hi: u32,
    lo: u32,

    current_pc: u32,
    next_pc: u32,
    pending_writes: [Option<(u32, u32)>; 2],
    branch: bool,
    delay_slot: bool,

    system_control: SystemControl,

    interface: Rc<RefCell<Interface>>,

    trace: bool,
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

            current_pc: pc,
            next_pc: pc.wrapping_add(4),
            pending_writes: [None; 2],
            branch: false,
            delay_slot: false,

            system_control: SystemControl::new(),

            interface,

            trace: false
        }
    }

    pub fn tick(&mut self) {
        let instruction = self.read32(self.pc);

        self.delay_slot = self.branch;
        self.branch = false;

        self.current_pc = self.pc;

        if self.current_pc & 0b11 != 0 {
            self.raise_exception(Cause::AdEL);
        }

        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        self.execute(instruction);
        self.commit_writes();
    }

    pub fn execute(&mut self, instruction: u32) {
        if self.trace {println!("instruction: {:08X}, pc: {:08X}, R31: {:08X}", instruction, self.pc, self.R[31])};
        let op = instruction.op();
        match op {
            0b000000 => {
                let funct = instruction.funct();
                match funct {
                    0b000000 => self.sll(instruction),
                    0b000010 => self.srl(instruction),
                    0b000011 => self.sra(instruction),
                    0b000100 => self.sllv(instruction),
                    0b000110 => self.srlv(instruction),
                    0b000111 => self.srav(instruction),
                    0b001100 => self.raise_exception(Cause::Sys),
                    0b001000 => self.jr(instruction),
                    0b001001 => self.jalr(instruction),
                    0b010000 => self.mfhi(instruction),
                    0b010001 => self.mthi(instruction),
                    0b010010 => self.mflo(instruction),
                    0b010011 => self.mtlo(instruction),
                    0b011010 => self.div(instruction),
                    0b011001 => self.multu(instruction),
                    0b011011 => self.divu(instruction),
                    0b100000 => self.add(instruction),
                    0b100001 => self.addu(instruction),
                    0b100011 => self.subu(instruction),
                    0b100100 => self.and(instruction),
                    0b100101 => self.or(instruction),
                    0b100110 => self.xor(instruction),
                    0b100111 => self.nor(instruction),
                    0b101010 => self.slt(instruction),
                    0b101011 => self.sltu(instruction),
                    _ => panic!("{:08X} Unsupported funct: {:06b}..{:06b}", instruction, op, funct),
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
            0b001011 => self.sltiu(instruction),
            0b001100 => self.andi(instruction),
            0b001101 => self.ori(instruction),
            0b001111 => self.lui(instruction),
            0b010000 => {
                let cop_instruction = instruction.rs();
                match cop_instruction {
                    0b00000 => self.mfc0(instruction),
                    0b00100 => self.mtc0(instruction),
                    0b10000 => self.system_control.rfe(),
                    _ => panic!("{:08X} Unsupported cop0 op: {:06b}..{:05b}", instruction, op, cop_instruction)
                }
            }
            0b100000 => self.lb(instruction),
            0b100001 => self.lh(instruction),
            0b100011 => self.lw(instruction),
            0b100100 => self.lbu(instruction),
            0b100101 => self.lhu(instruction),
            0b101000 => self.sb(instruction),
            0b101001 => self.sh(instruction),
            0b101011 => self.sw(instruction),
            _ => panic!("{:08X} Unsupported op: {:06b}", instruction, op),
        }
    }

    fn raise_exception(&mut self, cause: Cause) {
        println!("Raised exception on cause: {:#?}", cause);

        self.pc = if self.system_control.raise_exception(cause as u32, self.current_pc, self.delay_slot) {
            0xBFC0_0180
        } else {
            0x8000_0080
        };

        self.next_pc = self.pc.wrapping_add(4);
    }

    fn read32(&self, addr: u32) -> u32 {
        self.interface.borrow().read32(addr)
    }

    fn read16(&self, addr: u32) -> u16 {
        self.interface.borrow_mut().read16(addr)
    }

    fn read8(&self, addr: u32) -> u8 {
        self.interface.borrow().read8(addr)
    }

    fn write32(&mut self, addr: u32, value: u32) {
        if self.system_control.read_register(12) & 0x10000 != 0 {
            println!("Cache not implemented");
            return;
        }

        self.interface.borrow_mut().write32(addr, value);
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