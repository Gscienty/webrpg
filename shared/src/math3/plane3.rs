use std::ops::Mul;

use super::Vector3;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Plane3 {
    pub(crate) normal: Vector3,
    pub(crate) distance: f32,
}

impl Mul<Vector3> for Plane3 {
    type Output = f32;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Vector3) -> Self::Output {
        self.normal.dot(rhs) + self.distance
    }
}

impl Plane3 {
    pub fn normalize(&self) -> Self {
        let magnitude = self.normal.length();

        Self {
            normal: self.normal.normalize(),
            distance: self.distance / magnitude,
        }
    }
}
