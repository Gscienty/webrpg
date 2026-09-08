use std::ops::{Add, Mul, Neg};

use crate::{Matrix4, Vector3};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Quaternion {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) w: f32,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RotationOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::from((0., 0., 0., 1.))
    }
}

impl From<(f32, f32, f32, f32)> for Quaternion {
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self {
        Self { x, y, z, w }
    }
}

impl Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from((-self.x, -self.y, -self.z, -self.w))
    }
}

impl Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from((self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs))
    }
}

impl Add<Quaternion> for Quaternion {
    type Output = Self;

    fn add(self, rhs: Quaternion) -> Self::Output {
        Self::from((
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        ))
    }
}

impl Quaternion {
    pub fn dot(self, right: Self) -> f32 {
        self.x * right.x + self.y * right.y + self.z * right.z + self.w * right.w
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn from_euler(euler: Vector3, order: RotationOrder) -> Self {
        let x = Self::axis(euler.x, 0);
        let y = Self::axis(euler.y, 1);
        let z = Self::axis(euler.z, 2);

        match order {
            RotationOrder::XYZ => x.multiply(y).multiply(z),
            RotationOrder::XZY => x.multiply(z).multiply(y),
            RotationOrder::YXZ => y.multiply(x).multiply(z),
            RotationOrder::YZX => y.multiply(z).multiply(x),
            RotationOrder::ZXY => z.multiply(x).multiply(y),
            RotationOrder::ZYX => z.multiply(y).multiply(x),
        }
    }

    pub fn normalized(self) -> Self {
        self * (1. / self.length())
    }

    pub fn from_matrix(mat: Matrix4) -> Self {
        let m = mat.v;
        let trace = m[0] + m[5] + m[10];
        let result = if trace > 0. {
            let s = (trace + 1.).sqrt() * 2.;
            Self::from((
                (m[6] - m[9]) / s,
                (m[8] - m[2]) / s,
                (m[1] - m[4]) / s,
                s * 0.25,
            ))
        } else if m[0] > m[5] && m[0] > m[10] {
            let s = (1. + m[0] - m[5] - m[10]).sqrt() * 2.;
            Self::from((
                s * 0.25,
                (m[4] + m[1]) / s,
                (m[8] + m[2]) / s,
                (m[6] - m[9]) / s,
            ))
        } else if m[5] > m[10] {
            let s = (1. + m[5] - m[0] - m[10]).sqrt() * 2.;
            Self::from((
                (m[4] + m[1]) / s,
                s * 0.25,
                (m[9] + m[6]) / s,
                (m[8] - m[2]) / s,
            ))
        } else {
            let s = (1. + m[10] - m[0] - m[5]).sqrt() * 2.;
            Self::from((
                (m[8] + m[2]) / s,
                (m[9] + m[6]) / s,
                s * 0.25,
                (m[1] - m[4]) / s,
            ))
        };
        result.normalized()
    }

    pub fn slerp(lhs: Self, mut rhs: Self, t: f32) -> Self {
        let mut cosine = lhs.dot(rhs);
        if cosine < 0. {
            rhs = -rhs;
            cosine = -cosine;
        }

        let (scale_p, scale_q) = if cosine < 1. - 1e-6 {
            let omega = cosine.acos();
            let sine = omega.sin();

            (((1. - t) * omega).sin() / sine, (t * omega).sin() / sine)
        } else {
            (1. - t, t)
        };

        lhs * scale_p + rhs * scale_q
    }

    pub fn sqlerp(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        let first = Self::slerp(a, d, t);
        let second = Self::slerp(b, c, t);
        Self::slerp(first, second, 2. * t * (1. - t))
    }

    fn axis(radians: f32, axis: usize) -> Self {
        let (s, c) = (radians * 0.5).sin_cos();
        match axis {
            0 => Self::from((s, 0., 0., c)),
            1 => Self::from((0., s, 0., c)),
            _ => Self::from((0., 0., s, c)),
        }
    }

    fn multiply(self, right: Self) -> Self {
        Self::from((
            self.w * right.x + self.x * right.w + self.y * right.z - self.z * right.y,
            self.w * right.y - self.x * right.z + self.y * right.w + self.z * right.x,
            self.w * right.z + self.x * right.y - self.y * right.x + self.z * right.w,
            self.w * right.w - self.x * right.x - self.y * right.y - self.z * right.z,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equivalent(left: Quaternion, right: Quaternion) -> bool {
        left.dot(right).abs() > 1. - 1e-5
    }

    #[test]
    fn converts_euler_and_matrix() {
        let q = Quaternion::from_euler(
            Vector3::from((0., 0., std::f32::consts::FRAC_PI_2)),
            RotationOrder::XYZ,
        );
        let matrix = Matrix4::from([
            0., 1., 0., 0., -1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
        ]);
        assert!(equivalent(q, Quaternion::from_matrix(matrix)));
    }

    #[test]
    fn interpolates_shortest_path() {
        let identity = Quaternion::default();
        let half_turn = Quaternion::from((0., 0., 1., 0.));
        let halfway = Quaternion::slerp(identity, half_turn, 0.5);
        let expected = Quaternion::from_euler(
            Vector3::from((0., 0., std::f32::consts::FRAC_PI_2)),
            RotationOrder::XYZ,
        );
        assert!(equivalent(halfway, expected));
        assert!(equivalent(
            Quaternion::sqlerp(identity, identity, half_turn, half_turn, 0.5),
            expected
        ));
    }
}
