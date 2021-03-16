use std::{borrow::Borrow, fmt::Display};

use nalgebra_glm as glm;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector3 {
    pub(crate) v: glm::Vec3,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            v: glm::Vec3::new(x, y, z),
        }
    }

    pub fn x(&self) -> f32 {
        self.v[0]
    }

    pub fn y(&self) -> f32 {
        self.v[1]
    }

    pub fn z(&self) -> f32 {
        self.v[2]
    }

    pub fn as_slice(&self) -> &[f32] {
        self.v.as_slice()
    }

    pub fn to_array(&self) -> [f32; 3] {
        use std::convert::TryInto;
        self.v.as_slice().try_into().unwrap()
    }

    /// warning: math 내부에서만 사용 가능
    pub fn get_inner(&self) -> &glm::Vec3 {
        &self.v
    }

    pub fn dot(lhs: impl Borrow<Vector3>, rhs: impl Borrow<Vector3>) -> f32 {
        let lhs = Borrow::<Vector3>::borrow(&lhs);
        let rhs = Borrow::<Vector3>::borrow(&rhs);

        lhs.v.dot(&rhs.v)
    }

    pub fn get_normalized(v: impl Borrow<Vector3>) -> Self {
        let v = Borrow::<Vector3>::borrow(&v);
        
        Self {
            v: v.v.normalize()
        }
    }

    pub fn magnitude(&self) -> f32 {
        self.v.magnitude()
    }
}

impl std::ops::Add for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output { v: self.v + rhs.v }
    }
}

impl std::ops::Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output { v: self.v - rhs.v }
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output { v: self.v * rhs }
    }
}

impl std::ops::AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.v += rhs.v;
    }
}

impl From<&[f32]> for Vector3 {
    fn from(slice: &[f32]) -> Self {
        assert!(slice.len() == 3);
        Self::new(slice[0], slice[1], slice[2])
    }
}

impl From<&[f32; 3]> for Vector3 {
    fn from(array: &[f32; 3]) -> Self {
        Self::new(array[0], array[1], array[2])
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.v[0], self.v[1], self.v[2])
    }
}

#[cfg(test)]
impl approx::AbsDiffEq for Vector3 
{
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.v[0], &other.v[0], epsilon) &&
        f32::abs_diff_eq(&self.v[1], &other.v[1], epsilon) &&
        f32::abs_diff_eq(&self.v[2], &other.v[2], epsilon) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn as_slice() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn to_array() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.to_array(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn add() {
        let mut v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);

        let v3 = v1 + v2;
        assert_eq!(v3.as_slice(), &[5.0, 7.0, 9.0]);

        v1 += v2;
        assert_eq!(v1.as_slice(), &[5.0, 7.0, 9.0]);
    }

    #[test]
    fn from_array() {
        let array: [f32; 3] = [1.0, 2.0, 3.0];
        let v: Vector3 = (&array).into();
        assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn from_slice() {
        let slice: &[f32] = &[1.0, 2.0, 3.0];
        let v: Vector3 = slice.into();
        assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn mul_to_f32() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let v2: Vector3 = v * 3.0;
        assert_eq!(v2.as_slice(), &[3.0, 6.0, 9.0]);
    }
}
