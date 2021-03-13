use crate::Vector3;

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub c: Vector3, // center of sphere
    pub r: f32,     // radius of sphere
}

impl Sphere {
    pub fn new(c: Vector3, r: f32) -> Self {
        assert!(r >= 0.0);

        Self { c, r }
    }

    /// create sphere using projection
    /// reference: https://lxjk.github.io/2017/04/15/Calculate-Minimal-Bounding-Sphere-of-Frustum.html
    pub fn from_proj(near: f32, far: f32, aspect: f32, fov: f32) -> Self {
        let rev_aspect = 1.0 / aspect;
        let k = (1.0 + rev_aspect * rev_aspect).sqrt() * (fov / 2.0).tan();
        let k2 = k * k;

        let fn_fn = (far - near) / (far + near);
        if k2 >= fn_fn {
            let c = Vector3::new(0.0, 0.0, far);
            let r = far * k;
            Sphere::new(c, r)
        } else {
            let cz = 0.5 * (far + near) * (1.0 + k2);
            let c = Vector3::new(0.0, 0.0, cz);
            let r = 0.5
                * ((far - near) * (far - near)
                    + 2.0 * (far * far + near * near) * k2
                    + (far + near) * (far + near) * k2)
                    .sqrt();
            Sphere::new(c, r)
        }
    }

    pub fn fron_view_proj(
        eye: Vector3,
        target: Vector3,
        up: Vector3,
        near: f32,
        far: f32,
        aspect: f32,
        fov: f32,
    ) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create() {
        let center = Vector3::new(1.0, 2.0, 3.0);
        let sp = Sphere::new(center, 10.0);
        assert_eq!(sp.c, center);
        assert_eq!(sp.r, 10.0);
    }

    #[test]
    fn test_from_proj() {
        let sp = Sphere::from_proj(1.0, 10.0, 1.0, (90_f32).to_radians());
        println!("{:?}", sp);
        assert!(sp.c.z() >= 1.0 && sp.c.z() <= 10.0);
        assert!(sp.r <= 20.0);
    }
}
