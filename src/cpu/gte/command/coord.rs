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
}