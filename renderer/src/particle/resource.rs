use std::collections::BTreeMap;

use fileformat::mdx;
use shared::{BlendMode, math3};

use crate::{
    particle::{Particle, utils::generate_random_direction},
    texture_set::TextureSet,
};

const PARTICLE_HEAD: u32 = 0;
const PARTICLE_BOTH: u32 = 2;
const NODE_XY_QUAD: u32 = 0x10_0000;
const EMIT_BUDGET: u32 = 256;

#[derive(Default)]
pub struct Resource {
    pub particles: Vec<Particle>,

    accumulators: BTreeMap<(usize, u32), f64>,
}

impl Resource {
    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        &mut self,
        model: &mdx::Model,
        emitter: &mdx::ParticleEmitter2,
        textures: &[usize],
        texture_set: &TextureSet,
        model_matrix: math3::Matrix4,
        node_matrices: &BTreeMap<u32, math3::Matrix4>,
        team_id: u32,
        global_time: u32,
        frame: u32,
        delta_ms: u32,
    ) {
        let Some(node_matrix) = node_matrices.get(&emitter.node.object_id) else {
            return;
        };

        let rate = emitter.get_emission_rate(global_time, frame);
        let key = (model as *const mdx::Model as usize, emitter.node.object_id);
        let emitted = emit_count(rate, self.accumulators.entry(key).or_default(), delta_ms);
        if emitted == 0 {
            return;
        }

        let matrix = model_matrix * *node_matrix;
        let speed = emitter.get_speed(global_time, frame);
        let variation = emitter.get_variation(global_time, frame);
        let latitude = emitter.get_latitude(global_time, frame);
        let gravity = emitter.get_gravity(global_time, frame);
        let width = emitter.get_width(global_time, frame);
        let length = emitter.get_length(global_time, frame);

        for _ in 0..emitted {
            self.spawn(
                model,
                emitter,
                textures,
                texture_set,
                matrix,
                team_id,
                speed,
                variation,
                latitude,
                gravity,
                emitter.lifespan,
                length,
                width,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        model: &mdx::Model,
        emitter: &mdx::ParticleEmitter2,
        textures: &[usize],
        texture_set: &TextureSet,
        matrix: math3::Matrix4,
        team_id: u32,
        speed: f32,
        variation: f32,
        latitude: f32,
        gravity: f32,
        lifespan: f32,
        length: f32,
        width: f32,
    ) {
        let r = rand::random::<f32>();
        let origin = math3::Vector3::from((
            (r - 0.5) * length,
            (rand::random::<f32>() - 0.5) * width,
            0.,
        ));
        let pivot = model
            .pivots
            .get(emitter.node.object_id as usize)
            .copied()
            .unwrap_or_default();

        let mut orientation_matrix = matrix;
        orientation_matrix[3][0] = 0.;
        orientation_matrix[3][1] = 0.;
        orientation_matrix[3][2] = 0.;
        let world_direction = orientation_matrix * generate_random_direction(latitude.to_radians());
        let model_texture_id = emitter.texture_id as usize;
        let texture = model.textures.get(model_texture_id).and_then(|_| {
            let texture_id = textures.get(model_texture_id).copied();
            if emitter.replaceable_id == 0 && texture_id.is_none() {
                return None;
            }
            texture_set
                .get_texture(
                    texture_id.unwrap_or_default(),
                    team_id,
                    emitter.replaceable_id,
                    None,
                    0,
                )
                .cloned()
        });

        let mut particle = Particle {
            texture,
            org: matrix * (origin + pivot),
            vel: world_direction * (speed + (r - 0.5) * variation),
            accel: (0., 0., -gravity).into(),
            lifespan,
            midtime: (emitter.time * 255.) as u8,
            blend_mode: particle_blend_mode(emitter.filter_mode),
            columns: emitter.columns as u8,
            rows: emitter.rows as u8,
            color: [emitter.color(0), emitter.color(1), emitter.color(2)],
            ..Default::default()
        };
        if emitter.node.flags & NODE_XY_QUAD != 0 {
            particle.quad_right = orientation_matrix * math3::Vector3::from((1., 0., 0.));
            particle.quad_up = orientation_matrix * math3::Vector3::from((0., 1., 0.));
        }
        encode_particle_size(&mut particle, &emitter.particle_scaling);

        if emitter.frame_flags == PARTICLE_BOTH {
            self.particles.push(particle.clone());
        }
        if emitter.frame_flags != PARTICLE_HEAD {
            particle.tail = particle.vel * emitter.tail_length;
        }
        self.particles.push(particle);
    }
}

fn emit_count(rate: f32, accumulator: &mut f64, delta_ms: u32) -> u32 {
    if rate <= 0. || delta_ms == 0 {
        return 0;
    }
    if !rate.is_finite() || !accumulator.is_finite() || *accumulator < 0. {
        *accumulator = 0.;
        return 0;
    }

    let mut pending = *accumulator + rate as f64 * delta_ms.min(100) as f64 / 1000.;
    let emitted = pending.floor().min(EMIT_BUDGET as f64) as u32;
    pending -= emitted as f64;
    if pending >= 1. {
        pending %= 1.;
    }
    *accumulator = pending;

    emitted
}

fn particle_blend_mode(filter_mode: u32) -> BlendMode {
    match filter_mode {
        0 => BlendMode::Blend,
        1 => BlendMode::Add,
        2 => BlendMode::Modulate,
        3 => BlendMode::Modulate2X,
        4 => BlendMode::AlphaKey,
        _ => BlendMode::Blend,
    }
}

fn encode_particle_size(particle: &mut Particle, values: &[f32; 3]) {
    let peak = values[0].max(values[1]).max(values[2]);
    particle.size_value_scale = if peak > 0. { peak / 255. } else { 1. };
    for (size, value) in particle.size.iter_mut().zip(values) {
        *size = if peak > 0. {
            (value / peak * 255. + 0.5).clamp(0., 255.) as u8
        } else {
            0
        };
    }
    particle.size_time_scale = 1. / particle.lifespan.max(0.001);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_particle_filter_modes() {
        assert_eq!(particle_blend_mode(0), BlendMode::Blend);
        assert_eq!(particle_blend_mode(1), BlendMode::Add);
        assert_eq!(particle_blend_mode(2), BlendMode::Modulate);
        assert_eq!(particle_blend_mode(3), BlendMode::Modulate2X);
        assert_eq!(particle_blend_mode(4), BlendMode::AlphaKey);
        assert_eq!(particle_blend_mode(u32::MAX), BlendMode::Blend);
    }

    #[test]
    fn encodes_fractional_particle_sizes() {
        let mut particle = Particle {
            lifespan: 2.,
            ..Default::default()
        };

        encode_particle_size(&mut particle, &[0.25, 0.5, 1.]);

        assert_eq!(particle.size, [64, 128, 255]);
        assert_eq!(particle.size_value_scale, 1. / 255.);
        assert_eq!(particle.size_time_scale, 0.5);
    }

    #[test]
    fn advances_emission_accumulator_without_touching_model_data() {
        let mut accumulator = 0.5;

        assert_eq!(emit_count(10., &mut accumulator, 50), 1);
        assert_eq!(accumulator, 0.);
        assert_eq!(emit_count(10., &mut accumulator, 50), 0);
        assert_eq!(accumulator, 0.5);
    }

    #[test]
    fn clamps_particle_emission_stalls_and_budget() {
        let mut accumulator = 0.;

        assert_eq!(emit_count(10_000., &mut accumulator, 1_000), EMIT_BUDGET);
        assert!(accumulator < 1.);
    }
}
