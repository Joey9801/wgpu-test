use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub texcoord: [f32; 2],

    /// RGBA color
    pub color: [f32; 4],
}

impl Vertex {
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
                    format: wgpu::VertexFormat::Float3,
                    offset: 3 * 4,
                    shader_location: 1,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 6 * 4,
                    shader_location: 2,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: 8 * 4,
                    shader_location: 3,
                },
            ],
        }
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}
