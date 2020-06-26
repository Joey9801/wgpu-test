use std::collections::HashMap;

use crate::{model_data::ModelData, shader_cache::ShaderCache, vertex::Vertex};

pub mod frame_packet;
use frame_packet::{FramePacket, InstanceData};

/// Represents a handle to a single model's data on the GPU
struct GpuModel {
    vertex_buff: wgpu::Buffer,
    index_buff: wgpu::Buffer,
    index_count: u32,
}

impl GpuModel {
    fn from_data(data: &ModelData, device: &wgpu::Device) -> Self {
        let vertex_buff = device.create_buffer_with_data(
            bytemuck::cast_slice(&data.vertices),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buff = device.create_buffer_with_data(
            bytemuck::cast_slice(&data.indices),
            wgpu::BufferUsage::INDEX,
        );
        let index_count = data.indices.len() as u32;

        Self {
            vertex_buff,
            index_buff,
            index_count,
        }
    }
}

/// Exposed as a handle to a GpuModel
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelId(usize);

#[allow(unused)]
pub struct Renderer {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    depth_texture: wgpu::Texture,

    next_model_id: ModelId,
    models: HashMap<ModelId, GpuModel>,

    render_stage: RenderStage,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::VULKAN,
        )
        .await
        .expect("Failed to create adapter that can draw to our window");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: true,
                    ..wgpu::Extensions::default()
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main depth texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
        });

        let render_stage = RenderStage::new(&device).await;

        Self {
            size,
            surface,
            adapter,
            device,
            queue,
            swapchain,
            depth_texture,
            next_model_id: ModelId(0),
            models: HashMap::new(),
            render_stage,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn upload_model(&mut self, data: ModelData) -> ModelId {
        let new_gpu_model = GpuModel::from_data(&data, &self.device);
        let new_model_id = self.next_model_id;

        self.models.insert(new_model_id, new_gpu_model);
        self.next_model_id = ModelId(self.next_model_id.0 + 1);

        new_model_id
    }

    pub fn draw_frame(&mut self, frame_packet: &FramePacket) {
        let frame = match self.swapchain.get_next_texture() {
            Ok(frame) => frame,
            Err(e) => panic!("Failed to get next swapchain frame: {:?}", e),
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Per frame encoder"),
            });

        self.render_stage.draw_frame(
            self,
            frame_packet,
            &mut encoder,
            &frame.view,
            &self.depth_texture.create_default_view(),
        );

        self.queue.submit(&[encoder.finish()]);
    }
}

#[derive(Clone, Copy)]
#[allow(unused)]
struct UniformData {
    view: cgmath::Matrix4<f32>,
    proj: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for UniformData {}
unsafe impl bytemuck::Zeroable for UniformData {}

struct RenderStage {
    uniform_bind_group: wgpu::BindGroup,
    uniform_buff: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl RenderStage {
    pub async fn new(device: &wgpu::Device) -> Self {
        let mut shader_cache = ShaderCache::new();
        let vs_spirv = shader_cache
            .get_shader(
                "./src/renderer/shaders/shader.vert",
                shaderc::ShaderKind::Vertex,
            )
            .await;
        let fs_spirv = shader_cache
            .get_shader(
                "./src/renderer/shaders/shader.frag",
                shaderc::ShaderKind::Fragment,
            )
            .await;

        let vs_module = device.create_shader_module(&vs_spirv);
        let fs_module = device.create_shader_module(&fs_spirv);

        let uniform_buff = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<UniformData>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            label: Some("Render stage uniform buffer"),
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("Render stage uniform buffer layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buff,
                    range: 0..std::mem::size_of::<UniformData>() as wgpu::BufferAddress,
                },
            }],
            label: Some("Render stage uniform bind group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&uniform_bind_group_layout],
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
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[
                    Vertex::vertex_buffer_descriptor(),
                    InstanceData::vertex_buffer_descriptor(),
                ],
            },
            sample_count: 1,
            sample_mask: 0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            uniform_buff,
            uniform_bind_group,
            pipeline,
        }
    }

    pub fn draw_frame(
        &self,
        renderer: &Renderer,
        frame_packet: &FramePacket,
        encoder: &mut wgpu::CommandEncoder,
        color_output: &wgpu::TextureView,
        depth_output: &wgpu::TextureView,
    ) {
        let uniform_staging = renderer.device.create_buffer_with_data(
            bytemuck::cast_slice(&[UniformData {
                view: frame_packet.view,
                proj: frame_packet.proj,
            }]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &uniform_staging,
            0,
            &self.uniform_buff,
            0,
            std::mem::size_of::<UniformData>() as wgpu::BufferAddress,
        );

        for model in &frame_packet.models {
            let model_data = renderer
                .models
                .get(&model.model_id)
                .expect("Frame packet references model with unknown id");

            let instance_data_buff = renderer.device.create_buffer_with_data(
                bytemuck::cast_slice(&model.instances[..]),
                wgpu::BufferUsage::VERTEX,
            );

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &color_output,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: depth_output,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);

            rpass.set_vertex_buffer(0, &model_data.vertex_buff, 0, 0);
            rpass.set_vertex_buffer(1, &instance_data_buff, 0, 0);
            rpass.set_index_buffer(&model_data.index_buff, 0, 0);
            rpass.draw_indexed(
                0..model_data.index_count,
                0,
                0..model.instances.len() as u32,
            );
        }
    }
}
