use std::ops::{Add, Index, Mul, Neg, Sub};

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Vector3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

impl Index<usize> for Vector3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => unreachable!(),
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Add<Vector3> for Vector3 {
    type Output = Self;

    fn add(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Vector3 {
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).length()
    }

    pub fn normalize(self) -> Self {
        self * (1. / self.length())
    }

    pub fn mad(self, rhs: Self, s: f32) -> Self {
        self + rhs * s
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn rotate(self, axis: Self, radians: f32) -> Self {
        let c = radians.cos();
        let s = radians.sin();

        let n = axis.normalize();
        let d = self.dot(n);
        let cross = n.cross(self);

        self * c + cross * s + n * (d * (1. - c))
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
            z: self.z.clamp(min, max),
        }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        a * (1. - t) + b * t
    }

    pub fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        let inverse_t = 1. - t;

        let inverse_t_pow2 = inverse_t.powi(2);
        let t_pow2 = t.powi(2);

        let factor_1 = inverse_t_pow2 * inverse_t;
        let factor_2 = 3. * t * inverse_t_pow2;
        let factor_3 = 3. * t_pow2 * inverse_t;
        let factor_4 = t_pow2 * t;

        a * factor_1 + b * factor_2 + c * factor_3 + d * factor_4
    }

    pub fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        let t_pow2 = t.powi(2);

        let factor_1 = t_pow2 * (2. * t - 3.) + 1.;
        let factor_2 = t_pow2 * (t - 2.) + t;
        let factor_3 = t_pow2 * (t - 1.);
        let factor_4 = t_pow2 * (3. - 2. * t);

        a * factor_1 + b * factor_2 + c * factor_3 + d * factor_4
    }
}
