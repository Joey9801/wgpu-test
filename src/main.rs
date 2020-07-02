#[cfg(test)]
#[macro_use]
extern crate cgmath;

use winit::{
    dpi::PhysicalSize,
    event::{self, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::fs::File;
use tokio::prelude::*;

mod app;
mod camera;
mod input_manager;
mod model_data;
mod renderer;
mod shader_cache;
mod vertex;

use app::App;
use model_data::ModelData;
use renderer::Renderer;
use std::time::{Duration, Instant};
use vertex::Vertex;

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(PhysicalSize {
            width: 1920,
            height: 1080,
        })
        .with_title("wgpu-test")
        .build(&event_loop)
        .unwrap();

    window.set_cursor_grab(true).expect("Failed to grab cursor");
    window.set_cursor_visible(false);

    let mut renderer = Renderer::new(&window).await;

    let model_id = renderer.upload_model(
        ModelData::load_gltf("./AntiqueCamera.glb")
            .await
            .expect("Failed to load model from disk"),
    );

    let atlas_id;
    {
        let mut atlas_file = File::open("./atlas.png")
            .await
            .expect("Failed to open atlas file");
        let mut atlas_data = Vec::new();
        atlas_file.read_to_end(&mut atlas_data)
            .await
            .expect("Failed to read atlas file");
        let atlas_data = image::load_from_memory(&atlas_data)
            .expect("Failed to parse atlas file");
        atlas_id = renderer.upload_atlas(atlas_data.to_rgba());
    }

    let mut app = App::new(model_id, atlas_id);

    let mut last_update_inst = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(1));

        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() > Duration::from_secs_f32(1.0 / 200.0) {
                    app.tick(last_update_inst.elapsed());
                    last_update_inst = Instant::now();
                    window.request_redraw();
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            event::Event::RedrawRequested(_) => {
                let frame_packet = app.generate_frame_packet(renderer.aspect_ratio());
                renderer.draw_frame(&frame_packet);
            }
            _ => app.handle_event(&event),
        }
    });
}
