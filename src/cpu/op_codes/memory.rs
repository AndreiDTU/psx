use std::hint::unreachable_unchecked;

use crate::cpu::{decoder::{Cause, Instruction}, CPU};

impl CPU {
    pub fn lb(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.read8(addr) as i8 as u32;

        self.schedule_write(rt, value);
    }

    pub fn lh(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b1 != 0 {
            self.raise_exception(Cause::AdEL);
            return;
        }

        let value = self.read16(addr) as i16 as u32;
        self.schedule_write(rt, value);
    }

    pub fn lwl(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let mut current_value = self.R[rt];
        
        if let Some((r, v)) = self.pending_writes[0] {
            if r == rt {current_value = v}
        }

        let aligned_word = self.read32(addr & !0b11);

        let value = match addr & 3 {
            0 => (current_value & 0x00FF_FFFF) | (aligned_word << 24),
            1 => (current_value & 0x0000_FFFF) | (aligned_word << 16),
            2 => (current_value & 0x0000_00FF) | (aligned_word <<  8),
            3 => (current_value & 0x0000_0000) | (aligned_word <<  0),
            _ => unsafe { unreachable_unchecked() }
        };

        self.schedule_write(rt, value);
    }

    pub fn lw(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b11 != 0 {
            self.raise_exception(Cause::AdEL);
            return;
        }

        let value = self.read32(addr);
        self.schedule_write(rt, value);
    }

    pub fn lbu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.read8(addr) as u32;

        self.schedule_write(rt, value);
    }

    pub fn lhu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b1 != 0 {
            self.raise_exception(Cause::AdEL);
            return;
        }

        let value = self.read16(addr) as u32;
        self.schedule_write(rt, value);
    }

    pub fn lwr(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let mut current_value = self.R[rt];
        
        if let Some((r, v)) = self.pending_writes[0] {
            if r == rt {current_value = v}
        }

        let aligned_word = self.read32(addr & !0b11);

        let value = match addr & 3 {
            0 => (current_value & 0x0000_0000) | (aligned_word >>  0),
            1 => (current_value & 0xFF00_0000) | (aligned_word >>  8),
            2 => (current_value & 0xFFFF_0000) | (aligned_word >> 16),
            3 => (current_value & 0xFFFF_FF00) | (aligned_word >> 24),
            _ => unsafe { unreachable_unchecked() }
        };

        self.schedule_write(rt, value);
    }

    pub fn sb(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.R[rt] as u8;

        self.write8(addr, value);
    }

    pub fn sh(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b1 != 0 {
            self.raise_exception(Cause::AdES);
            return;
        }

        let value = self.R[rt] as u16;
        self.write16(addr, value);
    }

    pub fn swl(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.R[rt];

        let aligned_addr = addr & !0b11;
        let current_mem = self.read32(aligned_addr);

        let mem = match addr & 3 {
            0 => (current_mem & 0xFFFF_FF00) | (value >> 24),
            1 => (current_mem & 0xFFFF_0000) | (value >> 16),
            2 => (current_mem & 0xFF00_0000) | (value >>  8),
            3 => (current_mem & 0x0000_0000) | (value >>  0),
            _ => unsafe { unreachable_unchecked() }
        };

        self.write32(aligned_addr, mem);
    }

    pub fn sw(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b11 != 0 {
            self.raise_exception(Cause::AdES);
            return;
        }

        let value = self.R[rt];
        self.write32(addr, value);
    }

    pub fn swr(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.R[rt];

        let aligned_addr = addr & !0b11;
        let current_mem = self.read32(aligned_addr);

        let mem = match addr & 3 {
            0 => (current_mem & 0x0000_0000) | (value <<  0),
            1 => (current_mem & 0x0000_00FF) | (value <<  8),
            2 => (current_mem & 0x0000_FFFF) | (value << 16),
            3 => (current_mem & 0x00FF_FFFF) | (value << 24),
            _ => unsafe { unreachable_unchecked() }
        };

        self.write32(aligned_addr, mem);
    }
}