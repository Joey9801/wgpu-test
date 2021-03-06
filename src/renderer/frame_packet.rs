use super::{AtlasId, ModelId};

#[derive(Clone, Copy)]
pub struct InstanceData {
    /// Transforms positions from model space to world space
    pub model_matrix: cgmath::Matrix4<f32>,

    /// Transforms normals from model space to view space
    pub normal_matrix: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for InstanceData {}
unsafe impl bytemuck::Zeroable for InstanceData {}

impl InstanceData {
    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        const FLOAT_SIZE: wgpu::BufferAddress = 4;
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: 0,
                    shader_location: 4,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4,
                    shader_location: 5,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 2,
                    shader_location: 6,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 3,
                    shader_location: 7,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 4,
                    shader_location: 8,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 5,
                    shader_location: 9,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 6,
                    shader_location: 10,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 7,
                    shader_location: 11,
                },
            ],
        }
    }
}

pub struct FramePacketModel {
    pub model_id: ModelId,
    pub instances: Vec<InstanceData>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SpriteInstanceData {
    /// The clip space x/y coordinate of the top-left corner of this sprite
    pub screen_pos: cgmath::Vector2<f32>,

    /// The clip space size of the this sprite
    pub screen_size: cgmath::Vector2<f32>,

    /// The atlas x/y coordinate of the top-left corner of this sprite
    pub atlas_pos: cgmath::Vector2<f32>,

    /// The size of the sprite in the atlas
    pub atlas_size: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for SpriteInstanceData {}
unsafe impl bytemuck::Zeroable for SpriteInstanceData {}

impl SpriteInstanceData {
    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 2 * 4,
                    shader_location: 1,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 4,
                    shader_location: 2,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 6 * 4,
                    shader_location: 3,
                },
            ],
        }
    }
}


pub struct FramePacketSprites {
    pub atlas_id: AtlasId,
    pub sprites: Vec<SpriteInstanceData>,
}

/// Desribes a frame for the renderer to draw in its entirity
pub struct FramePacket {
    pub view: cgmath::Matrix4<f32>,
    pub proj: cgmath::Matrix4<f32>,
    pub models: Vec<FramePacketModel>,
    pub overlay_sprites: Vec<FramePacketSprites>,
}