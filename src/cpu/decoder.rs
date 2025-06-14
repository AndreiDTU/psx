pub trait Instruction {
    fn op(&self) -> Self;
    fn rs(&self) -> Self;
    fn rt(&self) -> Self;
    fn imm(&self) -> Self;
    fn target(&self) -> Self;
    fn rd(&self) -> Self;
    fn shamt(&self) -> Self;
    fn funct(&self) -> Self;
    fn imm_se(&self) -> Self;
}

impl Instruction for u32 {
    fn op(&self) -> Self {
        (*self >> 26) & 0x3F
    }
    
    fn rs(&self) -> Self {
        (*self >> 21) & 0x1F
    }
    
    fn rt(&self) -> Self {
        (*self >> 16) & 0x1F
    }
    
    fn imm(&self) -> Self {
        (*self >> 0) & 0xFFFF
    }
    
    fn target(&self) -> Self {
        (*self >> 0) & 0x03FF_FFFF
    }

    fn rd(&self) -> Self {
        (*self >> 11) & 0x1F
    }
    
    fn shamt(&self) -> Self {
        (*self >> 6) & 0x1F
    }
    
    fn funct(&self) -> Self {
        (*self >> 0) & 0x3F
    }

    fn imm_se(&self) -> Self {
        *self as i16 as u32
    }
}

pub enum Cause {
    INT = 0x00,
    AdEL = 0x04,
    AdES = 0x05,
    IBE = 0x06,
    DBE = 0x07,
    Sys = 0x08,
    Bp = 0x09,
    RI = 0x0A,
    CpU = 0x0B,
    Ovf = 0x0C,
}