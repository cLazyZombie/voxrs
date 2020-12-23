
pub struct Camera {
    eye: glm::Vec3,
    target: glm::Vec3,
    up: glm::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(eye: glm::Vec3, target: glm::Vec3, up: glm::Vec3, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
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

    pub fn build_view_projection_matrix(&self) -> glm::Mat4 {
        let view : glm::Mat4 = glm::look_at_lh(&self.eye, &self.target, &self.up);
        let proj : glm::Mat4 = glm::perspective_lh_zo(self.aspect, self.fovy, self.znear, self.zfar);

        return proj * view;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn move_camera(&mut self, offset: glm::Vec3) {
        self.eye += offset;
        self.target += offset;
    }
}