use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn addu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs].wrapping_add(self.R[rt]);
        self.write_register(rd, value);
    }
    
    pub fn addi(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se();

        if let Some(value) = self.R[rs].checked_add(imm) {
            self.write_register(rt, value);
        } else {
            self.raise_exception();
        }
    }

    pub fn addiu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se();

        let value = self.R[rs].wrapping_add(imm);
        self.write_register(rt, value);
    }
}