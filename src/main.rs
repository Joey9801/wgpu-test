use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time,
};

use bytemuck::Zeroable;
use cgmath::{Angle, Deg, InnerSpace, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod shader_cache;
mod vertex;

use shader_cache::ShaderCache;
use vertex::Vertex;

/// Represents the data for a single model on the CPU
struct ModelData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl ModelData {
    fn new_cube() -> Self {
        let vertices = vec![
            Vertex::new([1.0, -1.0, 1.0]),
            Vertex::new([1.0, -1.0, -1.0]),
            Vertex::new([-1.0, -1.0, 1.0]),
            Vertex::new([-1.0, -1.0, -1.0]),
            Vertex::new([1.0, 1.0, 1.0]),
            Vertex::new([1.0, 1.0, -1.0]),
            Vertex::new([-1.0, 1.0, 1.0]),
            Vertex::new([-1.0, 1.0, -1.0]),
        ];

        let indices = vec![
            4, 2, 0, 2, 7, 3, 6, 5, 7, 1, 7, 5, 0, 3, 1, 4, 1, 5, 4, 6, 2, 2, 6, 7, 6, 4, 5, 1, 3,
            7, 0, 2, 3, 4, 0, 1,
        ];

        Self { vertices, indices }
    }
}

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

fn viewproj(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
    // TODO: use an actual controllable camera rather than hardcoding these

    let view = Matrix4::look_at(
        Point3::new(4.0, 4.0, 4.0),
        Point3::new(0.0, 0.0, 0.2),
        Vector3::new(0.0, 0.0, 1.0),
    );

    let proj = cgmath::perspective(Deg(60.0), aspect_ratio, 0.1, 10.0);

    // cgmath's perspective matrix is for opengl, which has a different screenspace coordinate system to vulkan.
    // To get from the opengl screenspace coordinate system to vulkans, the y axis should be
    // flipped, and the z axis transformed from the range (-1, 1) to (0, 1).
    // Those transforms are represented by the following matrix.
    //
    // rustfmt wants to put all the values below on one line, rather than in a grid that looks
    // like the matrix being constructed.
    #[rustfmt::skip]
    let correction: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );

    // correction * proj * view
    proj * view
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ModelId(usize);

#[derive(Clone, Copy)]
struct InstanceData {
    model_matrix: cgmath::Matrix4<f32>,
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
                    shader_location: 2,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4,
                    shader_location: 3,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 2,
                    shader_location: 4,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: FLOAT_SIZE * 4 * 3,
                    shader_location: 5,
                },
            ],
        }
    }
}

struct FramePacketModel {
    model_id: ModelId,
    instances: Vec<InstanceData>,
}

struct FramePacket {
    view_proj: Matrix4<f32>,
    models: Vec<FramePacketModel>,
}

struct Renderer {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,

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

        let render_stage = RenderStage::new(&device).await;

        Self {
            size,
            surface,
            adapter,
            device,
            queue,
            swapchain,
            next_model_id: ModelId(0),
            models: HashMap::new(),
            render_stage,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn load_model(&mut self, data: ModelData) -> ModelId {
        let new_gpu_model = GpuModel::from_data(&data, &self.device);
        let new_model_id = self.next_model_id;

        self.models.insert(new_model_id, new_gpu_model);
        self.next_model_id = ModelId(self.next_model_id.0 + 1);

        new_model_id
    }

    pub fn get_model(&self, id: &ModelId) -> Option<&GpuModel> {
        self.models.get(id)
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

        self.render_stage.draw_frame(self, frame_packet, &mut encoder, &frame.view);

        self.queue.submit(&[
            encoder.finish(),
        ]);
    }
}

#[derive(Clone, Copy)]
struct UniformData {
    view_proj: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for UniformData {}
unsafe impl bytemuck::Zeroable for UniformData {}

struct RenderStage {
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buff: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl RenderStage {
    pub async fn new(device: &wgpu::Device) -> Self {
        let mut shader_cache = ShaderCache::new();
        let vs_spirv = shader_cache
            .get_shader("./src/shaders/shader.vert", shaderc::ShaderKind::Vertex)
            .await;
        let fs_spirv = shader_cache
            .get_shader("./src/shaders/shader.frag", shaderc::ShaderKind::Fragment)
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
                    visibility: wgpu::ShaderStage::VERTEX,
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
            depth_stencil_state: None,
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
            vs_module,
            fs_module,
            uniform_buff,
            uniform_bind_group_layout,
            uniform_bind_group,
            pipeline,
        }
    }

    pub fn draw_frame(&self, renderer: &Renderer, frame_packet: &FramePacket, encoder: &mut wgpu::CommandEncoder, color_output: &wgpu::TextureView) {
        let uniform_staging = renderer.device.create_buffer_with_data(
            bytemuck::cast_slice(&[UniformData {
                view_proj: frame_packet.view_proj,
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
            let model_data = renderer.get_model(&model.model_id)
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
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);

            rpass.set_vertex_buffer(0, &model_data.vertex_buff, 0, 0);
            rpass.set_vertex_buffer(1, &instance_data_buff, 0, 0);
            rpass.set_index_buffer(&model_data.index_buff, 0, 0);
            rpass.draw_indexed(0..model_data.index_count, 0, 0..model.instances.len() as u32);
        }
    }
}

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window).await;
    let cube_id = renderer.load_model(ModelData::new_cube());

    let static_frame_packet = FramePacket {
        view_proj: viewproj(renderer.aspect_ratio()),
        models: vec![
            FramePacketModel {
                model_id: cube_id,
                instances: vec![
                    InstanceData {
                        model_matrix: cgmath::Matrix4::from_translation([0.0, 0.0, -1.0].into()),
                    },
                    InstanceData {
                        model_matrix: cgmath::Matrix4::from_translation([0.0, 0.0, 2.0].into()),
                    },
                ]
            }
        ],
    };

    std::thread::spawn(move || loop {
        renderer.draw_frame(&static_frame_packet);
        std::thread::sleep(std::time::Duration::from_millis(10));
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
