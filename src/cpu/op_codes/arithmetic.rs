use crate::cpu::{decoder::{Cause, Instruction}, CPU};

impl CPU {
    pub fn div(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();

        let dividend = self.R[rs] as i32;
        let divisor = self.R[rt] as i32;

        if divisor == 0 {return}
        if dividend == i32::MIN && divisor == -1 {
            (self.lo, self.hi) = (i32::MIN as u32, 0);
            return;
        }

        let (quotient, remainder) = (dividend / divisor, dividend % divisor);
        (self.lo, self.hi) = (quotient as u32, remainder as u32);
    }

    pub fn mult(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();

        let a = self.R[rs] as i32 as i64;
        let b = self.R[rt] as i32 as i64;

        let value = (a * b) as u64;

        self.lo = value as u32;
        self.hi = (value >> 32) as u32;
    }

    pub fn multu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();

        let a = self.R[rs] as u64;
        let b = self.R[rt] as u64;

        let value = a * b;

        self.lo = value as u32;
        self.hi = (value >> 32) as u32;
    }

    pub fn divu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();

        let dividend = self.R[rs];
        let divisor = self.R[rt];

        if divisor == 0 {return}

        (self.lo, self.hi) = (dividend / divisor, dividend % divisor);
    }

    pub fn add(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let a = self.R[rs] as i32;
        let b = self.R[rt] as i32;
        
        if let Some(value) = a.checked_add(b) {
            self.write_register(rd, value as u32);
        } else {
            self.raise_exception(Cause::Ovf);
        }
    }

    pub fn addu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs].wrapping_add(self.R[rt]);
        self.write_register(rd, value);
    }

    pub fn sub(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let a = self.R[rs] as i32;
        let b = self.R[rt] as i32;

        if let Some(value) = a.checked_sub(b) {
            self.write_register(rd, value as u32);
        } else {
            self.raise_exception(Cause::Ovf);
        }
    }

    pub fn subu(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let rd = instruction.rd();

        let value = self.R[rs].wrapping_sub(self.R[rt]);

        self.write_register(rd, value);
    }
    
    pub fn addi(&mut self, instruction: u32) {
        let rs = instruction.rs();
        let rt = instruction.rt();
        let imm = instruction.imm_se() as i32;

        if let Some(value) = (self.R[rs] as i32).checked_add(imm) {
            self.write_register(rt, value as u32);
        } else {
            self.raise_exception(Cause::Ovf);
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