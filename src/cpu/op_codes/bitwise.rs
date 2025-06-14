use crate::cpu::{decoder::Instruction, CPU};

impl CPU {
    pub fn sll(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        let shamt = instruction.shamt();

        let value = self.R[rt] << shamt;
        self.write_register(rd, value);
    }

    pub fn srl(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        let shamt = instruction.shamt();

        let value = self.R[rt] >> shamt;
        self.write_register(rd, value);
    }

    pub fn sra(&mut self, instruction: u32) {
        let rt = instruction.rt();
        let rd = instruction.rd();
        let shamt = instruction.shamt();

        let value = (self.R[rt] as i32) >> shamt;
        self.write_register(rd, value as u32);
    }

    pub fn sllv(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rt] << (self.R[rs] & 0x1F);
        self.write_register(rd, value);
    }

    pub fn srlv(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rt] >> (self.R[rs] & 0x1F);
        self.write_register(rd, value);
    }

    pub fn srav(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = (self.R[rt] as i32) >> (self.R[rs] & 0x1F);
        self.write_register(rd, value as u32);
    }

    pub fn and(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs] & self.R[rt];
        self.write_register(rd, value);
    }

    pub fn or(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs] | self.R[rt];
        self.write_register(rd, value);
    }

    pub fn xor(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs] ^ self.R[rt];
        self.write_register(rd, value);
    }

    pub fn nor(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = !(self.R[rs] | self.R[rt]);
        self.write_register(rd, value);
    }

    pub fn andi(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm();

        let value = self.R[rs] & imm;

        self.write_register(rt, value);
    }

    pub fn ori(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm();

        let value = self.R[rs] | imm;

        self.write_register(rt, value);
    }
}