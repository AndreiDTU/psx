pub struct SPU {
    voice: [u8; 0x180],
    control: [u8; 0x40],
    reverb: [u8; 0x40],
}

impl SPU {
    pub fn new() -> Self {
        Self {
            voice: [0; 0x180],
            control: [0; 0x40],
            reverb: [0; 0x40],
        }
    }

    pub fn read_voice32(&mut self, addr: u32) -> u32 {
        u32::from_le_bytes(*self.voice[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_control32(&mut self, addr: u32) -> u32 {
        u32::from_le_bytes(*self.control[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_reverb32(&mut self, addr: u32) -> u32 {
        u32::from_le_bytes(*self.reverb[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_voice16(&mut self, addr: u32) -> u16 {
        u16::from_le_bytes(*self.voice[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_control16(&mut self, addr: u32) -> u16 {
        u16::from_le_bytes(*self.control[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_reverb16(&mut self, addr: u32) -> u16 {
        u16::from_le_bytes(*self.reverb[(addr as usize)..].first_chunk_mut().unwrap())
    }

    pub fn read_voice8(&mut self, addr: u32) -> u8 {
        self.voice[addr as usize]
    }

    pub fn read_control8(&mut self, addr: u32) -> u8 {
        self.control[addr as usize]
    }

    pub fn read_reverb8(&mut self, addr: u32) -> u8 {
        self.reverb[addr as usize]
    }

    pub fn write_voice32(&mut self, addr: u32, value: u32) {
        *self.voice[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_control32(&mut self, addr: u32, value: u32) {
        *self.control[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_reverb32(&mut self, addr: u32, value: u32) {
        *self.reverb[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_voice16(&mut self, addr: u32, value: u16) {
        *self.voice[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_control16(&mut self, addr: u32, value: u16) {
        *self.control[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_reverb16(&mut self, addr: u32, value: u16) {
        *self.reverb[(addr as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }

    pub fn write_voice8(&mut self, addr: u32, value: u8) {
        self.control[addr as usize] = value;
    }

    pub fn write_control8(&mut self, addr: u32, value: u8) {
        self.control[addr as usize] = value;
    }

    pub fn write_reverb8(&mut self, addr: u32, value: u8) {
        self.control[addr as usize] = value;
    }
}