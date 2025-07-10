pub mod coord;
pub mod general;
pub mod color;

pub trait GTE_Command {
    fn sf(&self) -> bool;
    fn mx(&self) -> Self;
    fn v(&self) -> Self;
    fn cv(&self) -> Self;
    fn lm(&self) -> bool;
    fn num(&self) -> Self;
}

impl GTE_Command for u32 {
    #[inline(always)]
    fn sf(&self) -> bool {
        (*self >> 19) & 1 != 0
    }

    #[inline(always)]
    fn mx(&self) -> Self {
        (*self >> 17) & 3
    }

    #[inline(always)]
    fn v(&self) -> Self {
        (*self >> 15) & 3
    }

    #[inline(always)]
    fn cv(&self) -> Self {
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