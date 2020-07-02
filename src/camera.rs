use cgmath::{Deg, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};

pub struct Camera {
    /// Position of this camera in world coordinates
    pub location: Point3<f32>,

    /// A unit vector in the direction this camera is facing
    pub direction: Vector3<f32>,

    /// Near clipping plane for the perspective projection
    pub near_clip: f32,

    /// Far clipping plane for the perspective projection
    pub far_clip: f32,

    pub vertical_fov: Rad<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            location: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(1.0, 0.0, 0.0),
            near_clip: 0.1,
            far_clip: 1000.0,
            vertical_fov: Deg(90.0).into(),
        }
    }
}

impl Camera {
    /// Generate a matrix that transforms world space into this camera's view space
    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_dir(self.location, self.direction, [0.0, 0.0, 1.0].into())
    }

    /// Generate a matrix that transforms view space into Vulkan screenspace coordinates
    pub fn proj(&self, aspect_ratio: f32) -> Matrix4<f32> {
        // OPENGL_SCREENSPACE_TO_VULKAN *
        cgmath::perspective(
            self.vertical_fov,
            aspect_ratio,
            self.near_clip,
            self.far_clip,
        )
    }

    /// Pan this camera left/right
    pub fn pan_horizonal<A: Into<Rad<f32>>>(&mut self, angle: A) {
        let rot_matrix = Matrix3::from_axis_angle([0.0, 0.0, 1.0].into(), Rad(0.0) - angle.into());
        self.direction = rot_matrix * self.direction;
    }

    /// Pan this camera up/down
    ///
    /// Clamps the vertical pan to straight up/straight down.
    /// A positive angle pans upwards.
    pub fn pan_vertical<A: Into<Rad<f32>>>(&mut self, pan_angle: A) {
        // Vector pointing out the right hand side of the camera
        let axis = self.direction.cross([0.0, 0.0, 1.0].into()).normalize();

        // Rad(pi/2) => straight up
        // Rad(0) => horizontal
        // Rad(-pi/2) => straight down
        let current_angle: Rad<f32> =
            Rad(std::f32::consts::FRAC_PI_2 - self.direction.dot([0.0, 0.0, 1.0].into()).acos());

        // Bounds for the pan angle that prevent the camera going past straight up/down. Include a
        // small amount of buffer room so that the camera is never quite pointing straight up/down,
        // so that the cross product of the camera direction and the vertical is allways well
        // defined.
        let max_pan: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2) - current_angle - Rad(0.01);
        let min_pan: Rad<f32> = Rad(-std::f32::consts::FRAC_PI_2) - current_angle + Rad(0.01);

        let pan_angle = pan_angle.into();
        let pan_angle = if pan_angle > max_pan {
            max_pan
        } else if pan_angle < min_pan {
            min_pan
        } else {
            pan_angle
        };

        let rot_matrix = Matrix3::from_axis_angle(axis, pan_angle);
        self.direction = rot_matrix * self.direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_pan_horizontal() {
        let mut camera = Camera::default();

        camera.direction = Vector3::new(1.0, 0.0, 0.0);
        camera.pan_horizonal(Deg(-90.0));
        assert_ulps_eq!(camera.direction.x, 0.0);
        assert_ulps_eq!(camera.direction.y, 1.0);
        assert_ulps_eq!(camera.direction.z, 0.0);

        camera.direction = Vector3::new(1.0, 0.0, 0.0);
        camera.pan_horizonal(Deg(90.0));
        assert_ulps_eq!(camera.direction.x, 0.0);
        assert_ulps_eq!(camera.direction.y, -1.0);
        assert_ulps_eq!(camera.direction.z, 0.0);
    }

    #[test]
    fn test_camera_pan_vertical() {
        let mut camera = Camera::default();

        camera.direction = Vector3::new(1.0, 0.0, 0.0);
        camera.pan_vertical(Deg(45.0));
        assert_ulps_eq!(camera.direction.magnitude(), 1.0);
        // This one should be pretty exact
        assert_ulps_eq!(
            camera.direction,
            [(0.5f32).sqrt(), 0.0, (0.5f32).sqrt()].into()
        );

        camera.pan_vertical(Deg(90.0));
        assert_ulps_eq!(camera.direction.magnitude(), 1.0);
        // Relatively large epsilon from here on to account for the fudge factor preventing looking straight up
        assert_relative_eq!(camera.direction, [0.0, 0.0, 1.0].into(), epsilon = 0.01);

        camera.pan_vertical(Deg(-90.0));
        assert_ulps_eq!(camera.direction.magnitude(), 1.0);
        assert_relative_eq!(camera.direction, [1.0, 0.0, 0.0].into(), epsilon = 0.01);

        camera.pan_vertical(Deg(-45.0));
        assert_ulps_eq!(camera.direction.magnitude(), 1.0);
        assert_relative_eq!(
            camera.direction,
            [(0.5f32).sqrt(), 0.0, -(0.5f32).sqrt()].into(),
            epsilon = 0.01
        );

        camera.pan_vertical(Deg(-90.0));
        assert_ulps_eq!(camera.direction.magnitude(), 1.0);
        assert_relative_eq!(camera.direction, [0.0, 0.0, -1.0].into(), epsilon = 0.01);
    }
}
