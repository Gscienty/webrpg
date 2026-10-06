use std::sync;

use shared::BlendMode;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};

use crate::{
    Error,
    buffer::{ParticleVertices, RenderBuffer},
    texture::Texture,
};

#[derive(Clone)]
pub struct Particle {
    pub emitter_id: u32,
    pub texture: Option<sync::Arc<Texture>>,
    pub org: shared::math3::Vector3,
    pub vel: shared::math3::Vector3,
    pub accel: shared::math3::Vector3,
    pub tail: shared::math3::Vector3,
    pub quad_right: shared::math3::Vector3,
    pub quad_up: shared::math3::Vector3,
    pub color: [shared::color::Color32; 3],
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
            emitter_id: 0,
            texture: None,
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
            size_value_scale: 1.0,
            size_time_scale: 1.0,
            time: 0.0,
            lifespan: 0.0,
        }
    }
}

pub struct GlobalParticle {
    pub particles: Vec<Vec<Particle>>,

    pub particle_buffer: RenderBuffer,
}

impl GlobalParticle {
    pub fn init(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        Ok(Self {
            particles: Vec::default(),

            particle_buffer: ParticleVertices::create_array_object(gl)?,
        })
    }

    pub fn begin(&mut self, particles: Vec<Particle>) {
        self.particles.push(particles);
    }

    pub fn end(&mut self) {
        self.particles.pop();
    }

    pub fn count_emitter(&self, emitter_id: u32) -> usize {
        if emitter_id == 0 {
            return 0;
        }
        let Some(particles) = self.particles.last() else {
            return 0;
        };

        particles
            .iter()
            .filter(|particle| {
                particle.emitter_id == emitter_id && particle.time <= particle.lifespan
            })
            .count()
    }

    pub fn update(&mut self, delta_time: f32) {
        let frame_time = delta_time / 1000.;

        let Some(particles) = self.particles.last_mut() else {
            return;
        };

        particles.retain_mut(|particle| {
            particle.time += frame_time;
            particle.time <= particle.lifespan
        });
    }
}
