use vulkano::buffer::BufferContents;

#[repr(C)]
#[derive(Clone, Copy, Debug, BufferContents)]
pub struct Vertex {
    pub coords: [i16; 2],
    pub color: [u8; 4],
    pub texpage: u16,
    pub clut: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BufferContents)]
pub struct Tri {
    pub vertices: [Vertex; 3]
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BufferContents)]
pub struct TriTask {
    pub min: [i32; 2],
    pub max: [i32; 2],
    pub v0: [i32; 2],
    pub v1: [i32; 2],
    pub v2: [i32; 2],
    pub c0: [u32; 3],
    pub c1: [u32; 3],
    pub c2: [u32; 3],
}