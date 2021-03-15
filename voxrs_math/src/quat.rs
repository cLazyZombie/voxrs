use nalgebra_glm::Qua;

use crate::Vector3;

pub struct Quat {
    q: Qua<f32>,
}

impl Quat {
    /// make quaternion 
    /// ratate from dir1 to dir2
    pub fn rotation(dir1: &Vector3, dir2: &Vector3) -> Self {
        let rot = nalgebra_glm::quat_rotation(&dir1.v, &dir2.v);
        Self { 
            q: rot,
        }
    }

    pub fn transform(&self, p: &Vector3) -> Vector3 {
        let v = nalgebra_glm::quat_rotate_vec3(&self.q, &p.v);
        Vector3 {
            v,
        }
    }
}