use shared::{BlendMode, color, math3};

use crate::texture;

#[derive(Clone)]
pub struct Particle {
    pub texture: Option<texture::Texture>,
    pub emitter_id: u32,
    pub org: math3::Vector3,
    pub vel: math3::Vector3,
    pub accel: math3::Vector3,
    pub tail: math3::Vector3,
    pub quad_right: math3::Vector3,
    pub quad_up: math3::Vector3,
    pub color: [color::Color32; 3],
    pub size: [u8; 3],
    pub midtime: u8,
    pub columns: u8,
    pub rows: u8,
    pub blend_mode: BlendMode,
    pub uv_start: u16,
    pub uv_mid: u16,
    pub uv_end: u16,
    pub use_uv_curve: bool,
    pub size_value_scale: f32,
    pub size_time_scale: f32,
    pub time: f32,
    pub lifespan: f32,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            texture: None,
            emitter_id: 0,
            org: Default::default(),
            vel: Default::default(),
            accel: Default::default(),
            tail: Default::default(),
            quad_right: Default::default(),
            quad_up: Default::default(),
            color: Default::default(),
            size: Default::default(),
            midtime: 0,
            columns: 0,
            rows: 0,
            blend_mode: BlendMode::Add,
            uv_start: 0,
            uv_mid: 0,
            uv_end: 0,
            use_uv_curve: false,
            size_value_scale: 1.,
            size_time_scale: 1.,
            time: 0.,
            lifespan: 0.,
        }
    }
}

impl Particle {
    pub fn blend_color(&self) -> color::Color32 {
        color::blend_color(
            &self.color,
            self.time / self.lifespan,
            self.midtime as f32 / 255.,
        )
    }

    pub fn blend_size(&self) -> f32 {
        let k = self.time * self.size_time_scale;
        let midtime = self.midtime as f32 / 255.;
        let value = if k > midtime {
            math3::lerp_number(
                self.size[1] as f32,
                self.size[2] as f32,
                (k - midtime) / (1. - midtime),
            )
        } else {
            math3::lerp_number(self.size[0] as f32, self.size[1] as f32, k / midtime)
        };

        self.size_value_scale * value
    }

    pub fn get_frame(&self) -> color::Color32 {
        let columns = u32::from(self.columns.max(1));
        let rows = u32::from(self.rows.max(1));
        let total = columns * rows;
        let k = if self.lifespan > 0. {
            self.time / self.lifespan
        } else {
            0.
        }
        .clamp(0., 1.);
        let mut frame = if self.use_uv_curve {
            let mid = self.midtime as f32 / 255.;
            let value = if k > mid {
                math3::lerp_number(
                    self.uv_mid as f32,
                    self.uv_end as f32,
                    (k - mid) / (1. - mid).max(0.0001),
                )
            } else {
                math3::lerp_number(
                    self.uv_start as f32,
                    self.uv_mid as f32,
                    k / mid.max(0.0001),
                )
            };
            value.max(0.) as u32
        } else {
            (k * total as f32) as u32
        };
        if frame >= total {
            frame = total - 1;
        }

        let u = frame % columns;
        let v = frame / columns;
        let width = 256 / columns;
        let height = 256 / rows;
        color::Color32 {
            red: (width * u) as u8,
            green: (height * (v + 1) - 1) as u8,
            blue: (width * (u + 1) - 1) as u8,
            alpha: (height * v) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blends_particle_color_across_midtime() {
        let particle = Particle {
            color: [
                color::Color32::black(),
                color::Color32 {
                    red: 100,
                    green: 100,
                    blue: 100,
                    alpha: 255,
                },
                color::Color32::white(),
            ],
            time: 0.5,
            lifespan: 2.,
            midtime: 128,
            ..Default::default()
        };

        let result = particle.blend_color();

        assert_eq!(result.red, 49);
        assert_eq!(result.alpha, 255);
    }

    #[test]
    fn blends_encoded_particle_size_across_midtime() {
        let particle = Particle {
            size: [0, 100, 200],
            size_value_scale: 0.5,
            size_time_scale: 0.5,
            time: 1.5,
            midtime: 128,
            ..Default::default()
        };

        let result = particle.blend_size();

        assert!((result - 74.90157).abs() < 0.0001);
    }

    #[test]
    fn gets_lifetime_animated_frame_with_flipped_v() {
        let particle = Particle {
            columns: 2,
            rows: 2,
            time: 0.5,
            lifespan: 1.,
            ..Default::default()
        };

        assert_eq!(
            particle.get_frame(),
            color::Color32 {
                red: 0,
                green: 255,
                blue: 127,
                alpha: 128,
            }
        );
    }

    #[test]
    fn gets_frame_from_uv_curve() {
        let particle = Particle {
            columns: 2,
            rows: 2,
            lifespan: 1.,
            use_uv_curve: true,
            uv_start: 1,
            uv_mid: 2,
            uv_end: 3,
            midtime: 128,
            ..Default::default()
        };

        assert_eq!(
            particle.get_frame(),
            color::Color32 {
                red: 128,
                green: 127,
                blue: 255,
                alpha: 0,
            }
        );
    }
}
