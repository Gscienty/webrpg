use std::ops::{Div, Mul};

use crate::Vector2;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

impl Mul<f32> for Rect {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            w: self.w * rhs,
            h: self.h * rhs,
        }
    }
}

impl Div<f32> for Rect {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1. / rhs)
    }
}

impl Rect {
    pub fn contains(self, point: Vector2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    pub fn inset(self, inset: f32) -> Self {
        Self {
            x: self.x + inset,
            y: self.y + inset,
            w: self.w - inset * 2.,
            h: self.h - inset * 2.,
        }
    }

    pub fn center(self) -> Vector2 {
        Vector2::from((self.x + self.w * 0.5, self.y + self.h * 0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_scales_insets_and_centers() {
        let rect = Rect {
            x: 2.,
            y: 4.,
            w: 8.,
            h: 6.,
        };

        assert!(rect.contains(Vector2::from((2., 10.))));
        assert!(!rect.contains(Vector2::from((10.1, 7.))));
        assert_eq!((rect * 2.) / 2., rect);
        assert_eq!(
            rect.inset(1.),
            Rect {
                x: 3.,
                y: 5.,
                w: 6.,
                h: 4.,
            }
        );
        assert_eq!(rect.center(), Vector2::from((6., 7.)));
    }
}
