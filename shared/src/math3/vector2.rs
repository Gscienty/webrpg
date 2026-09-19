use std::ops::{Add, Index, Mul, Neg, Sub};

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Vector2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl From<(f32, f32)> for Vector2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => unreachable!(),
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vector2> for Vector2 {
    type Output = f32;

    fn mul(self, rhs: Vector2) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Self;

    fn add(self, rhs: Vector2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Vector2) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Vector2 {
    pub fn length_sq(self) -> f32 {
        self * self
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

    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        self * (1. - t) + rhs * t
    }

    pub fn mad(self, rhs: Self, s: f32) -> Self {
        self + rhs * s
    }
}
