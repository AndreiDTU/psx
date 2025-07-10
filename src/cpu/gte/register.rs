use glam::{I16Vec2, I16Vec3, I64Vec3, IVec2, IVec3, U8Vec4, UVec3, Vec3Swizzles};

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

    pub fn write_ir0(&mut self, ir0: i16) {
        self.R[8] = ir0 as u32;
    }

    pub fn update_ir0_flags(&mut self, raw_ir0: i64) -> i16 {
        const IR0_MAX: i64 = 0x1000;

        let overflow = raw_ir0 < 0 || raw_ir0 > IR0_MAX;
        self.R[63] |= (overflow as u32) << 12;

        raw_ir0.clamp(0, IR0_MAX) as i16
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

    pub fn update_ir_flags_rtp(&mut self, vector: IVec3, lm: bool, raw_mac3: i32) -> I16Vec3 {
        let bugged_overflow = raw_mac3 < i16::MIN as i32 || raw_mac3 > i16::MAX as i32;
        self.R[63] |= (bugged_overflow as u32) << 24;

        if !lm {
            let neg_overflow = vector.yx().cmplt(I16Vec2::MIN.as_ivec2());
            let pos_overflow = vector.yx().cmpgt(I16Vec2::MAX.as_ivec2());

            self.R[63] |= (pos_overflow | neg_overflow).bitmask() << 22;
            vector.clamp(I16Vec3::MIN.as_ivec3(), I16Vec3::MAX.as_ivec3()).as_i16vec3()
        } else {
            let neg_overflow = vector.yx().cmplt(IVec2::ZERO);
            let pos_overflow = vector.yx().cmpgt(I16Vec2::MAX.as_ivec2());

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

    pub fn push_sxy_fifo(&mut self, sx2: i16, sy2: i16) {
        self.R[12] = self.R[13];
        self.R[13] = self.R[14];
        self.R[14] = (sx2 as u16 as u32) | ((sy2 as u16 as u32) << 16);
    }

    pub fn update_sxy2_flags(&mut self, raw_sx2: i64, raw_sy2: i64) -> [i16; 2] {
        const SXY2_MIN: i64 = -0x400;
        const SXY2_MAX: i64 = 0x3FF;

        let sx2_overflow = raw_sx2 < SXY2_MIN || raw_sx2 > SXY2_MAX;
        let sy2_overflow = raw_sy2 < SXY2_MIN || raw_sy2 > SXY2_MAX;

        self.R[63] |= (sx2_overflow as u32) << 14;
        self.R[63] |= (sy2_overflow as u32) << 13;

        [
            raw_sx2.clamp(SXY2_MIN, SXY2_MAX) as i16,
            raw_sy2.clamp(SXY2_MIN, SXY2_MAX) as i16,
        ]
    }

    pub fn screen_z(&self, idx: u32) -> u16 {
        self.R[idx | 16] as u16
    }

    pub fn push_screen_z_fifo(&mut self, value: u16) {
        self.R[16] = self.R[17];
        self.R[17] = self.R[18];
        self.R[18] = self.R[19];
        self.R[19] = value as u32;
    }

    pub fn update_sz3_flags(&mut self, raw_sz3: i64) -> u16 {
        let overflow = raw_sz3 > u16::MAX as i64 || raw_sz3 < 0;
        self.R[63] |= (overflow as u32) << 18;
        raw_sz3.clamp(0, u16::MAX as i64) as u16
    }

    pub fn pop_color_fifo(&mut self) -> U8Vec4 {
        U8Vec4::from_array(self.R[20].to_le_bytes())
    }

    pub fn push_color_fifo(&mut self) {
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

            raw_mac.as_ivec3()
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
        let row1 = I16Vec3 {
            x: self.R[32] as i16,
            y: (self.R[32] >> 16) as i16,
            z: self.R[33] as i16
        };
        let row2 = I16Vec3 {
            x: (self.R[33] >> 16) as i16,
            y: self.R[34] as i16,
            z: (self.R[34] >> 16) as i16
        };
        let row3 = I16Vec3 {
            x: self.R[35] as i16,
            y: (self.R[35] >> 16) as i16,
            z: self.R[36] as i16
        };

        [row1, row2, row3]
    }

    pub fn d_vector(&self) -> I16Vec3 {
        I16Vec3 { x: self.R[32] as i16, y: self.R[34] as i16, z: self.R[36] as i16 }
    }

    pub fn tr(&self) -> IVec3 {
        (UVec3 { x: self.R[37], y: self.R[38], z: self.R[39] }).as_ivec3()
    }

    pub fn llm(&self) -> [I16Vec3; 3] {
        let row1 = I16Vec3 {
            x: self.R[40] as i16,
            y: (self.R[40] >> 16) as i16,
            z: self.R[41] as i16
        };
        let row2 = I16Vec3 {
            x: (self.R[41] >> 16) as i16,
            y: self.R[42] as i16,
            z: (self.R[42] >> 16) as i16
        };
        let row3 = I16Vec3 {
            x: self.R[43] as i16,
            y: (self.R[43] >> 16) as i16,
            z: self.R[44] as i16
        };

        [row1, row2, row3]
    }

    pub fn bk(&self) -> IVec3 {
        UVec3::from_array([self.R[45], self.R[46], self.R[47]]).as_ivec3()
    }

    pub fn lcm(&self) -> [I16Vec3; 3] {
        let row1 = I16Vec3 {
            x: self.R[48] as i16,
            y: (self.R[48] >> 16) as i16,
            z: self.R[49] as i16
        };
        let row2 = I16Vec3 {
            x: (self.R[49] >> 16) as i16,
            y: self.R[50] as i16,
            z: (self.R[50] >> 16) as i16
        };
        let row3 = I16Vec3 {
            x: self.R[51] as i16,
            y: (self.R[51] >> 16) as i16,
            z: self.R[52] as i16
        };

        [row1, row2, row3]
    }

    pub fn fc(&self) -> IVec3 {
        IVec3 {
            x: self.R[53] as i32,
            y: self.R[54] as i32,
            z: self.R[55] as i32,
        }
    }

    pub fn screen_offset(&self) -> [i32; 2] {
        [self.R[56] as i32, self.R[57] as i32]
    }

    pub fn h(&self) -> u16 {
        self.R[58] as u16
    }

    pub fn dqa(&self) -> i16 {
        self.R[59] as i16
    }

    pub fn dqb(&self) -> i32 {
        self.R[60] as i32
    }

    pub fn zsf3(&self) -> i16 {
        self.R[61] as i16
    }

    pub fn zsf4(&self) -> i16 {
        self.R[62] as i16
    }
}