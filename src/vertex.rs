use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],

    /// RGBA color
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: [f32; 3]) -> Self {
        Vertex {
            position,
            color: [0.4, 0.4, 0.4, 1.0],
        }
    }

    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: 3 * 4,
                    shader_location: 1,
                },
            ],
        }
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}
