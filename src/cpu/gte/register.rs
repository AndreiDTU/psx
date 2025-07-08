use glam::{I16Vec2, I16Vec3, I64Vec3, IVec3, U8Vec4, UVec3};

use crate::cpu::gte::GTE;

impl GTE {
    pub fn vector(&self, idx: u32) -> I16Vec3 {
        let register_idx = idx << 1;
        let x = self.R[register_idx] as i16;
        let y = (self.R[register_idx] >> 16) as i16;
        let z = self.R[register_idx | 1] as i16;

        I16Vec3 { x, y, z }
    }

    pub fn rgbc(&self) -> U8Vec4 {
        U8Vec4::from_array(self.R[6].to_le_bytes())
    }

    pub fn otz(&self) -> u16 {
        self.R[7] as u16
    }

    pub fn ir0(&self) -> i16 {
        self.R[8] as u16 as i16
    }

    pub fn ir_vector(&self) -> I16Vec3 {
        let ir1 = self.R[9] as u16 as i16;
        let ir2 = self.R[10] as u16 as i16;
        let ir3 = self.R[11] as u16 as i16;

        I16Vec3::from_array([ir1, ir2, ir3])
    }

    pub fn write_ir_vector(&mut self, vector: I16Vec3) {
        [self.R[9], self.R[10], self.R[11]] = vector.as_u16vec3().as_uvec3().to_array();
        self.update_irgb_red(self.R[9]);
        self.update_irgb_green(self.R[10]);
        self.update_irgb_blue(self.R[11]);
    }

    pub fn update_ir_flags(&mut self, vector: IVec3, lm: bool) -> I16Vec3 {
        if !lm {
            let neg_overflow = vector.cmplt(IVec3::splat(-0x8000));
            let pos_overflow = vector.cmpgt(IVec3::splat(0x7FFF));

            let mut truncated = vector.as_i16vec3();
            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[31] |= (pos_overflow | neg_overflow).bitmask() << 22;
            truncated = I16Vec3 {
                x: if pos_overflow.x {0x7FFF} else if neg_overflow.x {-0x8000} else {truncated.x},
                y: if pos_overflow.y {0x7FFF} else if neg_overflow.y {-0x8000} else {truncated.y},
                z: if pos_overflow.z {0x7FFF} else if neg_overflow.z {-0x8000} else {truncated.z},
            };

            truncated
        } else {
            let neg_overflow = vector.cmplt(IVec3::splat(0));
            let pos_overflow = vector.cmpgt(IVec3::splat(0x7FFF));

            let mut truncated = vector.as_i16vec3();
            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[31] |= (pos_overflow | neg_overflow).bitmask() << 22;
            truncated = I16Vec3 {
                x: if pos_overflow.x {0x7FFF} else if neg_overflow.x {0} else {truncated.x},
                y: if pos_overflow.y {0x7FFF} else if neg_overflow.y {0} else {truncated.y},
                z: if pos_overflow.z {0x7FFF} else if neg_overflow.z {0} else {truncated.z},
            };

            truncated
        }
    }

    pub fn screen_xy(&self, idx: u32) -> I16Vec2 {
        let register_idx = idx | 12;
        let x = self.R[register_idx] as i16;
        let y = (self.R[register_idx] >> 16) as i16;

        I16Vec2 { x, y }
    }

    pub fn screen_z(&self, idx: u32) -> u16 {
        self.R[idx | 16] as u16
    }

    pub fn write_screen_z(&mut self, idx: u32, value: u16) {
        self.R[idx | 16] = value as u32;
    }

    pub fn rgb(&self, idx: u32) -> U8Vec4 {
        U8Vec4::from_array(self.R[idx | 20].to_le_bytes())
    }

    pub fn mac0(&self) -> i32 {
        self.R[24] as i32
    }

    pub fn mac_vector(&self) -> IVec3 {
        IVec3 { x: self.R[25] as i32, y: self.R[26] as i32, z: self.R[27] as i32 }
    }

    pub fn write_mac_vector(&mut self, vector: IVec3) {
        [self.R[25], self.R[26], self.R[27]] = vector.as_uvec3().to_array();
    }

    pub fn update_mac_flags(&mut self, new_mac: I64Vec3) -> IVec3 {
        let neg_overflow = new_mac.cmplt(I64Vec3::splat(-0x8000_0000));
        let pos_overflow = new_mac.cmpgt(I64Vec3::splat(0x7FFF_FFFF));

        let mut truncated = new_mac.as_ivec3();

        if !neg_overflow.any() && !pos_overflow.any() {return truncated}

        self.R[31] |= pos_overflow.bitmask() << 28;
        truncated = IVec3 {
            x: if pos_overflow.x {0x7FFF_FFFF} else {truncated.x},
            y: if pos_overflow.y {0x7FFF_FFFF} else {truncated.y},
            z: if pos_overflow.z {0x7FFF_FFFF} else {truncated.z},
        };

        self.R[31] |= neg_overflow.bitmask() << 25;
        truncated = IVec3 {
            x: if neg_overflow.x {-0x8000_0000} else {truncated.x},
            y: if neg_overflow.y {-0x8000_0000} else {truncated.y},
            z: if neg_overflow.z {-0x8000_0000} else {truncated.z},
        };

        truncated
    }

    pub fn lzcs(&self) -> i32 {
        self.R[30] as i32
    }

    pub fn lzcr(&self) -> u32 {
        let lzcs = self.lzcs();
        lzcs.leading_ones() | lzcs.leading_zeros()
    }

    pub fn rt(&self) -> [I16Vec3; 3] {
        let row1 = I16Vec3 { x: self.R[32] as i16,         y: (self.R[32] >> 16) as i16, z: self.R[33] as i16        };
        let row2 = I16Vec3 { x: (self.R[33] >> 16) as i16, y: self.R[34] as i16,         z: (self.R[34] >> 16) as i16};
        let row3 = I16Vec3 { x: self.R[35] as i16,         y: (self.R[34] >> 16) as i16, z: self.R[35] as i16        };

        [row1, row2, row3]
    }

    pub fn tr(&self) -> IVec3 {
        (UVec3 { x: self.R[37], y: self.R[38], z: self.R[39] }).as_ivec3()
    }

    pub fn h(&self) -> u16 {
        self.R[58] as u16
    }
}