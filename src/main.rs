#[cfg(test)]
#[macro_use]
extern crate approx;

use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent, self},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod shader_cache;
mod vertex;
mod model_data;
mod renderer;
mod camera;
mod input_manager;
mod app;


use vertex::Vertex;
use model_data::ModelData;
use renderer::{
    Renderer,
};
use app::App;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(winit::dpi::LogicalSize {width: 1920, height: 1080 })
        .build(&event_loop)
        .unwrap();

    window.set_cursor_grab(true).expect("Failed to grab cursor");
    window.set_cursor_visible(false);
    let window_size = window.inner_size().cast::<f64>();
    let window_center = PhysicalPosition {
        x: window_size.width / 2.0, y: window_size.height / 2.0
    };

    let mut renderer = Renderer::new(&window).await;

    let suzanne_id = renderer
        .upload_model(ModelData::load_gltf("./suzanne.glb")
            .await
            .expect("Failed to upload model to GPU")
        );

    let mut app = App::new(suzanne_id);

    let mut last_update_inst = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(1));
        
        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() > Duration::from_secs_f32(1.0 / 144.0) {
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
            _ => app.handle_event(&event)
        }

    });
}
