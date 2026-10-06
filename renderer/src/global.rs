use std::cell::RefCell;

use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{ParticleVertices, RenderBuffer, Vertices},
    render_stats::RenderStats,
    shader,
    texture::{RenderTarget, Texture},
};

pub enum RenderPhase {
    Solid,
    Lights,
    Alpha,
}

pub struct Global {
    pub render_phase: RenderPhase,
    pub msaa_samples: i32,

    pub builtin_shader: shader::BuiltinShader,

    pub ui_buffer: RenderBuffer,
    pub particle_buffer: RenderBuffer,

    pub white_texture: Texture,
    pub black_texture: Texture,
    pub placeholder_texture: Texture,
    pub blob_shadow_texture: Texture,
    pub loading_indicator_texture: Texture,

    pub depth_map: RenderTarget,

    pub window_width: u32,
    pub window_height: u32,

    pub drawable_width: u32,
    pub drawable_height: u32,

    pub render_stats: RefCell<RenderStats>,
}

impl Global {
    pub fn init(
        gl: &WebGl2RenderingContext,
        window_width: u32,
        window_height: u32,
        drawable_width: u32,
        drawable_height: u32,
    ) -> Result<Self, Error> {
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.clear_color(0., 0., 0., 1.);
        gl.viewport(0, 0, drawable_width as i32, drawable_height as i32);

        // TODO init

        Ok(Self {
            render_phase: RenderPhase::Solid,
            msaa_samples: 0,

            builtin_shader: shader::BuiltinShader::init(gl)?,

            ui_buffer: Vertices::create_array_object(gl)?,
            particle_buffer: ParticleVertices::create_array_object(gl)?,

            white_texture: Texture::white(gl)?,
            black_texture: Texture::black(gl)?,
            placeholder_texture: Texture::placeholder(gl)?,
            blob_shadow_texture: Texture::blob_shadow(gl)?,
            loading_indicator_texture: Texture::loading_indicator(gl)?,

            depth_map: RenderTarget::new(
                gl,
                1024,
                1024,
                WebGl2RenderingContext::DEPTH_COMPONENT,
                WebGl2RenderingContext::FLOAT,
                WebGl2RenderingContext::DEPTH_ATTACHMENT,
            )?,

            window_width,
            window_height,

            drawable_width,
            drawable_height,

            render_stats: RefCell::new(RenderStats::default()),
        })
    }

    pub fn set_alpha_key_state(&self, gl: &WebGl2RenderingContext, enabled: bool) {
        if !enabled {
            gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
            return;
        }

        if matches!(self.render_phase, RenderPhase::Lights) {
            gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
        }

        if self.msaa_samples != 0 {
            gl.disable(WebGl2RenderingContext::BLEND);
            gl.enable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
            gl.depth_mask(true);
            gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
        } else {
            gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
        }
    }
}
