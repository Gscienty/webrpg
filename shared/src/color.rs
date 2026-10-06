#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Color32 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color32 {
    pub fn white() -> Self {
        Self {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }
    }
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let lerp = |a, b| crate::math3::lerp_number(a as f32, b as f32, t) as u8;
    Color32 {
        red: lerp(a.red, b.red),
        green: lerp(a.green, b.green),
        blue: lerp(a.blue, b.blue),
        alpha: lerp(a.alpha, b.alpha),
    }
}

pub fn blend_color(values: &[Color32; 3], k: f32, midtime: f32) -> Color32 {
    if k > midtime {
        lerp_color(values[1], values[2], (k - midtime) / (1. - midtime))
    } else {
        lerp_color(values[0], values[1], k / midtime)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerps_color_channels() {
        let a = Color32 {
            red: 0,
            green: 20,
            blue: 100,
            alpha: 255,
        };
        let b = Color32 {
            red: 200,
            green: 40,
            blue: 200,
            alpha: 55,
        };

        assert_eq!(lerp_color(a, b, 0.), a);
        assert_eq!(lerp_color(a, b, 1.), b);
        assert_eq!(
            lerp_color(a, b, 0.5),
            Color32 {
                red: 100,
                green: 30,
                blue: 150,
                alpha: 155,
            }
        );
    }

    #[test]
    fn blends_color_across_midtime() {
        let values = [
            Color32::default(),
            Color32 {
                red: 100,
                green: 100,
                blue: 100,
                alpha: 100,
            },
            Color32 {
                red: 200,
                green: 200,
                blue: 200,
                alpha: 200,
            },
        ];

        assert_eq!(blend_color(&values, 0.125, 0.25).red, 50);
        assert_eq!(blend_color(&values, 0.625, 0.25).red, 150);
    }
}
