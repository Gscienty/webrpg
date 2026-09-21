pub fn lerp_number(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}
