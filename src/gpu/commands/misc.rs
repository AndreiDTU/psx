use crate::{bus::interrupt::IRQ, gpu::{GP0_State, GPU}};

impl GPU {
    pub fn irq(&mut self) -> GP0_State {
        let old_irq = self.gpu_status.interrupt_request() != 0;
        if !old_irq {
            self.gpu_status.set_interrupt_request(1);
            self.interrupt.borrow_mut().request(IRQ::GPU);
        }

        GP0_State::CommandStart
    }
}