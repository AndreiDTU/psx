use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn lb(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.read8(addr) as i8 as u32;

        self.schedule_write(rt, value);
    }

    pub fn lw(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
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
        let value = self.R[rt] as u16;

        self.write16(addr, value);
    }

    pub fn sw(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        let value = self.R[rt];

        self.write32(addr, value);
    }
}