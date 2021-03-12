use nalgebra_glm as glm;

use crate::Vector3;

#[derive(Copy, Clone, Debug)]
pub struct Matrix4 {
    m: glm::Mat4,
}

impl Matrix4 {
    pub fn identity() -> Self {
        Self { m: glm::identity() }
    }

    #[rustfmt::skip]
    pub fn new( m11: f32, m12: f32, m13: f32, m14: f32,
                m21: f32, m22: f32, m23: f32, m24: f32,
                m31: f32, m32: f32, m33: f32, m34: f32,
                m41: f32, m42: f32, m43: f32, m44: f32,
    ) -> Self {
        Self {
            m: glm::Mat4::new(
                m11, m12, m13, m14,
                m21, m22, m23, m24,
                m31, m32, m33, m34,
                m41, m42, m43, m44,
            ),
        }
    }

    // to column oriented slice
    pub fn as_slice(&self) -> &[f32] {
        self.m.as_slice()
    }

    // to column oriented array
    pub fn to_array(&self) -> [f32; 16] {
        use std::convert::TryInto;

        self.m.as_slice().try_into().unwrap()
    }

    /// create look at (left hand) matrix
    pub fn look_at(eye: &Vector3, target: &Vector3, up: &Vector3) -> Self {
        Self {
            m: glm::look_at_lh(eye.get_inner(), target.get_inner(), up.get_inner()),
        }
    }

    pub fn perspective(aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        Self {
            m: glm::perspective_lh_zo(aspect, fovy, near, far),
        }
    }

    #[rustfmt::skip]
    pub fn translate(v: &Vector3) -> Self {
        Self::new(
            1.0, 0.0, 0.0, v.x(),
            0.0, 1.0, 0.0, v.y(),
            0.0, 0.0, 1.0, v.z(),
            0.0, 0.0, 0.0, 1.0,
        )
    }

    #[rustfmt::skip]
    pub fn uniform_scale(s: f32) -> Self {
        Self::new(
            s, 0.0, 0.0, 0.0, 
            0.0, s, 0.0, 0.0,
            0.0, 0.0, s, 0.0, 
            0.0, 0.0, 0.0, 1.0
        )
    }

    /// get inverse matrix
    /// panic if matrix is not invertable
    pub fn inverse(&self) -> Self {
        let inverted = self.m.try_inverse().unwrap();
        Self {
            m: inverted,
        }
    }
}

impl std::ops::Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        assert!(index.0 > 0 && index.0 <= 4);
        assert!(index.1 > 0 && index.1 <= 4);

        self.m.index((index.0 - 1) + (index.1 - 1) * 4)
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output { m: self.m * rhs.m }
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::new(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        )
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let m = Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0,
        );

        for c in 0..4_usize {
            for r in 0..4_usize {
                assert_eq!(m[(r+1, c+1)], (r * 4 + (c + 1)) as f32);
            }
        }
    }

    #[test]
    fn create_identity_matrix() {
        let m = Matrix4::identity();
        assert_eq!(m[(1, 1)], 1.0);
        assert_eq!(m[(2, 2)], 1.0);
        assert_eq!(m[(3, 3)], 1.0);
        assert_eq!(m[(4, 4)], 1.0);
    }

    #[test]
    fn as_slice() {
        let m = Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0,
        );

        let s: &[f32] = m.as_slice();
        assert_eq!(
            s,
            &[
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );
    }

    #[test]
    fn to_array() {
        let m = Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0,
        );

        let s = m.to_array();
        assert_eq!(
            s,
            [
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );
    }

    #[test]
    fn multiply_matrix_to_matrix() {
        let m1 = Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0,
        );

        let m2 = Matrix4::new(
            2.0, 0.0, 0.0, 0.0, 
            0.0, 2.0, 0.0, 0.0, 
            0.0, 0.0, 2.0, 0.0, 
            0.0, 0.0, 0.0, 2.0,
        );

        let m3 = m1 * m2;
        assert_eq!(
            m3.as_slice(),
            Matrix4::new(
                2.0, 4.0, 6.0, 8.0, 
                10.0, 12.0, 14.0, 16.0, 
                18.0, 20.0, 22.0, 24.0, 
                26.0, 28.0, 30.0, 32.0,
            )
            .as_slice()
        );
    }

    #[test]
    #[should_panic]
    fn test_out_of_index() {
        let m = Matrix4::new(
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0, 
        );

        let _ = m[(0, 0)];
    }
}
