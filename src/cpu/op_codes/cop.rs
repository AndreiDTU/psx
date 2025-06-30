use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn cop0(&mut self, instruction: u32) {
        let cop_instruction = instruction.rs();
        match cop_instruction {
            0b00000 => self.mfc0(instruction),
            0b00100 => self.mtc0(instruction),
            0b10000 => self.system_control.borrow_mut().rfe(),
            _ => panic!("{:08X} Unsupported cop op: {:06b}..{:05b}", instruction, instruction.op(), cop_instruction)
        }
    }

    fn mfc0(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        
        let value = self.system_control.borrow().read_register(rd);
        self.schedule_write(rt, value);
    }

    fn mtc0(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();

        self.system_control.borrow_mut().write_register(rd, self.R[rt]);
    }

    pub fn cop2(&mut self, instruction: u32) {
        println!("{:08X}, GTE not yet implemented!", instruction);
    }

    pub fn lwc2(&mut self, instruction: u32) {
        println!("{:08X}, GTE not yet implemented!", instruction);
    }

    pub fn swc2(&mut self, instruction: u32) {
        println!("{:08X}, GTE not yet implemented!", instruction);
    }
}