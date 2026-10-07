use crate::math3::{Quaternion, Vector3};

pub trait Interp {
    fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self;

    fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self;

    fn lerp(left: Self, right: Self, t: f32) -> Self;
}

impl Interp for f32 {
    fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        let inverse_t = 1. - t;
        let inverse_t_pow2 = inverse_t * inverse_t;
        let t_pow2 = t * t;

        a * inverse_t_pow2 * inverse_t
            + b * 3. * t * inverse_t_pow2
            + c * 3. * t_pow2 * inverse_t
            + d * t_pow2 * t
    }

    fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        let t_pow2 = t * t;

        a * (t_pow2 * (2. * t - 3.) + 1.)
            + b * (t_pow2 * (t - 2.) + t)
            + c * t_pow2 * (t - 1.)
            + d * t_pow2 * (3. - 2. * t)
    }

    fn lerp(left: Self, right: Self, t: f32) -> Self {
        left as Self * (1. - t) + right * t
    }
}

impl Interp for u32 {
    fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        f32::bezier(a as f32, b as f32, c as f32, d as f32, t) as Self
    }

    fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        f32::hermite(a as f32, b as f32, c as f32, d as f32, t) as Self
    }

    fn lerp(left: Self, right: Self, t: f32) -> Self {
        f32::lerp(left as f32, right as f32, t) as Self
    }
}

impl Interp for Vector3 {
    fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        Vector3::bezier(a, b, c, d, t)
    }

    fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        Vector3::hermite(a, b, c, d, t)
    }

    fn lerp(left: Self, right: Self, t: f32) -> Self {
        Vector3::lerp(left, right, t)
    }
}

impl Interp for Quaternion {
    fn bezier(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        Quaternion::sqlerp(a, b, c, d, t)
    }

    fn hermite(a: Self, b: Self, c: Self, d: Self, t: f32) -> Self {
        Quaternion::sqlerp(a, b, c, d, t)
    }

    fn lerp(left: Self, right: Self, t: f32) -> Self {
        Quaternion::slerp(left, right, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_scalars_and_vectors() {
        assert_eq!(f32::bezier(0., 10., 20., 30., 0.5), 15.);
        assert_eq!(f32::hermite(0., 10., 20., 30., 0.5), 13.75);
        assert_eq!(u32::bezier(0, 10, 20, 30, 0.5), 15);
        assert_eq!(u32::hermite(0, 10, 20, 30, 0.5), 13);

        let result = <Vector3 as Interp>::bezier(
            (0., 0., 0.).into(),
            (10., 20., 30.).into(),
            (20., 40., 60.).into(),
            (30., 60., 90.).into(),
            0.5,
        );
        assert_eq!(result, (15., 30., 45.).into());
    }
}
