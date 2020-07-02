use std::collections::HashMap;

use crate::shader_cache::ShaderCache;
use super::{frame_packet::{FramePacket, SpriteInstanceData}, Renderer, AtlasId, GpuAtlas};

pub struct SpriteOverlayRenderStage {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<AtlasId, wgpu::BindGroup>,
    texture_sampler: wgpu::Sampler,
}

impl SpriteOverlayRenderStage {
    pub async fn new(device: &wgpu::Device) -> Self {
        let mut shader_cache = ShaderCache::new();
        let vs_spirv = shader_cache
            .get_shader(
                "./src/renderer/shaders/sprite.vert",
                shaderc::ShaderKind::Vertex,
            )
            .await;
        let fs_spirv = shader_cache
            .get_shader(
                "./src/renderer/shaders/sprite.frag",
                shaderc::ShaderKind::Fragment,
            )
            .await;

        let vs_module = device.create_shader_module(&vs_spirv);
        let fs_module = device.create_shader_module(&fs_spirv);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: Some("UI render stage bind group layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[
                    SpriteInstanceData::vertex_buffer_descriptor(),
                ],
            },
            sample_count: 1,
            sample_mask: 0,
            alpha_to_coverage_enabled: false,
        });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Self {
            pipeline,
            texture_sampler,
            texture_bind_group_layout,
            texture_bind_groups: HashMap::new(),
        }
    }

    pub fn add_atlas(&mut self, device: &wgpu::Device, atlas_id: AtlasId, atlas: &GpuAtlas) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                }
            ],
            label: Some("Texture atlas bind group"),
        });

        self.texture_bind_groups.insert(atlas_id, bind_group);
    }

    pub fn draw_frame(
        &self,
        renderer: &Renderer,
        frame_packet: &FramePacket,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
    ) {
        for sprite_set in &frame_packet.overlay_sprites {
            let bind_group = self
                .texture_bind_groups
                .get(&sprite_set.atlas_id)
                .expect("Frame packet references sprite atlas with unknown id");

            let instance_data_buff = renderer.device.create_buffer_with_data(
                bytemuck::cast_slice(&sprite_set.sprites[..]),
                wgpu::BufferUsage::VERTEX,
            );

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &output,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLUE,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_vertex_buffer(0, &instance_data_buff, 0, 0);
            rpass.draw(
                0..4,
                0..(sprite_set.sprites.len() as u32)
            );
        }
    }
}