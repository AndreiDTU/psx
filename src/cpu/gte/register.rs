use glam::{I16Vec2, I16Vec3, I64Vec3, IVec3, U8Vec4, UVec3, Vec3Swizzles};

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

    pub fn write_otz(&mut self, value: u16) {
        self.R[7] = value as u32;
    }

    pub fn update_otz_flags(&mut self, value: i32) -> u16 {
        if value > 0xFFFF {
            self.R[63] |= 1 << 18;
            0xFFFF
        } else if value < 0 {
            self.R[63] |= 1 << 18;
            0
        } else {
            value as u16
        }
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
            let neg_overflow = vector.zyx().cmplt(IVec3::splat(i16::MIN as i32));
            let pos_overflow = vector.zyx().cmpgt(IVec3::splat(i16::MAX as i32));

            let mut truncated = vector.as_i16vec3();
            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[63] |= (pos_overflow | neg_overflow).bitmask() << 22;
            truncated = I16Vec3 {
                x: if pos_overflow.z {i16::MAX} else if neg_overflow.z {i16::MIN} else {truncated.x},
                y: if pos_overflow.y {i16::MAX} else if neg_overflow.y {i16::MIN} else {truncated.y},
                z: if pos_overflow.x {i16::MAX} else if neg_overflow.x {i16::MIN} else {truncated.z},
            };

            truncated
        } else {
            let neg_overflow = vector.zyx().cmplt(IVec3::splat(0));
            let pos_overflow = vector.zyx().cmpgt(IVec3::splat(i16::MAX as i32));

            let mut truncated = vector.as_i16vec3();
            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[63] |= (pos_overflow | neg_overflow).bitmask() << 22;
            truncated = I16Vec3 {
                x: if pos_overflow.z {i16::MAX} else if neg_overflow.z {0} else {truncated.x},
                y: if pos_overflow.y {i16::MAX} else if neg_overflow.y {0} else {truncated.y},
                z: if pos_overflow.x {i16::MAX} else if neg_overflow.x {0} else {truncated.z},
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

    pub fn write_mac0(&mut self, value: i32) {
        self.R[24] = value as u32;
    }

    pub fn update_mac0_flags(&mut self, value: i64) -> i32 {
        if value > i32::MAX as i64 {
            self.R[63] |= 1 << 16;
        } else if value < i32::MIN as i64 {
            self.R[63] |= 1 << 15;
        }

        value as i32
    }

    pub fn mac_vector(&self) -> IVec3 {
        IVec3 { x: self.R[25] as i32, y: self.R[26] as i32, z: self.R[27] as i32 }
    }

    pub fn write_mac_vector(&mut self, vector: IVec3) {
        [self.R[25], self.R[26], self.R[27]] = vector.as_uvec3().to_array();
    }

    pub fn update_mac_vector_flags(&mut self, new_mac: I64Vec3, sf: bool) -> IVec3 {
        if sf {
            let new_mac: I64Vec3 = new_mac >> 12;
            let neg_overflow = new_mac.zyx().cmplt(I64Vec3::splat(i32::MIN as i64));
            let pos_overflow = new_mac.zyx().cmpgt(I64Vec3::splat(i32::MAX as i64));

            let mut truncated = new_mac.as_ivec3();

            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[63] |= pos_overflow.bitmask() << 28;
            truncated = IVec3 {
                x: if pos_overflow.z {i32::MAX} else {truncated.x},
                y: if pos_overflow.y {i32::MAX} else {truncated.y},
                z: if pos_overflow.x {i32::MAX} else {truncated.z},
            };

            self.R[63] |= neg_overflow.bitmask() << 25;
            truncated = IVec3 {
                x: if neg_overflow.z {i32::MIN} else {truncated.x},
                y: if neg_overflow.y {i32::MIN} else {truncated.y},
                z: if neg_overflow.x {i32::MIN} else {truncated.z},
            };

            truncated
        } else {
            const I44_MAX: i64 = 0x0000_07FF_FFFF_FFFF as i64;
            const I44_MIN: i64 = -0x0000_0800_0000_0000 as i64;
            let neg_overflow = new_mac.zyx().cmplt(I64Vec3::splat(I44_MIN));
            let pos_overflow = new_mac.zyx().cmpgt(I64Vec3::splat(I44_MAX));

            let mut truncated = new_mac.as_ivec3();

            if !neg_overflow.any() && !pos_overflow.any() {return truncated}

            self.R[63] |= pos_overflow.bitmask() << 28;
            truncated = IVec3 {
                x: if pos_overflow.z {I44_MAX as i32} else {truncated.x},
                y: if pos_overflow.y {I44_MAX as i32} else {truncated.y},
                z: if pos_overflow.x {I44_MAX as i32} else {truncated.z},
            };

            self.R[63] |= neg_overflow.bitmask() << 25;
            truncated = IVec3 {
                x: if neg_overflow.z {I44_MIN as i32} else {truncated.x},
                y: if neg_overflow.y {I44_MIN as i32} else {truncated.y},
                z: if neg_overflow.x {I44_MIN as i32} else {truncated.z},
            };

            truncated
        }
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

    pub fn zsf3(&self) -> i16 {
        self.R[61] as i16
    }

    pub fn zsf4(&self) -> i16 {
        self.R[62] as i16
    }
}