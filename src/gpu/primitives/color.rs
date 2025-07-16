use glam::{DMat3, DVec3, I8Vec3, U8Vec3};

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
    pub rgb: U8Vec3,
}

impl From<u16> for Color {
    #[inline]
    fn from(pixel: u16) -> Self {
        let r = COLOR_DEPTH_LUT[(pixel & 0x1F) as usize];
        let g = COLOR_DEPTH_LUT[((pixel >> 5) & 0x1F) as usize];
        let b = COLOR_DEPTH_LUT[((pixel >> 10) & 0x1F) as usize];
        Self {
            rgb: U8Vec3::from_array([r, g, b]),
        }
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        let rgb_word = value.to_le_bytes();
        Color { rgb: U8Vec3 { x: rgb_word[2], y: rgb_word[1], z: rgb_word[0] } }
    }
}

impl From<Color> for u32 {
    #[inline(always)]
    fn from(color: Color) -> Self {
        let [r, g, b] = color.rgb.to_array();
        (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
    }
}

impl Color {
    #[inline(always)]
    pub fn modulate(&self, vertex_color: Color) -> Color {
        Color {rgb: ((self.rgb.as_dvec3() * vertex_color.rgb.as_dvec3()) / DVec3::splat(128.0)).round().as_u8vec3()}
    }

    #[inline]
    pub fn apply_dithering(&mut self, p: Vertex) -> Self {
        let [px, py] = (p.coords & 3).to_array();
        let offset = DITHER_TABLE[(py & 3) as usize][(px & 3) as usize];

        self.rgb = self.rgb.saturating_add_signed(I8Vec3::splat(offset));

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
    pub fn interpolate_color(lambda: DVec3, colors: [Color; 3]) -> Color {
        let colors_rgb = DMat3::from_cols(colors[0].rgb.as_dvec3(), colors[1].rgb.as_dvec3(), colors[2].rgb.as_dvec3());

        let rgb = (colors_rgb * lambda).round().as_u8vec3();

        Color { rgb }
    }

    #[inline]
    pub fn blend(&self, back: Color, mode: u8) -> Color {
        match mode {
            0 => Color { rgb: (self.rgb / 2) + (back.rgb / 2) },
            1 => Color { rgb: self.rgb.saturating_add(back.rgb) },
            2 => Color { rgb: back.rgb.saturating_sub(self.rgb) },
            3 => Color { rgb: back.rgb.saturating_add(self.rgb / 4) },
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod test {
    use glam::U8Vec3;

    use crate::gpu::primitives::color::{Color, COLOR_DEPTH_LUT};

    #[test]
    fn halfword_to_color() {
        let halfword: u16 = 0x55D0;
        let color = Color::from(halfword);

        let r = COLOR_DEPTH_LUT[0b10000];
        let g = COLOR_DEPTH_LUT[0b01110];
        let b = COLOR_DEPTH_LUT[0b10101];

        let reference_color = Color { rgb: U8Vec3::from_array([r, g, b])};

        assert_eq!(color, reference_color);
    }
}