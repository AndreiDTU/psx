use std::ops::Index;

use bitflags::bitflags;

use crate::cd_rom::bin::DiskAddress;

#[derive(Clone, Copy, Debug)]
pub struct Sector {
    sub_header: SubHeader,
    data: [u8; 0x924],
}

impl Sector {
    pub fn from_bytes(bytes: &[u8]) -> (DiskAddress, Sector) {
        let mut data = [0; 0x924];
        (
            DiskAddress::from_bytes(&bytes[12..=15]),
            Sector {
                sub_header: SubHeader::from_bytes(&bytes[16..=23]),
                data: *bytes[12..].first_chunk().unwrap_or({
                    for i in 12..bytes.len() {
                        data[i - 12] = bytes[i];
                    }

                    &data
                })
            }
        )
    }

    pub fn get_sub_header(&self) -> SubHeader {
        self.sub_header
    }
}

impl Index<usize> for Sector {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct SubHeader {
    file_num: u8,
    channel_num: u8,
    sub_mode: SubMode,
    coding_info: CodingInfo,
}

impl SubHeader {
    pub fn from_bytes(bytes: &[u8]) -> SubHeader {
        Self {
            file_num: bytes[0],
            channel_num: bytes[1],
            sub_mode: SubMode::from_bits_truncate(bytes[2]),
            coding_info: CodingInfo::from_bits_truncate(bytes[3]),
        }
    }

    pub fn get_sub_mode(&self) -> SubMode {
        self.sub_mode
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct SubMode: u8 {
        const EOF       = 0x80;
        const REAL_TIME = 0x40;
        const FORM      = 0x20;
        const TRIGGER   = 0x10;
        const DATA      = 0x08;
        const AUDIO     = 0x04;
        const VIDEO     = 0x02;
        const EOR       = 0x01;
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct CodingInfo: u8 {
        const emphasis = 0x40;
        const bitssamp = 0x10;
        const samprate = 0x04;
        const stereo   = 0x01;
    }
}