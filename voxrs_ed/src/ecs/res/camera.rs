use voxrs_math::*;
use voxrs_render::blueprint;

pub struct Camera {
    eye: Vector3,
    dir: Vector3, // look at dir
    up: Vector3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Vector3,
        dir: Vector3,
        up: Vector3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            dir,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let target = self.get_eye_target();
        let view = Matrix4::look_at(&self.eye, &target, &self.up);
        let proj = Matrix4::perspective(self.aspect, self.fovy, self.znear, self.zfar);

        proj * view
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn move_camera(&mut self, offset: &Vector3) {
        self.eye += *offset;
    }

    pub fn move_camera_relative(&mut self, rel_offset: &Vector3) {
        let right = Vector3::cross(&self.up, &self.dir).get_normalized();
        let offset = self.dir * rel_offset.z() + self.up * rel_offset.y() + right * rel_offset.x();
        self.move_camera(&offset);
    }

    /// horizon: positive -> right, radians
    /// vert: positive -> up. radians
    pub fn rotate_camera(&mut self, horizon: f32, vert: f32) {
        let up = self.up;
        let dir = self.dir;
        let mut right = Vector3::cross(&up, &dir).get_normalized();

        let mut q = Quat::from_rotate_axis(&up, horizon);
        right = q.transform(&right);

        q.rotate_axis(&right, -vert);

        self.up = q.transform(&up);
        self.dir = q.transform(&dir);
    }

    pub fn get_sphere(&self) -> Sphere {
        Sphere::from_view_proj(
            &self.eye,
            &(self.eye + self.dir),
            &self.up,
            self.znear,
            self.zfar,
            self.aspect,
            self.fovy,
        )
    }

    pub fn get_eye_target(&self) -> Vector3 {
        self.eye + self.dir
    }
}

impl Into<blueprint::Camera> for &Camera {
    fn into(self) -> blueprint::Camera {
        blueprint::Camera {
            eye: self.eye,
            target: self.get_eye_target(),
            up: self.up,
            aspect: self.aspect,
            fovy: self.fovy,
            znear: self.znear,
            zfar: self.zfar,
            view_proj_mat: self.build_view_projection_matrix(),
        }
    }
}
