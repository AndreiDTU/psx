use glam::{I64Vec3, IVec3};

use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
    pub fn rtps(&mut self, command: u32) -> usize {
        self.perspective_transformation(1, command.sf(), command.lm());

        15
    }
    pub fn rtpt(&mut self, command: u32) -> usize {
        self.perspective_transformation(3, command.sf(), command.lm());

        23
    }

    fn perspective_transformation(&mut self, vectors: u32, sf: bool, lm: bool) {
        let tr = self.tr().as_i64vec3();
        let rt = self.rt().map(|row| row.as_i64vec3());
        let [ofx, ofy] = self.screen_offset();

        for idx in 0..vectors {
            let v = self.vector(idx).as_i64vec3();

            let row_product = I64Vec3::from_array(rt.map(|row| row.wrapping_mul(v).element_sum()));
            let raw_mac = (tr << 12) + row_product;
            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags_rtp(saturated_mac, lm, (raw_mac.z >> 12) as i32);
            self.write_ir_vector(saturated_ir);

            let ir = self.ir_vector().as_i64vec3();

            let raw_sz3 = raw_mac.z >> ((1 - sf as i64) * 12);
            let saturated_sz3 = self.update_sz3_flags(raw_sz3);
            self.push_screen_z_fifo(saturated_sz3);

            let unr_dividend = self.unr_divide();

            let raw_mac0 = unr_dividend as i64 * ir.x + ofx as i64;
            let saturated_mac0 = self.update_mac0_flags(raw_mac0);
            self.write_mac0(saturated_mac0);
            let raw_sx2 = raw_mac0 >> 16;

            let raw_mac0 = unr_dividend as i64 * ir.y + ofy as i64;
            let saturated_mac0 = self.update_mac0_flags(raw_mac0);
            self.write_mac0(saturated_mac0);
            let raw_sy2 = raw_mac0 >> 16;

            let [sx2, sy2] = self.update_sxy2_flags(raw_sx2, raw_sy2);
            self.push_sxy_fifo(sx2, sy2);

            let raw_mac0 = unr_dividend as i64 * self.dqa() as i64 + self.dqb() as i64;
            let saturated_mac0 = self.update_mac0_flags(raw_mac0);
            self.write_mac0(saturated_mac0);
            
            let raw_ir0 = raw_mac0 >> 12;
            let saturated_ir0 = self.update_ir0_flags(raw_ir0);
            self.write_ir0(saturated_ir0);
        }
    }

    pub fn nclip(&mut self, _command: u32) -> usize {
        let (s0, s1, s2) = (self.screen_xy(0).as_ivec2(), self.screen_xy(1).as_ivec2(), self.screen_xy(2).as_ivec2());

        let y_factors = IVec3 {
            x: s1.y - s2.y,
            y: s2.y - s0.y,
            z: s0.y - s1.y,
        };

        let sx = IVec3::from_array([s0.x, s1.x, s2.x]);

        let saturated_mac0 = self.update_mac0_flags((sx.as_i64vec3() * y_factors.as_i64vec3()).element_sum());     
        self.write_mac0(saturated_mac0);

        8
    }

    pub fn avsz3(&mut self, _command: u32) -> usize {
        let sz1 = self.screen_z(1) as u32;
        let sz2 = self.screen_z(2) as u32;
        let sz3 = self.screen_z(3) as u32;

        let zsf3 = self.zsf3() as i32;

        let raw_mac0 = zsf3 as i64 * (sz1 + sz2 + sz3) as i32 as i64;
        let saturated_mac0 = self.update_mac0_flags(raw_mac0);
        self.write_mac0(saturated_mac0);

        let saturated_otz = self.update_otz_flags(raw_mac0 >> 12);
        self.write_otz(saturated_otz);

        5
    }

    pub fn avsz4(&mut self, _command: u32) -> usize {
        let sz0 = self.screen_z(0) as u32;
        let sz1 = self.screen_z(1) as u32;
        let sz2 = self.screen_z(2) as u32;
        let sz3 = self.screen_z(3) as u32;

        let zsf4 = self.zsf4() as i32;

        let raw_mac0 = zsf4 as i64 * (sz0 + sz1 + sz2 + sz3) as i32 as i64;
        let saturated_mac0 = self.update_mac0_flags(raw_mac0);
        self.write_mac0(saturated_mac0);

        let saturated_otz = self.update_otz_flags(raw_mac0 >> 12);
        self.write_otz(saturated_otz);

        5
    }

    fn unr_divide(&mut self) -> u32 {
        let h = self.h() as u32;
        let sz3 = self.screen_z(3);
        if h < (sz3 as u32) << 1 {
            let z = sz3.leading_zeros();
            let n = h << z;
            let mut d = ((sz3 as u32) << z) as u64;
            let u = self.UNR_TABLE[((d - 0x7FC0) >> 7) as usize] as u64 + 0x101;
            d = (0x0200_0080 - ((d as u64) * u)) >> 8;
            d = (0x0000_0080 + ((d as u64) * u)) >> 8;
            (((n as u64 * d as u64) + 0x8000) >> 16).clamp(0, 0x1FFFF) as u32
        } else {
            self.R[63] |= const {1 << 17};
            0x1FFFF
        }
    }
}