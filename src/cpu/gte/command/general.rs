use crate::cpu::gte::{command::GTE_Command, GTE};

impl GTE {
    pub fn sqr(&mut self, command: u32) -> usize {
        let sf = command.sf();
        let ir = self.ir_vector().as_i64vec3();
        let squared_vector = ir * ir;

        let saturated_mac = self.update_mac_vector_flags(squared_vector, sf != 0);
        self.write_mac_vector(saturated_mac);
        
        let saturated_ir = self.update_ir_flags(saturated_mac, true);
        self.write_ir_vector(saturated_ir);

        5
    }
}