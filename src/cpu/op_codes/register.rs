use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn bne(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se();

        if self.R[rs] != self.R[rt] {
            self.branch(imm);
        }
    }

    pub fn j(&mut self, instruction: u32) {
        let target = instruction.target();

        self.pc = (self.pc & 0xF000_0000) | (target << 2);
    }

    pub fn lui(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let imm = instruction.imm();

        self.write_register(rt, imm << 16);
    }

    fn branch(&mut self, offset: u32) {
        let offset = offset << 2;

        self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
    }
}