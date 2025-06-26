pub mod color;
pub mod vertex;

pub fn interpolate_uv_coords(lambda: [f64; 3], uv: [(u32, u32); 3]) -> (u32, u32) {
    let uv = uv.map(|(x, y)| {(f64::from(x), f64::from(y))});

    let u = (lambda[0] * uv[0].0 + lambda[1] * uv[1].0 + lambda[2] * uv[2].0).round() as u32;
    let v = (lambda[0] * uv[0].1 + lambda[1] * uv[1].1 + lambda[2] * uv[2].1).round() as u32;

    (u, v)
}