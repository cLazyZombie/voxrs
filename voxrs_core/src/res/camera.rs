use voxrs_math::*;
use voxrs_render::blueprint;

/// CameraRes is free moving camera
pub struct CameraRes {
    eye: Vec3,
    horizon: Angle,
    vert: Angle,
    width: u32,
    height: u32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl CameraRes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        eye: Vec3,
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

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        let view = Mat4::look_at_lh(self.eye, target, y);
        let proj = Mat4::perspective_lh(self.fovy, self.aspect(), self.znear, self.zfar);

        proj * view
    }

    pub fn view_matrix(&self) -> Mat4 {
        let (_, y, z) = self.get_xyz();
        let target = self.eye + z;
        Mat4::look_at_lh(self.eye, target, y)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn move_camera(&mut self, offset: &Vec3) {
        self.eye += *offset;
    }

    pub fn move_camera_relative(&mut self, rel_offset: &Vec3) {
        let (x, y, z) = self.get_xyz();
        let offset = x * rel_offset.x + y * rel_offset.y + z * rel_offset.z;
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
        Sphere::from_view_proj(&self.eye, &target, &y, self.znear, self.zfar, self.aspect(), self.fovy)
    }

    /// get_xyz returns x(right) direction, y(up) direction, z(forward) direction in world coord
    pub fn get_xyz(&self) -> (Vec3, Vec3, Vec3) {
        let q1 = Quat::from_rotation_x(-self.vert.to_radians());
        let q2 = Quat::from_rotation_y(self.horizon.to_radians());
        let q = q2 * q1;

        let x = q.mul_vec3(Vec3::X);
        let y = q.mul_vec3(Vec3::Y);
        let z = q.mul_vec3(Vec3::Z);

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
        let view_dir = Vec3::new(view_x, view_y, 1.0).normalize();

        // transform to world coord
        let view_mat = self.view_matrix();
        let inv_view_mat = view_mat.inverse();
        let world_dir = inv_view_mat.transform_vector3(view_dir);

        Ray::from_values(&self.eye, &world_dir)
    }
}

impl From<&CameraRes> for blueprint::Camera {
    fn from(camera_res: &CameraRes) -> Self {
        let (_, y, z) = camera_res.get_xyz();
        let target = camera_res.eye + z;
        blueprint::Camera {
            eye: camera_res.eye,
            target,
            up: y,
            aspect: camera_res.aspect(),
            fovy: camera_res.fovy,
            znear: camera_res.znear,
            zfar: camera_res.zfar,
            view_proj_mat: camera_res.build_view_projection_matrix(),
        }
    }
}
