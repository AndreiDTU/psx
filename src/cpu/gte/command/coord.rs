use glam::IVec3;

use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
    pub fn rtps(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let tr = self.tr() * 0x1000;
        let rt = self.rt();
        let vector = self.vector(0).as_ivec3();

        let rotated_vector = IVec3 {
            x: (rt[0].as_ivec3() * vector).element_sum(),
            y: (rt[1].as_ivec3() * vector).element_sum(),
            z: (rt[2].as_ivec3() * vector).element_sum(),
        };

        let transformed_vector = (tr + rotated_vector) >> (sf * 12);
        self.write_mac_vector(transformed_vector);
        self.write_ir_vector(transformed_vector.as_i16vec3());

        let screen_z = self.mac_vector().z >> ((1-sf)*12);
        self.write_screen_z(3, screen_z as u16);

        15
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

        let saturated_mac0 = self.update_mac0_flags(zsf3 as i64 * (sz1 + sz2 + sz3) as i32 as i64);
        self.write_mac0(saturated_mac0);

        let saturated_otz = self.update_otz_flags(self.mac0() / 0x1000);
        self.write_otz(saturated_otz);

        5
    }
}