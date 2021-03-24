use crate::{Aabb, Dir, Vector3};

pub struct Ray {
    pub origin: Vector3,
    pub dir: Vector3,
}

impl Ray {
    pub fn new() -> Self {
        Self::from_values(&Vector3::zero(), &Vector3::front())
    }

    pub fn from_values(origin: &Vector3, dir: &Vector3) -> Self {
        Self {
            origin: *origin,
            dir: *dir,
        }
    }

    /// ray - aabb intersection
    /// reference: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
    pub fn check_aabb(&self, aabb: &Aabb) -> RayAabbResult {
        if aabb.has_point(&self.origin) {
            return RayAabbResult::Inside;
        }

        let mut collision_dir = Dir::XNeg;

        let mut tmin = (aabb.min.x() - self.origin.x()) / self.dir.x();
        let mut tmax = (aabb.max.x() - self.origin.x()) / self.dir.x();
        if tmin > tmax {
            collision_dir = Dir::XPos;
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut ydir = Dir::YNeg;
        let mut tymin = (aabb.min.y() - self.origin.y()) / self.dir.y();
        let mut tymax = (aabb.max.y() - self.origin.y()) / self.dir.y();
        if tymin > tymax {
            ydir = Dir::YPos;
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return RayAabbResult::NotIntersect;
        }

        if tymin > tmin {
            collision_dir = ydir;
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut zdir = Dir::ZNeg;
        let mut tzmin = (aabb.min.z() - self.origin.z()) / self.dir.z();
        let mut tzmax = (aabb.max.z() - self.origin.z()) / self.dir.z();
        if tzmin > tzmax {
            zdir = Dir::ZPos;
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return RayAabbResult::NotIntersect;
        }

        if tzmin > tmin {
            collision_dir = zdir;
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        if tmin < 0.0 && tmax < 0.0 {
            return RayAabbResult::NotIntersect;
        }

        if tmin < 0.0 && tmax >= 0.0 {
            tmin = 0.0;
        }

        return RayAabbResult::Intersect {
            dist: tmin,
            pos: self.origin + self.dir * tmin,
            dir: collision_dir,
        };
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new()
    }
}

/// RayAabbResult
/// Inside: ray is inside of aabb
/// NotIntersect: not intersected
/// Intersect: with distance and hit direction
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RayAabbResult {
    Inside,
    NotIntersect,
    Intersect { dist: f32, pos: Vector3, dir: Dir },
}

pub struct RayVoxelIter<'a> {
    ray: &'a Ray,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn check_aabb() {
        // z negative
        let ray = Ray::from_values(&Vector3::new(5.0, 5.0, -10.0), &Vector3::front());
        let aabb = Aabb::new(Vector3::zero(), Vector3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::Intersect { .. }));
        if let RayAabbResult::Intersect { dist, pos, dir } = result {
            assert_abs_diff_eq!(pos, Vector3::new(5.0, 5.0, 0.0));
            assert_abs_diff_eq!(dist, 10.0);
            assert_eq!(dir, Dir::ZNeg);
        }

        // z positive
        let ray = Ray::from_values(&Vector3::new(5.0, 5.0, 20.0), &Vector3::back());
        let aabb = Aabb::new(Vector3::zero(), Vector3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::Intersect { .. }));
        if let RayAabbResult::Intersect { dist, pos, dir } = result {
            assert_abs_diff_eq!(pos, Vector3::new(5.0, 5.0, 10.0));
            assert_abs_diff_eq!(dist, 10.0);
            assert_eq!(dir, Dir::ZPos);
        }

        // not intersect (reversed dir)
        let ray = Ray::from_values(&Vector3::new(5.0, 5.0, 20.0), &Vector3::front());
        let aabb = Aabb::new(Vector3::zero(), Vector3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::NotIntersect));
    }
}
