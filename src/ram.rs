pub struct RAM {
    data: Vec<u8>,
}

impl RAM {
    pub fn new(size: usize) -> RAM {
        Self { data: vec![0; size] }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        u32::from_le_bytes(*self.data[(offset as usize)..].first_chunk().unwrap())
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        *self.data[(offset as usize)..].first_chunk_mut().unwrap() = value.to_le_bytes();
    }
}