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
