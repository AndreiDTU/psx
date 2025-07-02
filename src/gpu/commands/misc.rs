use crate::{bus::interrupt::IRQ, gpu::{primitives::vertex::Vertex, GP0_State, GPU}};

impl GPU {
    pub fn quick_fill(&mut self, word: u32) -> GP0_State {
        let top_left = Vertex::from(self.gp0_parameters.pop_front().unwrap());
        let size = self.gp0_parameters.pop_front().unwrap();

        let height = (size >> 16) & 0x1FF;
        let width = size & 0x3FF;

        for x in top_left.coords.x..(top_left.coords.x + width as i32) {
            for y in top_left.coords.y..(top_left.coords.y + height as i32) {
                let coords: u32 = Vertex::from((x, y)).into();
                self.draw_pixel(word, coords);
            }
        }
        
        GP0_State::CommandStart
    }

    pub fn irq(&mut self) -> GP0_State {
        let old_irq = self.gpu_status.interrupt_request() != 0;
        if !old_irq {
            self.gpu_status.set_interrupt_request(1);
            self.interrupt.borrow_mut().request(IRQ::GPU);
        }

        GP0_State::CommandStart
    }
}