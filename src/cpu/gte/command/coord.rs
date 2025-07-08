use glam::IVec3;

use crate::cpu::gte::GTE;

impl GTE {
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
}