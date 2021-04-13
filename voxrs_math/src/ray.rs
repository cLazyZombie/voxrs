use crate::{Aabb, BlockPos, Dir, Vec3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new() -> Self {
        Self::from_values(&Vec3::ZERO, &Vec3::Z)
    }

    pub fn from_values(origin: &Vec3, dir: &Vec3) -> Self {
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

        let mut tmin = (aabb.min.x - self.origin.x) / self.dir.x;
        let mut tmax = (aabb.max.x - self.origin.x) / self.dir.x;
        if tmin > tmax {
            collision_dir = Dir::XPos;
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut ydir = Dir::YNeg;
        let mut tymin = (aabb.min.y - self.origin.y) / self.dir.y;
        let mut tymax = (aabb.max.y - self.origin.y) / self.dir.y;
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
        let mut tzmin = (aabb.min.z - self.origin.z) / self.dir.z;
        let mut tzmax = (aabb.max.z - self.origin.z) / self.dir.z;
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

        RayAabbResult::Intersect {
            dist: tmin,
            pos: self.origin + self.dir * tmin,
            dir: collision_dir,
        }
    }

    pub fn block_iter(&self, block_size: f32) -> impl Iterator<Item = BlockPos> + '_ {
        RayBlockIter::new(self, block_size, None)
    }

    /// iterator only nth
    pub fn block_iter_nth(&self, block_size: f32, nth: i32) -> impl Iterator<Item = BlockPos> + '_ {
        RayBlockIter::new(self, block_size, Some(nth))
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
#[must_use]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RayAabbResult {
    Inside,
    NotIntersect,
    Intersect { dist: f32, pos: Vec3, dir: Dir },
}

/// fast ray voxel intersection iterator
/// https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.42.3443&rep=rep1&type=pdf
#[derive(Debug)]
pub struct RayBlockIter<'a> {
    _ray: &'a Ray,
    nth: Option<i32>,
    cur_pos: BlockPos,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    delta_x: f32,
    delta_y: f32,
    delta_z: f32,
    step_x: i32,
    step_y: i32,
    step_z: i32,
}

impl<'a> RayBlockIter<'a> {
    pub fn new(ray: &'a Ray, block_size: f32, nth: Option<i32>) -> Self {
        let cur_pos = BlockPos::from_vec3(&ray.origin, block_size);

        // step
        let step_x = ray.dir.x.signum() as i32;
        let step_y = ray.dir.y.signum() as i32;
        let step_z = ray.dir.z.signum() as i32;

        let max_x = if step_x > 0 {
            ((cur_pos.x as f32 * block_size + block_size) - ray.origin.x) / ray.dir.x
        } else {
            ((cur_pos.x as f32 * block_size) - ray.origin.x) / ray.dir.x
        };

        let max_y = if step_y > 0 {
            ((cur_pos.y as f32 * block_size + block_size) - ray.origin.y) / ray.dir.y
        } else {
            ((cur_pos.y as f32 * block_size) - ray.origin.y) / ray.dir.y
        };

        let max_z = if step_z > 0 {
            ((cur_pos.z as f32 * block_size + block_size) - ray.origin.z) / ray.dir.z
        } else {
            ((cur_pos.z as f32 * block_size) - ray.origin.z) / ray.dir.z
        };

        // delta
        let delta_x = block_size / ray.dir.x.abs();
        let delta_y = block_size / ray.dir.y.abs();
        let delta_z = block_size / ray.dir.z.abs();

        Self {
            _ray: ray,
            nth,
            cur_pos,
            max_x,
            max_y,
            max_z,
            delta_x,
            delta_y,
            delta_z,
            step_x,
            step_y,
            step_z,
        }
    }
}

impl<'a> Iterator for RayBlockIter<'a> {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(nth) = self.nth {
            if nth < 1 {
                return None;
            }

            self.nth = Some(nth - 1);
        }

        enum Xyz {
            X,
            Y,
            Z,
        }
        // get smallest one
        // 0: x, 1: y, 2: z
        let smallest = if self.max_x < self.max_y {
            if self.max_x < self.max_z {
                Xyz::X
            } else {
                Xyz::Z
            }
        } else if self.max_y < self.max_z {
            Xyz::Y
        } else {
            Xyz::Z
        };

        match smallest {
            Xyz::X => {
                self.cur_pos.x += self.step_x;
                self.max_x += self.delta_x;
            }
            Xyz::Y => {
                self.cur_pos.y += self.step_y;
                self.max_y += self.delta_y;
            }
            Xyz::Z => {
                self.cur_pos.z += self.step_z;
                self.max_z += self.delta_z;
            }
        }

        Some(self.cur_pos)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn check_aabb() {
        // z negative
        let ray = Ray::from_values(&Vec3::new(5.0, 5.0, -10.0), &Vec3::Z);
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::Intersect { .. }));
        if let RayAabbResult::Intersect { dist, pos, dir } = result {
            assert!(pos.abs_diff_eq(Vec3::new(5.0, 5.0, 0.0), 0.01));
            assert_abs_diff_eq!(dist, 10.0);
            assert_eq!(dir, Dir::ZNeg);
        }

        // z positive
        let ray = Ray::from_values(&Vec3::new(5.0, 5.0, 20.0), &-Vec3::Z);
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::Intersect { .. }));
        if let RayAabbResult::Intersect { dist, pos, dir } = result {
            assert!(pos.abs_diff_eq(Vec3::new(5.0, 5.0, 10.0), 0.01));
            assert_abs_diff_eq!(dist, 10.0);
            assert_eq!(dir, Dir::ZPos);
        }

        // not intersect (reversed dir)
        let ray = Ray::from_values(&Vec3::new(5.0, 5.0, 20.0), &Vec3::Z);
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));

        let result = ray.check_aabb(&aabb);
        assert!(matches!(result, RayAabbResult::NotIntersect));
    }

    #[test]
    fn test_block_iter() {
        let ray = Ray::from_values(&(0.5, 0.5, 0.5).into(), &(0.0, 0.0, 1.0).into());
        let mut iter = ray.block_iter(1.0);
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, 1)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, 2)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, 3)));

        let ray = Ray::from_values(&(0.5, 0.5, 0.5).into(), &(0.0, 0.0, -1.0).into());
        let mut iter = ray.block_iter(1.0);
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -1)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -2)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -3)));

        let ray = Ray::from_values(&(0.5, 0.5, 0.5).into(), &(0.0, 0.0, -1.0).into());
        let mut iter = ray.block_iter(1.0);
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -1)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -2)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -3)));
    }

    #[test]
    fn test_block_iter_nth() {
        let ray = Ray::from_values(&(0.5, 0.5, 0.5).into(), &(0.0, 0.0, -1.0).into());
        let mut iter = ray.block_iter_nth(1.0, 3);
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -1)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -2)));
        assert_eq!(iter.next(), Some(BlockPos::new(0, 0, -3)));
        assert_eq!(iter.next(), None);
    }
}
