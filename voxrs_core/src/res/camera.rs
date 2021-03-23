use voxrs_math::*;
use voxrs_render::blueprint;

/// CameraRes is free moving camera
pub struct CameraRes {
    eye: Vector3,
    horizon: Angle,
    vert: Angle,
    width: u32,
    height: u32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl CameraRes {
    pub fn new(
        eye: Vector3,
        horizon: Angle,
        vert: Angle,
        width: u32,
        height: u32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            horizon,
            vert,
            width,
            height,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        let view = Matrix4::look_at(&self.eye, &target, &y);
        let proj = Matrix4::perspective(self.aspect(), self.fovy, self.znear, self.zfar);

        proj * view
    }

    pub fn view_matrix(&self) -> Matrix4 {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        let view = Matrix4::look_at(&self.eye, &target, &y);
        view
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn move_camera(&mut self, offset: &Vector3) {
        self.eye += *offset;
    }

    pub fn move_camera_relative(&mut self, rel_offset: &Vector3) {
        let (x, y, z) = self.get_xyz();
        let offset = x * rel_offset.x() + y * rel_offset.y() + z * rel_offset.z();
        self.move_camera(&offset);
    }

    /// horizon: positive -> right, radians
    /// vert: positive -> up. radians
    pub fn rotate_camera(&mut self, horizon: Angle, vert: Angle) {
        self.horizon = (self.horizon + horizon).normalize();
        self.vert = (self.vert + vert).clamp_half();
    }

    pub fn get_sphere(&self) -> Sphere {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        Sphere::from_view_proj(
            &self.eye,
            &target,
            &y,
            self.znear,
            self.zfar,
            self.aspect(),
            self.fovy,
        )
    }

    /// get_xyz returns x(right) direction, y(up) direction, z(forward) direction in world coord
    pub fn get_xyz(&self) -> (Vector3, Vector3, Vector3) {
        let mut q = Quat::from_rotate_axis(&Vector3::new(1.0, 0.0, 0.0), -self.vert);
        q.rotate_axis(&Vector3::new(0.0, 1.0, 0.0), self.horizon);

        let x = q.transform(&Vector3::new(1.0, 0.0, 0.0));
        let y = q.transform(&Vector3::new(0.0, 1.0, 0.0));
        let z = q.transform(&Vector3::new(0.0, 0.0, 1.0));

        (x, y, z)
    }

    pub fn create_ray(&self, screen_xy: (f32, f32)) -> Ray {
        // x : -1(left) to 1(right)
        let mut x = screen_xy.0 as f32;
        x /= self.width as f32;
        x = x.clamp(0.0, 1.0);
        x = x * 2.0 - 1.0;

        // y : -1(bottom) to 1(top)
        let mut y = (self.height as f32) - (screen_xy.1 as f32);
        y /= self.height as f32;
        y = y.clamp(0.0, 1.0);
        y = y * 2.0 - 1.0;

        // make view dir (view coord)
        let tan_y = (self.fovy * 0.5).tan();
        let tan_x = tan_y * self.aspect();
        let view_x = tan_x * (x as f32);
        let view_y = tan_y * (y as f32);
        let view_dir = Vector3::new(view_x, view_y, 1.0).get_normalized();

        // transform to world coord
        let view_mat = self.view_matrix();
        let inv_view_mat = view_mat.inverse();
        let world_dir = inv_view_mat.transform_normal(&view_dir);

        Ray::from_values(&self.eye, &world_dir)
    }
}

impl Into<blueprint::Camera> for &CameraRes {
    fn into(self) -> blueprint::Camera {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        blueprint::Camera {
            eye: self.eye,
            target,
            up: y,
            aspect: self.aspect(),
            fovy: self.fovy,
            znear: self.znear,
            zfar: self.zfar,
            view_proj_mat: self.build_view_projection_matrix(),
        }
    }
}
