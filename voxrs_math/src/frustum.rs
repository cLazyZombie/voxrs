use std::borrow::Borrow;

use crate::{Aabb, Matrix4, Plane};

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
    pub fn new(vp: &Matrix4) -> Self {
        let near = Plane::from_unnorm(
            vp[(4, 1)] + vp[(3, 1)],
            vp[(4, 2)] + vp[(3, 2)],
            vp[(4, 3)] + vp[(3, 3)],
            vp[(4, 4)] + vp[(3, 4)],
        );

        let far = Plane::from_unnorm(
            vp[(4, 1)] - vp[(3, 1)],
            vp[(4, 2)] - vp[(3, 2)],
            vp[(4, 3)] - vp[(3, 3)],
            vp[(4, 4)] - vp[(3, 4)],
        );

        let left = Plane::from_unnorm(
            vp[(4, 1)] + vp[(1, 1)],
            vp[(4, 2)] + vp[(1, 2)],
            vp[(4, 3)] + vp[(1, 3)],
            vp[(4, 4)] + vp[(1, 4)],
        );

        let right = Plane::from_unnorm(
            vp[(4, 1)] - vp[(1, 1)],
            vp[(4, 2)] - vp[(1, 2)],
            vp[(4, 3)] - vp[(1, 3)],
            vp[(4, 4)] - vp[(1, 4)],
        );

        let top = Plane::from_unnorm(
            vp[(4, 1)] - vp[(2, 1)],
            vp[(4, 2)] - vp[(2, 2)],
            vp[(4, 3)] - vp[(2, 3)],
            vp[(4, 4)] - vp[(2, 4)],
        );

        let bottom = Plane::from_unnorm(
            vp[(4, 1)] + vp[(2, 1)],
            vp[(4, 2)] + vp[(2, 2)],
            vp[(4, 3)] + vp[(2, 3)],
            vp[(4, 4)] + vp[(2, 4)],
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
    use crate::Vector3;

    #[test]
    fn test_cull_aabb() {
        let eye = Vector3::new(0.0, 0.0, 10.0);
        let target = Vector3::new(0.0, 0.0, 20.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let aspect = 1.0;
        let fovy = 1.4;
        let znear = 1.0;
        let zfar = 100.0;
        let view = Matrix4::look_at(&eye, &target, &up);
        let proj = Matrix4::perspective(aspect, fovy, znear, zfar);
        let vp = proj * view;
        let frustum = Frustum::new(&vp);

        let aabb = Aabb::new(Vector3::new(-1.0, -1.0, 20.0), Vector3::new(1.0, 1.0, 30.0));
        assert_eq!(frustum.cull_aabb(&aabb), true);

        let aabb = Aabb::new(Vector3::new(-1.0, -1.0, 120.0), Vector3::new(1.0, 1.0, 130.0));
        assert_eq!(frustum.cull_aabb(&aabb), false);
    }
}