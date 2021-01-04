use crate::math::{Matrix4, Vector3};

pub struct Camera {
    eye: Vector3,
    target: Vector3,
    up: Vector3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Vector3,
        target: Vector3,
        up: Vector3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let view = Matrix4::look_at(&self.eye, &self.target, &self.up);
        let proj = Matrix4::perspective(self.aspect, self.fovy, self.znear, self.zfar);

        proj * view
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn move_camera(&mut self, offset: Vector3) {
        self.eye += offset;
        self.target += offset;
    }
}
