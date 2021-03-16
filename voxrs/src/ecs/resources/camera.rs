use voxrs_render::blueprint::Camera;
use voxrs_math::*;

pub struct CameraRes {
    eye: Vector3,
    target: Vector3,
    up: Vector3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Clone for CameraRes {
    fn clone(&self) -> Self {
        Self::new(
            self.eye,
            self.target,
            self.up,
            self.aspect,
            self.fovy,
            self.znear,
            self.zfar,
        )
    }
}

impl CameraRes {
    pub fn new(
        eye: Vector3,
        target: Vector3,
        up: Vector3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
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

    pub fn get_sphere(&self) -> Sphere {
        Sphere::from_view_proj(&self.eye, &self.target, &self.up, self.znear, self.zfar, self.aspect, self.fovy)
    }
}

impl Into<Camera> for &CameraRes {
    fn into(self) -> Camera {
        Camera {
            eye: self.eye,
            target: self.target,
            up: self.up,
            aspect: self.aspect,
            fovy: self.fovy,
            znear: self.znear,
            zfar: self.zfar,
            view_proj_mat: self.build_view_projection_matrix(),
        }
    }
}
