use std::borrow::Borrow;

use crate::{get_matrix, Aabb, Mat4, Plane};

#[derive(Debug)]
pub struct Frustum {
    near: Plane,
    far: Plane,
    left: Plane,
    right: Plane,
    top: Plane,
    bottom: Plane,
}

impl Frustum {
    /// construct Frustum from view projection matrix
    pub fn new(vp: &Mat4) -> Self {
        let near = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) + get_matrix(&vp, 3, 1),
            get_matrix(&vp, 4, 2) + get_matrix(&vp, 3, 2),
            get_matrix(&vp, 4, 3) + get_matrix(&vp, 3, 3),
            get_matrix(&vp, 4, 4) + get_matrix(&vp, 3, 4),
        );

        let far = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) - get_matrix(&vp, 3, 1),
            get_matrix(&vp, 4, 2) - get_matrix(&vp, 3, 2),
            get_matrix(&vp, 4, 3) - get_matrix(&vp, 3, 3),
            get_matrix(&vp, 4, 4) - get_matrix(&vp, 3, 4),
        );

        let left = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) + get_matrix(&vp, 1, 1),
            get_matrix(&vp, 4, 2) + get_matrix(&vp, 1, 2),
            get_matrix(&vp, 4, 3) + get_matrix(&vp, 1, 3),
            get_matrix(&vp, 4, 4) + get_matrix(&vp, 1, 4),
        );

        let right = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) - get_matrix(&vp, 1, 1),
            get_matrix(&vp, 4, 2) - get_matrix(&vp, 1, 2),
            get_matrix(&vp, 4, 3) - get_matrix(&vp, 1, 3),
            get_matrix(&vp, 4, 4) - get_matrix(&vp, 1, 4),
        );

        let top = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) - get_matrix(&vp, 2, 1),
            get_matrix(&vp, 4, 2) - get_matrix(&vp, 2, 2),
            get_matrix(&vp, 4, 3) - get_matrix(&vp, 2, 3),
            get_matrix(&vp, 4, 4) - get_matrix(&vp, 2, 4),
        );

        let bottom = Plane::from_unnorm(
            get_matrix(&vp, 4, 1) + get_matrix(&vp, 2, 1),
            get_matrix(&vp, 4, 2) + get_matrix(&vp, 2, 2),
            get_matrix(&vp, 4, 3) + get_matrix(&vp, 2, 3),
            get_matrix(&vp, 4, 4) + get_matrix(&vp, 2, 4),
        );

        Self {
            near,
            far,
            left,
            right,
            top,
            bottom,
        }
    }

    /// culling frustum and aabb
    /// is aabb is inside or intersect with fructum, return true
    /// else false
    pub fn cull_aabb(&self, aabb: impl Borrow<Aabb>) -> bool {
        let aabb = Borrow::<Aabb>::borrow(&aabb);

        if self.near.dist_aabb(aabb) < 0.0 {
            return false;
        }

        if self.far.dist_aabb(aabb) < 0.0 {
            return false;
        }

        if self.left.dist_aabb(aabb) < 0.0 {
            return false;
        }

        if self.right.dist_aabb(aabb) < 0.0 {
            return false;
        }

        if self.top.dist_aabb(aabb) < 0.0 {
            return false;
        }

        if self.bottom.dist_aabb(aabb) < 0.0 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Vec3;

    #[test]
    fn test_cull_aabb() {
        let eye = Vec3::new(0.0, 0.0, 10.0);
        let target = Vec3::new(0.0, 0.0, 20.0);
        let up = Vec3::Y;
        let aspect = 1.0;
        let fovy = 1.4;
        let znear = 1.0;
        let zfar = 100.0;
        let view = Mat4::look_at_lh(eye, target, up);
        let proj = Mat4::perspective_lh(fovy, aspect, znear, zfar);
        let vp = proj * view;
        let frustum = Frustum::new(&vp);

        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, 20.0), Vec3::new(1.0, 1.0, 30.0));
        assert_eq!(frustum.cull_aabb(&aabb), true);

        let aabb = Aabb::new(Vec3::new(-1.0, -1.0, 120.0), Vec3::new(1.0, 1.0, 130.0));
        assert_eq!(frustum.cull_aabb(&aabb), false);
    }
}
