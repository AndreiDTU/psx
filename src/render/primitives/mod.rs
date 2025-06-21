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