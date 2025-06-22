use std::cmp;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

impl From<u32> for Vertex {
    #[inline]
    fn from(word: u32) -> Self {
        let mut x = word & 0x7FF;
        let mut y = (word >> 16) & 0x7FF;

        if x & (1 << 10) != 0 {x |= 0xFFFF_F800}
        if y & (1 << 10) != 0 {y |= 0xFFFF_F800}

        Self { x: x as i32, y: y as i32 }
    }
}

impl From<Vertex> for u32 {
    #[inline]
    fn from(value: Vertex) -> Self {
        let x = (value.x as u32) & 0x7FF;
        let y = ((value.y as u32) & 0x7FF) << 16;

        y | x
    }
}

impl Vertex {
    #[inline]
    pub fn translate(&self, translation: Vertex) -> Vertex {
        let x = self.x.wrapping_add(translation.x);
        let y = self.y.wrapping_add(translation.y);

        Vertex { x, y }
    }

    #[inline]
    pub fn is_inside_triangle(&self, v0: Vertex, v1: Vertex, v2: Vertex) -> bool {
        for (va, vb) in [(v0, v1), (v1, v2), (v2, v0)] {
            let z = Vertex::cross_product_z(va, vb, *self);
            if z < 0 {
                return false;
            }

            if z == 0 {
                if vb.y > va.y {
                    return false;
                }

                if vb.y == va.y && vb.x < va.x {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    pub fn compute_barycentric_coordinates(&self, v0: Vertex, v1: Vertex, v2: Vertex) -> [f64; 3] {
        let denominator = Vertex::cross_product_z(v0, v1, v2);
        if denominator == 0 {
            return [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        }

        let denominator: f64 = denominator.into();

        let lambda0 = f64::from(Vertex::cross_product_z(v1, v2, *self)) / denominator;
        let lambda1 = f64::from(Vertex::cross_product_z(v2, v0, *self)) / denominator;

        let lambda2 = 1.0 - lambda0 - lambda1;

        [lambda0, lambda1, lambda2]
    }

    #[inline(always)]
    fn cross_product_z(v0: Vertex, v1: Vertex, v2: Vertex) -> i32 {
        (v1.x - v0.x) * (v2.y - v0.y) - (v1.y - v0.y) * (v2.x - v0.x)
    }

    #[inline]
    pub fn ensure_vertex_order(v0: &mut Vertex, v1: &mut Vertex, v2: Vertex) -> bool {
        let cross_product_z = Vertex::cross_product_z(*v0, *v1, v2);
        if cross_product_z < 0 {
            std::mem::swap(v0, v1);
            return true;
        }

        false
    }

    #[inline]
    pub fn triangle_bounding_box(v0: Vertex, v1: Vertex, v2: Vertex, drawing_area_top_left: Vertex, drawing_area_bottom_right: Vertex) -> (i32, i32, i32, i32) {
        let mut min_x = cmp::min(v0.x, cmp::min(v1.x, v2.x));
        let mut max_x = cmp::max(v0.x, cmp::max(v1.x, v2.x));
        let mut min_y = cmp::min(v0.y, cmp::min(v1.y, v2.y));
        let mut max_y = cmp::max(v0.y, cmp::max(v1.y, v2.y));

        min_x = cmp::max(min_x, drawing_area_top_left.x);
        max_x = cmp::min(max_x, drawing_area_bottom_right.x);
        min_y = cmp::max(min_y, drawing_area_top_left.y);
        max_y = cmp::min(max_y, drawing_area_bottom_right.y);

        if min_x > max_x || min_y > max_y {
            return (0, 0, 0, 0);
        }

        return (min_x, max_x, min_y, max_y);
    }
}