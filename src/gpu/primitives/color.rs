use crate::gpu::primitives::vertex::Vertex;

const COLOR_DEPTH_LUT: [u8; 32] = [0, 8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99, 107, 115, 123, 132, 140, 148, 156, 165, 173, 181, 189, 197, 206, 214, 222, 230, 239, 247, 255];
const DITHER_TABLE: [[i8; 4]; 4] = [
    [-4,  0, -3,  1],
    [ 2, -2,  3, -1],
    [-3,  1, -4,  0],
    [ 3, -1,  2, -2],
];

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<u16> for Color {
    #[inline]
    fn from(pixel: u16) -> Self {
        Self {
            r: COLOR_DEPTH_LUT[(pixel & 0x1F) as usize],
            g: COLOR_DEPTH_LUT[((pixel >> 5) & 0x1F) as usize],
            b: COLOR_DEPTH_LUT[((pixel >> 10) & 0x1F) as usize],
            a: 255,
        }
    }
}

impl From<Color> for u32 {
    #[inline(always)]
    fn from(color: Color) -> Self {
        (color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16)
    }
}

impl Color {
    #[inline]
    pub fn apply_dithering(&mut self, p: Vertex) -> Self {
        let offset = DITHER_TABLE[(p.y & 3) as usize][(p.x & 3) as usize];

        self.r = self.r.saturating_add_signed(offset);
        self.g = self.g.saturating_add_signed(offset);
        self.b = self.b.saturating_add_signed(offset);

        *self
    }

    #[inline]
    pub fn compress_color_depth(color_24bit: u32) -> u16 {
        let r = (color_24bit & 0xFF) >> 3;
        let g = ((color_24bit >> 8) & 0xFF) >> 3;
        let b = ((color_24bit >> 16) & 0xFF) >> 3;

        (r | (g << 5) | (b << 10)) as u16
    }

    #[inline]
    pub fn interpolate_color(lambda: [f64; 3], colors: [Color; 3]) -> Color {
        let colors_r = colors.map(|color| f64::from(color.r));
        let colors_g = colors.map(|color| f64::from(color.g));
        let colors_b = colors.map(|color| f64::from(color.b));

        let r = (lambda[0] * colors_r[0] + lambda[1] * colors_r[1] + lambda[2] * colors_r[2]).round() as u8;
        let g = (lambda[0] * colors_g[0] + lambda[1] * colors_g[1] + lambda[2] * colors_g[2]).round() as u8;
        let b = (lambda[0] * colors_b[0] + lambda[1] * colors_b[1] + lambda[2] * colors_b[2]).round() as u8;

        Color { r, g, b, a: 255 }
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