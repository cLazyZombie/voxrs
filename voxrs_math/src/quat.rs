use nalgebra_glm::Qua;

use crate::Vector3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quat {
    q: Qua<f32>,
}

impl Quat {
    pub fn identity() -> Self {
        let q = Qua::identity();
        Self { q }
    }

    pub fn from_rotate_axis(axis: &Vector3, angle: f32) -> Self {
        let q = nalgebra_glm::quat_angle_axis(angle, &axis.v);
        Self { q }
    }
    /// make quaternion
    /// ratate from dir1 to dir2
    pub fn from_two_dirs(dir1: &Vector3, dir2: &Vector3) -> Self {
        let rot = nalgebra_glm::quat_rotation(&dir1.v, &dir2.v);
        Self { q: rot }
    }

    pub fn transform(&self, p: &Vector3) -> Vector3 {
        let v = nalgebra_glm::quat_rotate_vec3(&self.q, &p.v);
        Vector3 { v }
    }

    pub fn rotate_axis(&mut self, axis: &Vector3, angle: f32) {
        let q2 = Self::from_rotate_axis(axis, angle);
        *self = q2 * (*self);
        // let q2 = nalgebra_glm::quat_angle_axis(angle, &axis.v);
        // self.q = q2 * self.q;
    }
}

impl std::ops::Mul for Quat {
    type Output = Quat;

    fn mul(self, rhs: Self) -> Self::Output {
        Self { q: self.q * rhs.q }
    }
}

impl From<&[f32]> for Quat {
    fn from(v: &[f32]) -> Self {
        assert_eq!(v.len(), 4);
        Self {
            q: nalgebra_glm::quat(v[0], v[1], v[2], v[3]),
        }
    }
}

impl From<&[f32; 4]> for Quat {
    fn from(v: &[f32; 4]) -> Self {
        Self {
            q: nalgebra_glm::quat(v[0], v[1], v[2], v[3]),
        }
    }
}

impl approx::AbsDiffEq for Quat {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        0.001
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        for i in 0..4 {
            if (self.q[i] - other.q[i]).abs() > epsilon {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::*;

    #[test]
    fn test_identity() {
        let q = Quat::identity();
        assert_eq!(q, Quat::from(&[0.0, 0.0, 0.0, 1.0]));
        assert_abs_diff_eq!(q, Quat::identity());
    }

    #[test]
    fn test_rotate_axis() {
        let mut q = Quat::identity();
        let v = Vector3::new(100.0, 0.0, 0.0);

        // make rotate from yaxis, 90 degree
        q.rotate_axis(&[0.0, 1.0, 0.0].into(), (90.0_f32).to_radians());
        let rotated = q.transform(&v);
        assert_abs_diff_eq!(rotated, Vector3::new(0.0, 0.0, -100.0));

        // make rotate from x axis, 90 degree
        q.rotate_axis(&[1.0, 0.0, 0.0].into(), (90.0_f32).to_radians());
        let rotated = q.transform(&v);
        assert_abs_diff_eq!(rotated, Vector3::new(0.0, 100.0, 0.0));
    }
}
