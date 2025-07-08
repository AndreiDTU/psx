pub mod coord;
pub mod general;
pub mod color;

pub trait GTE_Command {
    fn sf(&self) -> bool;
    fn mul_matrix(&self) -> Self;
    fn mul_vector(&self) -> Self;
    fn trans_vector(&self) -> Self;
    fn lm(&self) -> bool;
    fn num(&self) -> Self;
}

impl GTE_Command for u32 {
    #[inline(always)]
    fn sf(&self) -> bool {
        (*self >> 19) & 1 != 0
    }

    #[inline(always)]
    fn mul_matrix(&self) -> Self {
        (*self >> 17) & 3
    }

    #[inline(always)]
    fn mul_vector(&self) -> Self {
        (*self >> 15) & 3
    }

    #[inline(always)]
    fn trans_vector(&self) -> Self {
        (*self >> 13) & 3
    }

    #[inline(always)]
    fn lm(&self) -> bool {
        (*self >> 10) & 1 != 0
    }

    #[inline(always)]
    fn num(&self) -> Self {
        (*self >> 0) & 0x3F
    }
}