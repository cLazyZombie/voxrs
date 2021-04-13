use crate::{Aabb, Mat4, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub c: Vec3, // center of sphere
    pub r: f32,  // radius of sphere
}

impl Sphere {
    pub fn new(c: Vec3, r: f32) -> Self {
        assert!(r >= 0.0);

        Self { c, r }
    }

    /// create sphere using projection
    /// reference: <https://lxjk.github.io/2017/04/15/Calculate-Minimal-Bounding-Sphere-of-Frustum.html/>
    pub fn from_proj(near: f32, far: f32, aspect: f32, fov: f32) -> Self {
        let rev_aspect = 1.0 / aspect;
        let k = (1.0 + rev_aspect * rev_aspect).sqrt() * (fov / 2.0).tan();
        let k2 = k * k;

        let fn_fn = (far - near) / (far + near);
        if k2 >= fn_fn {
            let c = Vec3::new(0.0, 0.0, far);
            let r = far * k;
            Sphere::new(c, r)
        } else {
            let cz = 0.5 * (far + near) * (1.0 + k2);
            let c = Vec3::new(0.0, 0.0, cz);
            let r = 0.5
                * ((far - near) * (far - near)
                    + 2.0 * (far * far + near * near) * k2
                    + (far + near) * (far + near) * k2)
                    .sqrt();
            Sphere::new(c, r)
        }
    }

    pub fn from_view_proj(
        eye: &Vec3,
        target: &Vec3,
        up: &Vec3,
        near: f32,
        far: f32,
        aspect: f32,
        fov: f32,
    ) -> Self {
        let sphere = Sphere::from_proj(near, far, aspect, fov);

        let view = Mat4::look_at_lh(*eye, *target, *up);
        let inv_view = view.inverse();

        let center = inv_view.transform_point3(sphere.c);

        Self {
            c: center,
            r: sphere.r,
        }
    }

    /// sphere aabb intersection
    /// reference: <https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection>
    pub fn intersect_aabb(&self, aabb: &Aabb) -> bool {
        let x = f32::max(aabb.min.x, f32::min(self.c.x, aabb.max.x));
        let y = f32::max(aabb.min.y, f32::min(self.c.y, aabb.max.y));
        let z = f32::max(aabb.min.z, f32::min(self.c.z, aabb.max.z));

        let distance = f32::sqrt(
            (x - self.c.x) * (x - self.c.x)
                + (y - self.c.y) * (y - self.c.y)
                + (z - self.c.z) * (z - self.c.z),
        );

        distance < self.r
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::*;

    #[test]
    fn test_create() {
        let center = Vec3::new(1.0, 2.0, 3.0);
        let sp = Sphere::new(center, 10.0);
        sp.c.abs_diff_eq(center, 0.01);
        assert_abs_diff_eq!(sp.r, 10.0);
    }

    #[test]
    fn test_from_proj() {
        let sp = Sphere::from_proj(1.0, 10.0, 1.0, (90_f32).to_radians());
        println!("{:?}", sp);
        assert!(sp.c.z >= 1.0 && sp.c.z <= 10.0);
        assert!(sp.r <= 20.0);
    }

    #[test]
    fn test_intersect_aabb() {
        let sp = Sphere::new(Vec3::new(10.0, 10.0, 10.0), 90.0);

        // fully inside
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(20.0, 20.0, 20.0));
        assert_eq!(sp.intersect_aabb(&aabb), true);

        // between
        let aabb = Aabb::new(Vec3::new(50.0, 50.0, 50.0), Vec3::new(120.0, 120.0, 120.0));
        assert_eq!(sp.intersect_aabb(&aabb), true);

        // outside
        let aabb = Aabb::new(
            Vec3::new(110.0, 110.0, 110.0),
            Vec3::new(120.0, 120.0, 120.0),
        );
        assert_eq!(sp.intersect_aabb(&aabb), false);
    }
}
