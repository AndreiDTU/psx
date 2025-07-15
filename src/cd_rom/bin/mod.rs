use std::{collections::HashMap, fmt::Debug, path::Path};

use crate::cd_rom::{bin::sector::Sector};

pub mod sector;

pub type DiskMap = HashMap<DiskAddress, Sector>;

pub trait DiskTrait {
    fn from_bin<P>(bin_path: P) -> anyhow::Result<DiskMap>
    where P: AsRef<Path>;
}

impl DiskTrait for DiskMap {
    fn from_bin<P>(bin_path: P) -> anyhow::Result<DiskMap>
    where P: AsRef<Path> {
        let disk = std::fs::read(bin_path)?;
        let chunks = disk.chunks_exact(2352);
        let mut sectors: HashMap<DiskAddress, Sector> = HashMap::new();

        for chunk in chunks {
            let (address, sector) = Sector::from_bytes(chunk);
            sectors.insert(address, sector);
        }

        Ok(sectors)
    }
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DiskAddress {
    min: u8,
    sec: u8,
    frame: u8,
}

impl DiskAddress {
    pub fn from_bytes(bytes: &[u8]) -> DiskAddress {
        Self { min: bytes[0], sec: bytes[1], frame: bytes[2] }
    }

    pub fn increment(&mut self) {
        fn carry_lo(x: &mut u8) {
            let carry = (((*x & 0x0F) + 6) & 0x10) >> 4;
            *x += carry * 6;
        }

        self.frame += 1;
        carry_lo(&mut self.frame);

        let second_carry = (self.frame == 0x75) as u8;
        self.frame -= 0x75 * second_carry;
        self.sec += second_carry;
        carry_lo(&mut self.sec);
        
        let minute_carry = (self.sec == 0x60) as u8;
        self.sec -= 0x60 * minute_carry;
        self.min += minute_carry;
        carry_lo(&mut self.min);
    }
}

impl Debug for DiskAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskAddress")
            .field("min", &format!("{:02X}",  self.min))
            .field("sec", &format!("{:02X}",  self.sec))
            .field("frame", &format!("{:02X}",  self.frame))
            .finish()
    }
}