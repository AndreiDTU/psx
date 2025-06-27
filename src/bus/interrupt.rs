#[derive(Default)]
pub struct Interrupt {
    I_STAT: u32,
    I_MASK: u32,
}

impl Interrupt {
    pub fn read_status32(&self) -> u32 {
        self.I_STAT
    }

    pub fn read_mask32(&self) -> u32 {
        self.I_MASK & 0x3FF
    }

    pub fn acknowledge32(&mut self, value: u32) {
        self.I_STAT &= value;
    }

    pub fn write_mask32(&mut self, value: u32) {
        self.I_MASK = value;
    }

    pub fn read_status16(&self) -> u16 {
        self.I_STAT as u16
    }

    pub fn read_mask16(&self) -> u16 {
        self.I_MASK as u16 & 0x3FF
    }

    pub fn acknowledge16(&mut self, value: u16) {
        self.I_STAT &= value as u32;
    }

    pub fn write_mask16(&mut self, value: u16) {
        self.I_MASK = value as u32;
    }

    pub fn request(&mut self, irq: IRQ) {
        self.I_STAT |= self.I_MASK & irq as u32;
    }
}

pub enum IRQ {
    VBLANK        = 0x001,
    GPU           = 0x002,
    CDROM         = 0x004,
    DMA           = 0x008,
    TMR0          = 0x010,
    TMR1          = 0x020,
    TMR2          = 0x040,
    BYTE_RECEIVED = 0x080,
    SIO           = 0x100,
    SPU           = 0x200,
    LIGHTPEN      = 0x800,
}