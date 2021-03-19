use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::{self, AddAssign, Neg, Sub, SubAssign},
};

use ops::Add;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Angle {
    radians: f32,
}

impl Angle {
    pub fn new() -> Self {
        Angle { radians: 0.0 }
    }

    pub fn from_radians(radians: f32) -> Self {
        Angle { radians }
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Angle {
            radians: degrees.to_radians(),
        }
    }

    /// clamp -(PI/2) to (PI/2)
    pub fn clamp_half(self) -> Self {
        let radians = self.radians.clamp(-FRAC_PI_2, FRAC_PI_2);
        Self::from_radians(radians)
    }

    /// make radians in [0, 2*PI)
    pub fn normalize(self) -> Self {
        let mut radians = self.radians % (PI * 2.0);
        if radians < 0.0 {
            radians += PI * 2.0;
        }
        Self::from_radians(radians)
    }

    /// make radians in [-PI, PI)
    pub fn normalize_signed(self) -> Self {
        let mut radians = self.normalize();

        if radians.to_radians() >= PI * 2.0 {
            radians = radians - Angle::from_radians(PI * 2.0);
        }
        radians
    }

    pub fn to_radians(self) -> f32 {
        self.radians
    }

    pub fn to_degrees(self) -> f32 {
        self.radians.to_degrees()
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians + rhs.radians,
        }
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.radians += rhs.radians;
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians - rhs.radians,
        }
    }
}

impl SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        self.radians -= rhs.radians
    }
}

impl Neg for Angle {
    type Output = Angle;

    fn neg(self) -> Self::Output {
        Self::Output {
            radians: -self.radians,
        }
    }
}

impl std::fmt::Debug for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}'", self.to_degrees())
    }
}

impl approx::AbsDiffEq for Angle {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        0.001
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        (self.radians - other.radians).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn test_degrees_radians() {
        let a = Angle::from_radians(PI);
        assert_abs_diff_eq!(a.to_radians(), PI);
        assert_abs_diff_eq!(a.to_degrees(), 180.0);
    }

    #[test]
    fn test_clamp_half() {
        let a = Angle::from_radians(FRAC_PI_2 * 0.5);
        assert_abs_diff_eq!(a.to_degrees(), 45.0);

        let a = Angle::from_degrees(100.0);
        assert_abs_diff_eq!(a.clamp_half().to_degrees(), 90.0);

        let a = Angle::from_degrees(-100.0);
        assert_abs_diff_eq!(a.clamp_half().to_degrees(), -90.0);
    }

    #[test]
    fn test_normalize() {
        assert_abs_diff_eq!(
            Angle::from_degrees(90.0).normalize(),
            Angle::from_degrees(90.0)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(360.0).normalize(),
            Angle::from_degrees(0.0)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(370.0).normalize(),
            Angle::from_degrees(10.0)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(730.0).normalize(),
            Angle::from_degrees(10.0)
        );

        assert_abs_diff_eq!(
            Angle::from_degrees(0.0).normalize(),
            Angle::from_degrees(0.0)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(-10.0).normalize(),
            Angle::from_degrees(350.0)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(-370.0).normalize(),
            Angle::from_degrees(350.0)
        );
    }
}
