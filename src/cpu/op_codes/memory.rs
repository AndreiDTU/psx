use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn lw(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(imm);
        let value = self.read32(addr);
        self.schedule_write(rt, value);
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