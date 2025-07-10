use glam::{I16Vec3, I64Vec3, IVec3};

use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
    pub fn mvmva(&mut self, command: u32) -> usize {
        let mx = self.mx(command.mx()).map(|row| row.as_i64vec3());
        let v = self.v(command.v()).as_i64vec3();
        let tx = self.tx(command.cv()).as_i64vec3();

        let raw_mac = (tx << 12) + I64Vec3::from_array(mx.map(|row| {(row * v).element_sum()}));
        let saturated_mac = self.update_mac_vector_flags(raw_mac, command.sf());
        self.write_mac_vector(saturated_mac);

        let ir = self.update_ir_flags(saturated_mac, command.lm());
        self.write_ir_vector(ir);

        8
    }

    pub fn sqr(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let ir = self.ir_vector().as_i64vec3();
        let squared_vector = ir * ir;

        let saturated_mac = self.update_mac_vector_flags(squared_vector, sf);
        self.write_mac_vector(saturated_mac);
        
        let saturated_ir = self.update_ir_flags(saturated_mac, true);
        self.write_ir_vector(saturated_ir);

        5
    }

    pub fn op(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();
        
        let ir = self.ir_vector().as_i64vec3();
        let d = self.d_vector().as_i64vec3();

        let cross_product = d.cross(ir);

        let saturated_mac = self.update_mac_vector_flags(cross_product, sf);
        self.write_mac_vector(saturated_mac);
        
        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        6
    }

    fn mx(&self, mx: u32) -> [I16Vec3; 3] {
        match mx {
            0 => self.rt(),
            1 => self.llm(),
            2 => self.lcm(),
            3 => {
                // Garbage matrix
                let row1 = I16Vec3::from_array([-0x60, 0x60, self.ir0()]);
                let row2 = I16Vec3::splat(self.R[41] as i16);
                let row3 = I16Vec3::splat(self.R[42] as i16);

                [row1, row2, row3]
            }
            _ => unreachable!(),
        }
    }

    fn v(&self, v: u32) -> I16Vec3 {
        match v {
            0 | 1 | 2 => self.vector(v),
            3 => self.ir_vector(),
            _ => unreachable!(),
        }
    }

    fn tx(&self, cv: u32) -> IVec3 {
        match cv {
            0 => self.tr(),
            1 => self.bk(),
            2 | 3 => IVec3::ZERO,
            _ => unreachable!(),
        }
    }
}