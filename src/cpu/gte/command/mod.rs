pub mod coord;
pub mod general;

pub trait GTE_Command {
    fn sf(&self) -> i16;
    fn mul_matrix(&self) -> Self;
    fn mul_vector(&self) -> Self;
    fn trans_vector(&self) -> Self;
    fn saturate(&self) -> Self;
    fn num(&self) -> Self;
}

impl GTE_Command for u32 {
    #[inline(always)]
    fn sf(&self) -> i16 {
        ((*self >> 19) & 1) as i16
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
    fn saturate(&self) -> Self {
        (*self >> 10) & 1
    }

    #[inline(always)]
    fn num(&self) -> Self {
        (*self >> 0) & 0x3F
    }
}