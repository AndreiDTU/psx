use glam::{I16Vec2, I16Vec3, I64Vec3, IVec3, U8Vec4, UVec3, Vec3Swizzles};

use crate::cpu::gte::GTE;

pub const I44_MAX: I64Vec3 = I64Vec3::splat(0x0000_07FF_FFFF_FFFF);
pub const I44_MIN: I64Vec3 = I64Vec3::splat(-0x0000_0800_0000_0000);

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

    pub fn update_otz_flags(&mut self, value: i64) -> u16 {
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
            let neg_overflow = vector.zyx().cmplt(I16Vec3::MIN.as_ivec3());
            let pos_overflow = vector.zyx().cmpgt(I16Vec3::MAX.as_ivec3());

            self.R[63] |= (pos_overflow | neg_overflow).bitmask() << 22;
            vector.clamp(I16Vec3::MIN.as_ivec3(), I16Vec3::MAX.as_ivec3()).as_i16vec3()
        } else {
            let neg_overflow = vector.zyx().cmplt(IVec3::ZERO);
            let pos_overflow = vector.zyx().cmpgt(I16Vec3::MAX.as_ivec3());

            self.R[63] |= (pos_overflow | neg_overflow).bitmask() << 22;
            vector.clamp(IVec3::ZERO, I16Vec3::MAX.as_ivec3()).as_i16vec3()
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

    pub fn write_color_fifo(&mut self) {
        const COLOR_MIN: IVec3 = IVec3::ZERO;
        const COLOR_MAX: IVec3 = IVec3::splat(0xFF);

        self.R[20] = self.R[21];
        self.R[21] = self.R[22];

        let code = (self.R[6] >> 24) as u8;
        let raw_rgb: IVec3 = self.mac_vector() >> 4;

        let neg_overflow = raw_rgb.zyx().cmplt(COLOR_MIN);
        let pos_overflow = raw_rgb.zyx().cmpgt(COLOR_MAX);
        self.R[63] |= (neg_overflow.bitmask() | pos_overflow.bitmask()) << 19;

        let rgb = raw_rgb.clamp(COLOR_MIN, COLOR_MAX).as_u8vec3().extend(code).to_array();
        self.R[22] = u32::from_le_bytes(rgb);
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

    pub fn update_mac_vector_flags(&mut self, raw_mac: I64Vec3, sf: bool) -> IVec3 {
        if sf {
            let new_mac: I64Vec3 = raw_mac >> 12;
            let neg_overflow = new_mac.zyx().cmplt(IVec3::MIN.as_i64vec3());
            let pos_overflow = new_mac.zyx().cmpgt(IVec3::MAX.as_i64vec3());

            self.R[63] |= pos_overflow.bitmask() << 28;
            self.R[63] |= neg_overflow.bitmask() << 25;

            new_mac.as_ivec3()
        } else {            
            let neg_overflow = raw_mac.zyx().cmplt(I44_MIN);
            let pos_overflow = raw_mac.zyx().cmpgt(I44_MAX);

            self.R[63] |= pos_overflow.bitmask() << 28;
            self.R[63] |= neg_overflow.bitmask() << 25;

            raw_mac.clamp(I44_MIN, I44_MAX).as_ivec3()
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
        let row3 = I16Vec3 { x: self.R[35] as i16,         y: (self.R[35] >> 16) as i16, z: self.R[36] as i16        };

        [row1, row2, row3]
    }

    pub fn d_vector(&self) -> I16Vec3 {
        I16Vec3 { x: self.R[32] as i16, y: self.R[34] as i16, z: self.R[36] as i16 }
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