pub fn lerp_number(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}

pub fn blend_float(values: &[u8; 3], k: f32, midtime: f32) -> f32 {
    if k > midtime {
        lerp_number(
            values[1] as f32,
            values[2] as f32,
            (k - midtime) / (1. - midtime),
        )
    } else {
        lerp_number(values[0] as f32, values[1] as f32, k / midtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends_float_across_midtime() {
        let values = [0, 100, 200];
        assert_eq!(blend_float(&values, 0.125, 0.25), 50.);
        assert_eq!(blend_float(&values, 0.625, 0.25), 150.);
    }
}
