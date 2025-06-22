use std::cmp;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    x: i32,
    y: i32,
}

impl Vertex {
    fn cross_product_z(v0: Vertex, v1: Vertex, v2: Vertex) -> i32 {
        (v1.x - v0.x) * (v2.y - v0.y) - (v1.y - v0.y) * (v2.x - v0.x)
    }

    pub fn ensure_vertex_order(v0: &mut Vertex, v1: &mut Vertex, v2: Vertex) {
        let cross_product_z = Vertex::cross_product_z(*v0, *v1, v2);
        if cross_product_z < 0 {
            std::mem::swap(v0, v1);
        }
    }

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

    pub fn is_inside_triangle(p: Vertex, v0: Vertex, v1: Vertex, v2: Vertex) -> bool {
        for (va, vb) in [(v0, v1), (v1, v2), (v2, v0)] {
            if Vertex::cross_product_z(va, vb, p) < 0 {
                return false;
            }
        }

        true
    }
}