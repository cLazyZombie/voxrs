use std::borrow::Borrow;

use nalgebra_glm as glm;

use crate::{Aabb, Vector3};

#[derive(Copy, Clone)]
pub struct Plane {
    p: glm::Vec4,
}

impl Plane {
    /// if some p on plane,
    /// p.x * x + p.y * y + p.z * z + d = 0 should be true
    pub fn new(x: f32, y: f32, z: f32, d: f32) -> Self {
        Self {
            p: glm::Vec4::new(x, y, z, d),
        }
    }

    pub fn x(&self) -> f32 {
        self.p[0]
    }

    pub fn y(&self) -> f32 {
        self.p[1]
    }

    pub fn z(&self) -> f32 {
        self.p[2]
    }

    pub fn d(&self) -> f32 {
        self.p[3]
    }

    pub fn dist(&self, v: impl Borrow<Vector3>) -> f32 {
        let v = v.borrow();
        v.x() * self.x() + v.y() * self.y() + v.z() * self.z() + self.d()
    }

    pub fn dist_aabb(&self, _aabb: impl Borrow<Aabb>) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let p1 = Plane::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(p1.x(), 1.0);
    }

    #[test]
    fn test_dist() {
        let p1 = Plane::new(1.0, 0.0, 0.0, -5.0);
        let point = Vector3::new(10.0, 20.0, 30.0);

        assert_eq!(p1.dist(&point), 5.0);
        assert_eq!(p1.dist(point), 5.0);
    }
}