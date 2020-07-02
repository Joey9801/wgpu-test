use std::time::Duration;

use cgmath::{Angle, Deg, InnerSpace, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3};

use crate::camera::Camera;
use crate::input_manager::{InputManager, KeyState, LogicalEvent, LogicalKey};
use crate::renderer::{
    frame_packet::{FramePacket, FramePacketModel, InstanceData, FramePacketSprites, SpriteInstanceData},
    ModelId, AtlasId,
};

struct AppObject {
    model: ModelId,
    scale: f32,
    pos: Point3<f32>,
    angle: Quaternion<f32>,
}

impl AppObject {
    fn rotate(&mut self, angle: impl Into<Rad<f32>>, axis: Vector3<f32>) {
        let angle = angle.into() / 2.0;
        let s = angle.sin();
        let c = angle.cos();
        let rotation = Quaternion::new(c, axis.x * s, axis.y * s, axis.z * s);

        self.angle = (rotation * self.angle).normalize();
    }

    /// Generates a matrix that transforms this objects model space into world space
    fn model_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.pos.x, self.pos.y, self.pos.z))
            * Matrix4::from(self.angle)
            * Matrix4::from_scale(self.scale)
    }

    /// Generates a matrix that transforms normals from this objects model space to the given view
    /// space
    fn normal_matrix(&self, view: Matrix4<f32>) -> Matrix4<f32> {
        let model_view = view * self.model_matrix();
        let mut normal = model_view
            .invert()
            .expect("Model-View matrix had a zero determinant");
        normal.transpose_self();
        normal
    }
}

pub struct App {
    input_manager: InputManager,
    main_camera: Camera,

    /// Camera velocity relative to the camera
    ///
    /// The Z component of this vector is straight up in world space
    /// The Y component is in the direction the camera is facing
    /// The X component points right out of the camera (camera.dir cross world up)
    camera_velocity: Vector3<f32>,

    object: AppObject,

    ui_atlas: AtlasId,
}

impl App {
    pub fn new(model: ModelId, ui_atlas: AtlasId) -> Self {
        let mut object = AppObject {
            model,
            scale: 0.4,
            pos: [0.0, 0.0, -1.0].into(),
            angle: [1.0, 0.0, 0.0, 0.0].into(),
        };
        object.rotate(Deg(90.0), [1.0, 0.0, 0.0].into());

        Self {
            input_manager: InputManager::new(),
            main_camera: Camera {
                location: [2.0, 2.0, 0.0].into(),
                direction: Vector3::new(-1.0, -1.0, 0.0).normalize(),
                ..Camera::default()
            },
            camera_velocity: [0.0, 0.0, 0.0].into(),
            object,
            ui_atlas,
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.input_manager.update(event);
        while let Some(logical_event) = self.input_manager.poll_logical_event() {
            self.handle_logical_event(logical_event);
        }
    }

    fn handle_logical_event(&mut self, event: LogicalEvent) {
        match event {
            LogicalEvent::MouseMovement { x, y } => {
                const MOUSE_SCALING: f32 = 1.0 / 1024.0;
                self.main_camera.pan_horizonal(Rad(x * MOUSE_SCALING));

                // A negative vertical delta is the mouse moving toward the top of the screen.
                // Invert it so that the mouse moving upwards is a positive vertical pan (looking
                // more up)
                self.main_camera.pan_vertical(Rad(-y * MOUSE_SCALING));
            }
            LogicalEvent::Key {
                logical_key,
                new_state,
            } => {
                self.handle_key_event(logical_key, new_state);
            }
        }
    }

    fn handle_key_event(&mut self, key: LogicalKey, new_state: KeyState) {
        let multiplier: f32 = match new_state {
            KeyState::Down => 10.0,
            KeyState::Up => -10.0,
        };

        let base_vel: Vector3<f32> = match key {
            LogicalKey::MoveForward => [0.0, 1.0, 0.0],
            LogicalKey::StrafeLeft => [-1.0, 0.0, 0.0],
            LogicalKey::MoveBackward => [0.0, -1.0, 0.0],
            LogicalKey::StrafeRight => [1.0, 0.0, 0.0],
            LogicalKey::MoveUp => [0.0, 0.0, 1.0],
            LogicalKey::MoveDown => [0.0, 0.0, -1.0],
        }
        .into();

        self.camera_velocity += multiplier * base_vel;
    }

    // Generates the world space camera velocity from the camera space first person velocity.
    fn world_camera_vel(&self) -> Vector3<f32> {
        let strafe_dir = self
            .main_camera
            .direction
            .cross([0.0, 0.0, 1.0].into())
            .normalize();
        let strafe: Vector3<f32> = strafe_dir * self.camera_velocity.x;
        let forward: Vector3<f32> = self.camera_velocity.y * self.main_camera.direction;
        let up: Vector3<f32> = self.camera_velocity.z * Vector3::new(0.0, 0.0, 1.0);
        strafe + forward + up
    }

    /// Allow the given amount of time to pass
    pub fn tick(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();
        self.object.rotate(Deg(100.0) * dt, [0.0, 0.0, 1.0].into());
        self.main_camera.location += self.world_camera_vel() * dt;
    }

    pub fn generate_frame_packet(&self, aspect_ratio: f32) -> FramePacket {
        let view = self.main_camera.view();
        let proj = self.main_camera.proj(aspect_ratio);

        FramePacket {
            view,
            proj,
            models: vec![FramePacketModel {
                model_id: self.object.model,
                instances: vec![InstanceData {
                    model_matrix: self.object.model_matrix(),
                    normal_matrix: self.object.normal_matrix(view),
                }],
            }],
            overlay_sprites: vec![FramePacketSprites {
                atlas_id: self.ui_atlas,
                sprites: vec![
                    SpriteInstanceData {
                        screen_pos: [0.09, 0.16].into(),
                        screen_size: [-0.09, -0.16].into(),
                        atlas_pos: [0.0, 0.0].into(),
                        atlas_size: [1.0, 1.0].into(),
                    }
                ]
            }]
        }
    }
}
