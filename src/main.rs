#[cfg(test)]
#[macro_use]
extern crate cgmath;

use winit::{
    event::{self, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

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
    let primary_monitor = event_loop.primary_monitor();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(primary_monitor.size())
        .with_fullscreen(Some(Fullscreen::Borderless(primary_monitor)))
        // .with_inner_size(winit::dpi::LogicalSize {width: 2560, height: 1440 })
        .build(&event_loop)
        .unwrap();

    window.set_cursor_grab(true).expect("Failed to grab cursor");
    window.set_cursor_visible(false);

    let mut renderer = Renderer::new(&window).await;

    let suzanne_id = renderer.upload_model(
        ModelData::load_gltf("./suzanne.glb")
            .await
            .expect("Failed to upload model to GPU"),
    );

    let mut app = App::new(suzanne_id);

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
