use std::{fs::File, io::Read, path::Path};

use anyhow::anyhow;

const BIOS_SIZE: u64 = 512 * 1024;

pub struct BIOS {
    data: Vec<u8>,
}

impl BIOS {
    pub fn new(path: &Path) -> Result<BIOS, anyhow::Error> {
        let file = File::open(path)?;
        let mut data = Vec::new();

        file.take(BIOS_SIZE).read_to_end(&mut data)?;
        
        if data.len() != BIOS_SIZE as usize {
            Err(anyhow!("Invalid BIOS"))
        } else {
            Ok(BIOS { data })
        }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        u32::from_le_bytes(*self.data[(offset as usize)..].first_chunk().unwrap())
    }
}