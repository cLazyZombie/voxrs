use std::borrow::Borrow;

use crate::{Aabb, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
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

    #[allow(clippy::many_single_char_names)]
    pub fn from_unnorm(x: f32, y: f32, z: f32, d: f32) -> Self {
        let v = Vector3::new(x, y, z);
        let mag = v.magnitude();

        Self::new(v.x() / mag, v.y() / mag, v.z() / mag, d / mag)
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
        Vector3::dot(self.normal(), v) + self.d()
    }

    pub fn normal(&self) -> Vector3 {
        Vector3::new(self.p[0], self.p[1], self.p[2])
    }

    pub fn dist_aabb(&self, aabb: impl Borrow<Aabb>) -> f32 {
        let aabb = Borrow::<Aabb>::borrow(&aabb);

        let center = aabb.center();
        let extend = aabb.max - center;
        let normal = self.normal();

        let r = extend.x() * normal.x().abs()
            + extend.y() * normal.y().abs()
            + extend.z() * normal.z().abs();
        let s = Vector3::dot(&self.normal(), &center) + self.d();

        if s.abs() <= r {
            0.0
        } else if s > 0.0 {
            s - r
        } else {
            s + r
        }
    }
}

#[cfg(test)]
impl approx::AbsDiffEq for Plane {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.p[0], &other.p[0], epsilon)
            && f32::abs_diff_eq(&self.p[1], &other.p[1], epsilon)
            && f32::abs_diff_eq(&self.p[2], &other.p[2], epsilon)
            && f32::abs_diff_eq(&self.p[3], &other.p[3], epsilon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    #[test]
    fn test_create() {
        let p1 = Plane::new(1.0, 0.0, 0.0, 1.0);
        assert_abs_diff_eq!(p1.x(), 1.0);
        assert_abs_diff_eq!(p1, Plane::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_dist() {
        let p1 = Plane::new(1.0, 0.0, 0.0, -5.0);
        let point = Vector3::new(10.0, 20.0, 30.0);

        assert_abs_diff_eq!(p1.dist(&point), 5.0);
        assert_abs_diff_eq!(p1.dist(point), 5.0);
    }

    #[test]
    fn test_dist_aabb() {
        let plane = Plane::new(1.0, 0.0, 0.0, -5.0);

        let aabb = Aabb::new(
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new(10.0, 10.0, 10.0),
        );
        let dist = plane.dist_aabb(aabb);
        assert_abs_diff_eq!(dist, 0.0);

        let aabb = Aabb::new(Vector3::new(6.0, 6.0, 6.0), Vector3::new(10.0, 10.0, 10.0));
        let dist = plane.dist_aabb(aabb);
        assert_abs_diff_eq!(dist, 1.0);

        let aabb = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(4.0, 4.0, 4.0));
        let dist = plane.dist_aabb(aabb);
        assert_abs_diff_eq!(dist, -1.0);
    }
}
