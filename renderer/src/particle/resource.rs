use std::collections::BTreeMap;

use fileformat::mdx;
use shared::{BlendMode, math3};

use crate::{
    Error,
    particle::{
        Batch, PARTICLE_TEXTURES, Particle, ParticleRender, utils::generate_random_direction,
    },
    shader,
    texture::Texture,
    texture_set::TextureSet,
};
use web_sys::WebGl2RenderingContext;

const PARTICLE_HEAD: u32 = 0;
const PARTICLE_BOTH: u32 = 2;
const NODE_XY_QUAD: u32 = 0x10_0000;
const EMIT_BUDGET: u32 = 256;

#[derive(Default)]
pub struct Resource {
    particles: Vec<Particle>,

    accumulators: BTreeMap<(usize, u32), f64>,
}

impl Resource {
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn update(&mut self, delta_ms: u32) {
        let frame_time = delta_ms as f32 / 1000.;
        self.particles.retain_mut(|particle| {
            particle.time += frame_time;
            particle.time <= particle.lifespan
        });
    }

    pub fn draw(
        &mut self,
        gl: &WebGl2RenderingContext,
        render: &ParticleRender,
        state: &mut shader::ParticleState,
        delta_ms: u32,
    ) -> Result<(), Error> {
        const VERTICES_PER_PARTICLE: u32 = 6;

        self.update(delta_ms);
        if self.particles.is_empty() {
            return Ok(());
        }

        let mut vertices = shader::ParticleVertices::default();
        let mut batches = Vec::new();
        let mut batch = Batch::default();
        for particle in &self.particles {
            if particle.texture.is_none() {
                continue;
            }

            let mut slot = batch.texture_slot(particle);
            if slot == PARTICLE_TEXTURES {
                batches.push(batch);
                batch = Batch {
                    first: vertices.len() as u32,
                    ..Default::default()
                };
                slot = batch.texture_slot(particle);
            }

            let velocity = particle.vel + particle.accel * (0.5 * particle.time);
            let origin = particle.org + velocity * particle.time;
            vertices.add(
                origin,
                particle.tail,
                particle.quad_right,
                particle.quad_up,
                particle.get_frame(),
                particle.blend_color(),
                particle.blend_size(),
            );
            vertices.set_last_quad_texture_slot(slot);
            batch.count += VERTICES_PER_PARTICLE;
        }
        if batch.count != 0 {
            batches.push(batch);
        }
        if batches.is_empty() {
            return Ok(());
        }

        render.update(gl, &vertices);
        let cull_enabled = gl.is_enabled(WebGl2RenderingContext::CULL_FACE);
        gl.disable(WebGl2RenderingContext::CULL_FACE);
        state.model = math3::Matrix4::identity();
        let mut bound = [None; PARTICLE_TEXTURES as usize];
        let result = batches
            .iter()
            .try_for_each(|batch| self.draw_batch(gl, render, state, batch, &mut bound));
        if cull_enabled {
            gl.enable(WebGl2RenderingContext::CULL_FACE);
        }
        state.alpha_key = false;
        state.modulate = 0;
        gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);

        result
    }

    pub fn draw_batch<'a>(
        &self,
        gl: &WebGl2RenderingContext,
        render: &ParticleRender,
        state: &mut shader::ParticleState,
        batch: &Batch<'a>,
        bound: &mut [Option<&'a Texture>; PARTICLE_TEXTURES as usize],
    ) -> Result<(), Error> {
        if batch.count == 0 || batch.texcount == 0 {
            return Ok(());
        }

        let Some(first_texture) = batch.textures.first().copied().flatten() else {
            return Ok(());
        };
        for (slot, current) in bound.iter_mut().enumerate() {
            let texture = if slot < batch.texcount as usize {
                let Some(texture) = batch.textures.get(slot).copied().flatten() else {
                    return Ok(());
                };
                texture
            } else {
                first_texture
            };
            if current.is_some_and(|current| current.is_same(texture)) {
                continue;
            }
            texture.bind(gl, slot as u32);
            *current = Some(texture);
        }
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        let alpha_key = batch.blend_mode == BlendMode::AlphaKey;
        state.alpha_key = alpha_key;
        state.alpha_cutoff = 0.5;
        state.modulate = particle_modulate_mode(batch.blend_mode);
        gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        match batch.blend_mode {
            BlendMode::None | BlendMode::AlphaKey => {
                gl.disable(WebGl2RenderingContext::BLEND);
                gl.depth_mask(true);
                gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
            }
            blend_mode => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.depth_mask(false);
                let (source, destination) = match blend_mode {
                    BlendMode::Add => (
                        WebGl2RenderingContext::SRC_ALPHA,
                        WebGl2RenderingContext::ONE,
                    ),
                    BlendMode::AddAlpha => {
                        (WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE)
                    }
                    BlendMode::Modulate => (
                        WebGl2RenderingContext::ZERO,
                        WebGl2RenderingContext::SRC_COLOR,
                    ),
                    BlendMode::Modulate2X => (
                        WebGl2RenderingContext::DST_COLOR,
                        WebGl2RenderingContext::SRC_COLOR,
                    ),
                    _ => (
                        WebGl2RenderingContext::SRC_ALPHA,
                        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                    ),
                };
                gl.blend_func(source, destination);
            }
        }

        render.bind(gl);
        render.apply(gl, state)?;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
            batch.first as i32,
            batch.count as i32,
        );
        Ok(())
    }

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
        set_uv_curve(
            &mut particle,
            emitter.lifespan_uv_anim,
            emitter.decay_uv_anim,
        );
        if emitter.node.flags & NODE_XY_QUAD != 0 {
            particle.quad_right = orientation_matrix * math3::Vector3::from((1., 0., 0.));
            particle.quad_up = orientation_matrix * math3::Vector3::from((0., 1., 0.));
        }
        encode_particle_size(&mut particle, &emitter.particle_scaling);

        if emitter.frame_flags == PARTICLE_BOTH {
            self.particles.push(particle.clone());
        }
        if emitter.frame_flags != PARTICLE_HEAD {
            set_uv_curve(
                &mut particle,
                emitter.tail_uv_anim,
                emitter.tail_decay_uv_anim,
            );
            particle.tail = particle.vel * emitter.tail_length;
        }
        self.particles.push(particle);
    }
}

fn set_uv_curve(
    particle: &mut Particle,
    lifespan: mdx::ParticleAnimation,
    decay: mdx::ParticleAnimation,
) {
    particle.uv_start = lifespan.start.min(u16::MAX as u32) as u16;
    particle.uv_mid = lifespan.end.min(u16::MAX as u32) as u16;
    particle.uv_end = decay.end.min(u16::MAX as u32) as u16;
    particle.use_uv_curve = true;
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
        2 => BlendMode::Blend,
        3 => BlendMode::Modulate2X,
        4 => BlendMode::AlphaKey,
        _ => BlendMode::Blend,
    }
}

fn particle_modulate_mode(blend_mode: BlendMode) -> i32 {
    match blend_mode {
        BlendMode::Modulate => 1,
        BlendMode::Modulate2X => 2,
        _ => 0,
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
        assert_eq!(particle_blend_mode(2), BlendMode::Blend);
        assert_eq!(particle_blend_mode(3), BlendMode::Modulate2X);
        assert_eq!(particle_blend_mode(4), BlendMode::AlphaKey);
        assert_eq!(particle_blend_mode(u32::MAX), BlendMode::Blend);
    }

    #[test]
    fn selects_modulate_shader_neutral() {
        assert_eq!(particle_modulate_mode(BlendMode::Modulate), 1);
        assert_eq!(particle_modulate_mode(BlendMode::Modulate2X), 2);
        assert_eq!(particle_modulate_mode(BlendMode::Add), 0);
    }

    #[test]
    fn applies_authored_particle_uv_curve() {
        let mut particle = Particle::default();
        set_uv_curve(
            &mut particle,
            mdx::ParticleAnimation {
                start: 4,
                end: 9,
                repeat: 1,
            },
            mdx::ParticleAnimation {
                start: 10,
                end: 15,
                repeat: 1,
            },
        );

        assert!(particle.use_uv_curve);
        assert_eq!(
            (particle.uv_start, particle.uv_mid, particle.uv_end),
            (4, 9, 15)
        );
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

    #[test]
    fn update_advances_and_removes_expired_particles() {
        let mut resource = Resource {
            particles: vec![
                Particle {
                    time: 0.5,
                    lifespan: 1.,
                    ..Default::default()
                },
                Particle {
                    time: 0.9,
                    lifespan: 1.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        resource.update(200);

        assert_eq!(resource.particles.len(), 1);
        assert_eq!(resource.particles[0].time, 0.7);
    }
}
