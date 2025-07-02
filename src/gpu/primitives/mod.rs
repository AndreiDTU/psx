use glam::DVec3;

pub mod color;
pub mod vertex;

#[inline]
pub fn interpolate_uv_coords(lambda: DVec3, uv: [(u32, u32); 3]) -> (u32, u32) {
    let uv = uv.map(|(u, v)| {(f64::from(u), f64::from(v))});
    let u_vector = DVec3::from_array([uv[0].0, uv[1].0, uv[2].0]);
    let v_vector = DVec3::from_array([uv[0].1, uv[1].1, uv[2].1]);

    let u = (lambda * u_vector).element_sum().round() as u32;
    let v = (lambda * v_vector).element_sum().round() as u32;

    (u, v)
}