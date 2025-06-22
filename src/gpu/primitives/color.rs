const COLOR_DEPTH_LUT: [u8; 32] = [0, 8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99, 107, 115, 123, 132, 140, 148, 156, 165, 173, 181, 189, 197, 206, 214, 222, 230, 239, 247, 255];

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<u16> for Color {
    fn from(pixel: u16) -> Self {
        Self {
            r: COLOR_DEPTH_LUT[(pixel & 0x1F) as usize],
            g: COLOR_DEPTH_LUT[((pixel >> 5) & 0x1F) as usize],
            b: COLOR_DEPTH_LUT[((pixel >> 10) & 0x1F) as usize],
            a: 255,
        }
    }
}

impl From<u32> for Color {
    fn from(color_word: u32) -> Self {
        let r = ((color_word & 0xFF) >> 3) as u8;
        let g = (((color_word >> 8) & 0xFF) >> 3) as u8;
        let b = (((color_word >> 16) & 0xFF) >> 3) as u8;
        
        Self { r, g, b, a: 255 }
    }
}

impl From<Color> for u16 {
    fn from(color: Color) -> Self {
        let (r, g, b) = (color.r as u16, color.g as u16, color.b as u16);
        r | (g << 5) | (b << 10)
    }
}

#[cfg(test)]
mod test {
    use crate::gpu::primitives::color::{Color, COLOR_DEPTH_LUT};

    #[test]
    fn halfword_to_color() {
        let halfword: u16 = 0x55D0;
        let color = Color::from(halfword);

        let reference_color = Color {
            r: COLOR_DEPTH_LUT[0b10000],
            g: COLOR_DEPTH_LUT[0b01110],
            b: COLOR_DEPTH_LUT[0b10101],
            a: 255,
        };

        assert_eq!(color, reference_color);
    }
}