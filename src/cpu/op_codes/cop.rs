use crate::cpu::{decoder::{Cause, Instruction}, CPU};

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
        if instruction & (1 << 25) != 0 {
            self.gte.issue_command(instruction);
        } else {
            let cop_instruction = instruction.rs();
            match cop_instruction {
                0b00000 => self.mfc2(instruction),
                0b00010 => self.cfc2(instruction),
                0b00100 => self.mtc2(instruction),
                0b00110 => self.ctc2(instruction),
                0b01000 => self.bc2(instruction),
                _ => panic!("{:08X} Unsupported cop op: {:06b}..{:05b}", instruction, instruction.op(), cop_instruction)
            }
        }
    }

    fn mfc2(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        
        let value = self.gte.read_data_register(rd);
        self.schedule_write(rt, value);
    }

    fn cfc2(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        
        let value = self.gte.read_ctrl_register(rd);
        self.schedule_write(rt, value);
    }

    fn mtc2(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();

        self.gte.write_data_register(rd, self.R[rt]);
    }

    fn ctc2(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();

        self.gte.write_ctrl_register(rd, self.R[rt]);
    }

    fn bc2(&mut self, instruction: u32) {
        if instruction & 0x0001_0000 != 0 {
            let offset = instruction.imm_se();
            self.branch(offset);
        }
    }

    pub fn lwc2(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b11 != 0 {
            self.raise_exception(Cause::AdEL);
            return;
        }

        let value = self.read32(addr);
        self.gte.write_data_register(rt, value);
    }

    pub fn swc2(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        let addr = self.R[rs].wrapping_add(offset);
        if addr & 0b11 != 0 {
            self.raise_exception(Cause::AdES);
            return;
        }

        let value = self.gte.read_data_register(rt);
        self.write32(addr, value);
    }
}