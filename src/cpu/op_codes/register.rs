use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn jr(&mut self, instruction: u32) {
        let rs = instruction.rs();

        self.branch = true;
        self.next_pc = self.R[rs];
    }

    pub fn jalr(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rd = instruction.rd();

        self.write_register(rd, self.next_pc);

        self.branch = true;
        self.next_pc = self.R[rs];
    }

    pub fn mfhi(&mut self, instruction: u32) {
        let rd = instruction.rd();

        self.write_register(rd, self.hi);
    }

    pub fn mthi(&mut self, instruction: u32) {
        let rs = instruction.rs();
        
        self.hi = self.R[rs];
    }

    pub fn mflo(&mut self, instruction: u32) {
        let rd = instruction.rd();
        
        self.write_register(rd, self.lo);
    }

    pub fn mtlo(&mut self, instruction: u32) {
        let rs = instruction.rs();

        self.lo = self.R[rs];
    }

    pub fn slt(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = if (self.R[rs] as i32) < (self.R[rt] as i32) {1} else {0};
        self.write_register(rd, value);
    }

    pub fn sltu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = if self.R[rs] < self.R[rt] {1} else {0};
        self.write_register(rd, value);
    }

    pub fn bxx(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let offset = instruction.imm_se();

        let greater = (instruction >> 16) & 1;
        let link = (instruction >> 17) & 0xF == 8;

        let value = self.R[rs] as i32;

        let test = ((value < 0) as u32) ^ greater;

        if link {self.write_register(31, self.next_pc)}

        if test != 0 {
            self.branch(offset);
        }
    }

    pub fn beq(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        if self.R[rs] == self.R[rt] {
            self.branch(offset);
        }
    }

    pub fn bne(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let offset = instruction.imm_se();

        if self.R[rs] != self.R[rt] {
            self.branch(offset);
        }
    }

    pub fn blez(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let offset = instruction.imm_se();

        if (self.R[rs] as i32) <= 0 {
            self.branch(offset);
        }
    }

    pub fn bgtz(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let offset = instruction.imm_se();

        if (self.R[rs] as i32) > 0 {
            self.branch(offset);
        }
    }

    pub fn j(&mut self, instruction: u32) {
        let target = instruction.target();

        self.branch = true;
        self.next_pc = (self.next_pc & 0xF000_0000) | (target << 2);
    }

    pub fn jal(&mut self, instruction: u32) {
        self.write_register(31, self.next_pc);

        self.j(instruction);
    }

    pub fn slti(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se();

        let value = if (self.R[rs] as i32) < (imm as i32) {1} else {0};
        self.write_register(rt, value);
    }

    pub fn sltiu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm();

        let value = if self.R[rs] < imm {1} else {0};
        self.write_register(rt, value);
    }

    pub fn lui(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let imm = instruction.imm();

        self.write_register(rt, imm << 16);
    }

    fn branch(&mut self, offset: u32) {
        let offset = offset << 2;

        self.branch = true;
        self.next_pc = self.next_pc.wrapping_add(offset).wrapping_sub(4);
    }
}