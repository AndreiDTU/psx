use glam::{DVec3, IVec2, IVec4, Vec4Swizzles};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub coords: IVec2,
}

impl From<(i32, i32)> for Vertex {
    fn from(value: (i32, i32)) -> Self {
        Self { coords: IVec2::from(value) }
    }
}

impl From<(u32, u32)> for Vertex {
    fn from(value: (u32, u32)) -> Self {
        let (x, y) = value;
        let value = (x as i32, y as i32);
        Self { coords: IVec2::from(value) }
    }
}

impl From<u32> for Vertex {
    #[inline]
    fn from(word: u32) -> Self {
        let mut x = word & 0x7FF;
        let mut y = (word >> 16) & 0x7FF;

        if x & (1 << 10) != 0 {x |= 0xFFFF_F800}
        if y & (1 << 10) != 0 {y |= 0xFFFF_F800}

        Self { coords: IVec2::from_array([x as i32, y as i32]) }
    }
}

impl From<Vertex> for u32 {
    #[inline]
    fn from(value: Vertex) -> Self {
        let [x, y] = value.coords.to_array();

        let x = (x as u32) & 0x7FF;
        let y = ((y as u32) & 0x7FF) << 16;

        y | x
    }
}

impl Vertex {
    #[inline]
    pub fn translate(&self, translation: Vertex) -> Vertex {
        Vertex { coords: self.coords.wrapping_add(translation.coords) }
    }

    #[inline]
    pub fn is_inside_triangle(&self, v0: Vertex, v1: Vertex, v2: Vertex) -> bool {
        for (va, vb) in [(v0, v1), (v1, v2), (v2, v0)] {
            let z = Vertex::cross_product_z(va, vb, *self);
            if z < 0 {
                return false;
            }

            if z == 0 {
                if vb.coords.y > va.coords.y {
                    return false;
                }

                if vb.coords.y == va.coords.y && vb.coords.x < va.coords.x {
                    return false;
                }
            }
        }

        true
    }

    #[inline]
    pub fn compute_barycentric_coordinates(&self, v0: Vertex, v1: Vertex, v2: Vertex) -> DVec3 {
        let denominator = Vertex::cross_product_z(v0, v1, v2);
        if denominator == 0 {
            return DVec3::from_array([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
        }

        let denominator: f64 = denominator.into();

        let lambda0 = f64::from(Vertex::cross_product_z(v1, v2, *self)) / denominator;
        let lambda1 = f64::from(Vertex::cross_product_z(v2, v0, *self)) / denominator;

        let lambda2 = 1.0 - lambda0 - lambda1;

        DVec3::from_array([lambda0, lambda1, lambda2])
    }

    #[inline(always)]
    fn cross_product_z(v0: Vertex, v1: Vertex, v2: Vertex) -> i32 {
        let a = v1.coords - v0.coords;
        let b = v2.coords - v0.coords;

        a.x * b.y - a.y * b.x
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
    pub fn triangle_bounding_box(v0: Vertex, v1: Vertex, v2: Vertex, drawing_area_top_left: Vertex, drawing_area_bottom_right: Vertex) -> IVec4 {
        let min = v0.coords.min(v1.coords.min(v2.coords)).max(drawing_area_top_left.coords);
        let max = v0.coords.max(v1.coords.max(v2.coords)).min(drawing_area_bottom_right.coords);

        if (min.cmpgt(max)).any() {
            return IVec4::ZERO;
        }

        return IVec4::from((min, max)).xzyw();
    }
}