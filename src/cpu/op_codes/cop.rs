use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn mtc0(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();

        self.system_control.write_register(rd, self.R[rt]);
    }
}