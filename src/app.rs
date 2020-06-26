use std::time::Duration;

use cgmath::{Rad, Quaternion, Vector3, Point3, InnerSpace, Angle, Matrix4, SquareMatrix, Deg};

use crate::camera::Camera;
use crate::renderer::{
    ModelId,
    frame_packet::{FramePacket, FramePacketModel, InstanceData}
};
use winit::event::{ElementState, VirtualKeyCode};


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

    fn translate(&mut self, translation: Vector3<f32>) {
        self.pos += translation;
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
        let mut normal = model_view.invert().expect("Model-View matrix had a zero determinant");
        normal.transpose_self();
        normal
    }
}

pub struct App {
    main_camera: Camera,

    /// Camera velocity relative to the camera
    ///
    /// The Z component of this vector is straight up in world space
    /// The Y component is in the direction the camera is facing
    /// The X component points right out of the camera (camera.dir cross world up)
    camera_velocity: Vector3<f32>,

    object: AppObject,
}

impl App {
    pub fn new(model: ModelId) -> Self {
        Self {
            main_camera: Camera {
                location: [2.0, 2.0, 0.0].into(),
                direction: Vector3::new(-1.0, -1.0, 0.0), //.normalize(),
                ..Camera::default()
            },
            camera_velocity: [0.0, 0.0, 0.0].into(),
            object: AppObject {
                model,
                scale: 1.0,
                pos: [0.0, 0.0, 0.0].into(),
                angle: [1.0, 0.0, 0.0, 0.0].into(),
            },
        }
    }

    pub fn handle_mouse_delta(&mut self, delta: [f32; 2]) {
        self.main_camera.pan_horizonal(Rad(delta[0]));

        // A negative vertical delta is the mouse moving toward the top of the screen.
        // Invert it so that the mouse moving up causes a pan upwards.
        self.main_camera.pan_vertical(Rad(-delta[1]));
    }

    pub fn handle_key_event(&mut self, key: VirtualKeyCode, new_state: ElementState) {
        let multiplier: f32 = match new_state {
            ElementState::Pressed => 10.0,
            ElementState::Released => -10.0,
        };

        let base_vel: Vector3<f32> = match key {
            VirtualKeyCode::W => [0.0, 1.0, 0.0],
            VirtualKeyCode::A => [-1.0, 0.0, 0.0],
            VirtualKeyCode::S => [0.0, -1.0, 0.0],
            VirtualKeyCode::D => [1.0, 0.0, 0.0],
            VirtualKeyCode::Space => [0.0, 0.0, 1.0],
            VirtualKeyCode::C => [0.0, 0.0, -1.0],
            _ => [0.0, 0.0, 0.0],
        }.into();

        self.camera_velocity += multiplier * base_vel;

        match key {
            VirtualKeyCode::Add => self.main_camera.vertical_fov += Deg(5.0).into(),
            VirtualKeyCode::Subtract => self.main_camera.vertical_fov -= Deg(5.0).into(),
            _ => (),
        }
    }

    // Generates the world space camera velocity from the camera space first person velocity.
    fn world_camera_vel(&self) -> Vector3<f32> {
            let strafe_dir = self.main_camera.direction.cross([0.0, 0.0, 1.0].into()).normalize();
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
            models: vec![
                FramePacketModel {
                    model_id: self.object.model,
                    instances: vec![
                        InstanceData {
                            model_matrix: self.object.model_matrix(),
                            normal_matrix: self.object.normal_matrix(view),
                        }
                    ]
                }
            ],
        }
    }
}