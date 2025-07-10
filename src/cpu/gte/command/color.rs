use glam::{I64Vec3, Vec4Swizzles};

use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
    pub fn ncs(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color(sf, lm, 1);

        14
    }

    pub fn nct(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color(sf, lm, 3);

        30
    }

    pub fn nccs(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color_color(sf, lm, 1);

        17
    }

    pub fn ncct(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color_color(sf, lm, 3);

        39
    }

    pub fn ncds(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color_depth_cue(sf, lm, 1);

        19
    }

    pub fn ncdt(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        self.normal_color_depth_cue(sf, lm, 3);

        44
    }

    pub fn cc(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        let raw_mac = (self.bk().as_i64vec3() << 12) + I64Vec3::from_array(self.lcm().map(|row| {
            (row.as_i64vec3() * self.ir_vector().as_i64vec3()).element_sum()
        }));

        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);

        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        let raw_mac: I64Vec3 = ((self.rgbc().xyz().as_i64vec3() * self.ir_vector().as_i64vec3()).as_u64vec3() << 4_u64).as_i64vec3();
        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);

        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        self.push_color_fifo();

        11
    }

    pub fn cdp(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        let raw_mac = (self.bk().as_i64vec3() << 12) + I64Vec3::from_array(self.lcm().map(|row| {
            (row.as_i64vec3() * self.ir_vector().as_i64vec3()).element_sum()
        }));

        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);

        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        let rgb_ir = ((self.rgbc().xyz().as_ivec3() * self.ir_vector().as_ivec3()).as_uvec3() << 4_u32).as_ivec3();
        self.apply_ir0_fc(sf, lm, rgb_ir.as_i64vec3());

        13
    }

    pub fn dcpl(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        let rgb_ir = ((self.rgbc().xyz().as_ivec3() * self.ir_vector().as_ivec3()).as_uvec3() << 4_u32).as_ivec3();
        self.apply_ir0_fc(sf, lm, rgb_ir.as_i64vec3());

        8
    }

    pub fn dpcs(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        let rgb = self.rgbc().xyz().as_u64vec3() << 16_u64;
        self.apply_ir0_fc(sf, lm, rgb.as_i64vec3());
        
        8
    }

    pub fn dpct(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        for _ in 0..3 {
            let rgb = self.pop_color_fifo().xyz().as_u64vec3() << 16_u64;
            self.apply_ir0_fc(sf, lm, rgb.as_i64vec3());
        }

        17
    }

    pub fn intpl(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();

        let ir = self.ir_vector().as_i64vec3().as_u64vec3() << 12_u64;
        self.apply_ir0_fc(sf, lm, ir.as_i64vec3());
        

        8
    }

    fn normal_color(&mut self, sf: bool, lm: bool, colors: u32) {
        for idx in 0..colors {
            let raw_mac = I64Vec3::from_array(self.llm().map(|row| {
                (row.as_i64vec3() * self.vector(idx).as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            let raw_mac = (self.bk().as_i64vec3() << 12) + I64Vec3::from_array(self.lcm().map(|row| {
                (row.as_i64vec3() * self.ir_vector().as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            self.push_color_fifo();
        }
    }

    fn normal_color_color(&mut self, sf: bool, lm: bool, colors: u32) {
        for idx in 0..colors {
            let raw_mac = I64Vec3::from_array(self.llm().map(|row| {
                (row.as_i64vec3() * self.vector(idx).as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            let raw_mac = (self.bk().as_i64vec3() << 12) + I64Vec3::from_array(self.lcm().map(|row| {
                (row.as_i64vec3() * self.ir_vector().as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            let raw_mac: I64Vec3 = ((self.rgbc().xyz().as_i64vec3() * self.ir_vector().as_i64vec3()).as_u64vec3() << 4_u64).as_i64vec3();
            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            self.push_color_fifo();
        }
    }

    fn normal_color_depth_cue(&mut self, sf: bool, lm: bool, colors: u32) {
        for idx in 0..colors {
            let raw_mac = I64Vec3::from_array(self.llm().map(|row| {
                (row.as_i64vec3() * self.vector(idx).as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            let raw_mac = (self.bk().as_i64vec3() << 12) + I64Vec3::from_array(self.lcm().map(|row| {
                (row.as_i64vec3() * self.ir_vector().as_i64vec3()).element_sum()
            }));

            let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
            self.write_mac_vector(saturated_mac);

            let saturated_ir = self.update_ir_flags(saturated_mac, lm);
            self.write_ir_vector(saturated_ir);

            let rgb_ir = ((self.rgbc().xyz().as_ivec3() * self.ir_vector().as_ivec3()).as_uvec3() << 4_u32).as_ivec3();
            self.apply_ir0_fc(sf, lm, rgb_ir.as_i64vec3());
        }
    }

    fn apply_ir0_fc(&mut self, sf: bool, lm: bool, in_mac: I64Vec3) {
        let rgb_fc = (self.fc().as_i64vec3().as_u64vec3() << 12_u64).as_i64vec3();
        let raw_ir = (rgb_fc - in_mac) >> ((sf as i64) * 12);
        let saturated_ir = self.update_ir_flags(raw_ir.as_ivec3(), false);
        let raw_mac = saturated_ir.as_i64vec3() * self.ir0() as i64 + in_mac;

        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);

        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        self.push_color_fifo();
    }

    pub fn gpf(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();
        let ir = self.ir_vector().as_i64vec3();
        let ir0 = self.ir0() as i64;

        let raw_mac = ir0 * ir;
        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);
        
        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        self.push_color_fifo();

        5
    }

    pub fn gpl(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let lm = command.lm();
        let ir = self.ir_vector().as_i64vec3();
        let ir0 = self.ir0() as i64;

        let raw_mac = ir0 * ir + (self.mac_vector().as_u64vec3() << (sf as u64 * 12)).as_i64vec3();
        let saturated_mac = self.update_mac_vector_flags(raw_mac, sf);
        self.write_mac_vector(saturated_mac);
        
        let saturated_ir = self.update_ir_flags(saturated_mac, lm);
        self.write_ir_vector(saturated_ir);

        self.push_color_fifo();

        5
    }
}