use std::ops::{Add, Mul, Neg};

use super::{Matrix4, Vector3};

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Orientation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
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

impl Default for Orientation {
    fn default() -> Self {
        Self::from((0., 0., 0.))
    }
}

impl From<(f32, f32, f32, f32)> for Quaternion {
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self {
        Self { x, y, z, w }
    }
}

impl From<(f32, f32, f32)> for Orientation {
    fn from((yaw, pitch, roll): (f32, f32, f32)) -> Self {
        Self { yaw, pitch, roll }
    }
}

impl From<&Orientation> for Quaternion {
    fn from(value: &Orientation) -> Self {
        let (sy, cy) = (value.yaw * 0.5).sin_cos();
        let (sp, cp) = (-value.pitch * 0.5).sin_cos();
        let (sr, cr) = (value.roll * 0.5).sin_cos();

        Self::from((
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
            cr * cp * cy + sr * sp * sy,
        ))
    }
}

impl From<Matrix4> for Quaternion {
    fn from(mat: Matrix4) -> Self {
        let trace = mat[0][0] + mat[1][1] + mat[2][2];
        let result = if trace > 0. {
            let s = (trace + 1.).sqrt() * 2.;
            Self::from((
                (mat[1][2] - mat[2][1]) / s,
                (mat[2][0] - mat[0][2]) / s,
                (mat[0][1] - mat[1][0]) / s,
                s * 0.25,
            ))
        } else if mat[0][0] > mat[1][1] && mat[0][0] > mat[2][2] {
            let s = (1. + mat[0][0] - mat[1][1] - mat[2][2]).sqrt() * 2.;
            Self::from((
                s * 0.25,
                (mat[1][0] + mat[0][1]) / s,
                (mat[2][0] + mat[0][2]) / s,
                (mat[1][2] - mat[2][1]) / s,
            ))
        } else if mat[1][1] > mat[2][2] {
            let s = (1. + mat[1][1] - mat[0][0] - mat[2][2]).sqrt() * 2.;
            Self::from((
                (mat[1][0] + mat[0][1]) / s,
                s * 0.25,
                (mat[2][1] + mat[1][2]) / s,
                (mat[2][0] - mat[0][2]) / s,
            ))
        } else {
            let s = (1. + mat[2][2] - mat[0][0] - mat[1][1]).sqrt() * 2.;
            Self::from((
                (mat[2][0] + mat[0][2]) / s,
                (mat[2][1] + mat[1][2]) / s,
                s * 0.25,
                (mat[0][1] - mat[1][0]) / s,
            ))
        };
        result.normalized()
    }
}

impl Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from((-self.x, -self.y, -self.z, self.w)).normalized()
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
    pub fn from_euler(euler: Vector3, order: RotationOrder) -> Self {
        Self::from(Matrix4::identity().rotate(euler, order))
    }

    pub fn dot(&self, right: Self) -> f32 {
        self.x * right.x + self.y * right.y + self.z * right.z + self.w * right.w
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn slerp(lhs: Self, mut rhs: Self, t: f32) -> Self {
        let mut cosine = lhs.dot(rhs);
        if cosine < 0. {
            rhs = rhs * -1.;
            cosine = -cosine;
        }

        let (scale_p, scale_q) = if cosine < 1. - f32::EPSILON {
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

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length < f32::EPSILON {
            return self;
        }
        self * (1. / length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slerp_treats_opposite_quaternion_signs_as_the_same_rotation() {
        let rotation = Quaternion::from((0., 0., 0., 1.));
        let same_rotation = Quaternion::from((0., 0., 0., -1.));

        assert_eq!(Quaternion::slerp(rotation, same_rotation, 0.5), rotation);
    }
}
