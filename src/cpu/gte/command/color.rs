use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
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

        self.write_color_fifo();

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

        self.write_color_fifo();

        5
    }
}