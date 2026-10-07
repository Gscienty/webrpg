mod buffers;
mod draw;
mod particles;
mod render_phase;
mod shaders;
mod textures;
mod utils;

pub use particles::Particles;
pub use render_phase::RenderPhase;
pub use shaders::Shaders;
pub use textures::Textures;

use std::cell::RefCell;

use web_sys::WebGl2RenderingContext;

use crate::{Error, context::buffers::Buffers, render_stats::RenderStats, texture::RenderTarget};

pub struct Context {
    pub gl: WebGl2RenderingContext,

    pub render_phase: RenderPhase,

    pub shaders: Shaders,
    pub textures: Textures,
    pub buffers: Buffers,
    pub particles: Particles,

    pub msaa_samples: i32,

    pub depth_map: RenderTarget,

    pub window_width: u32,
    pub window_height: u32,

    pub drawable_width: u32,
    pub drawable_height: u32,

    pub render_stats: RefCell<RenderStats>,
}

impl Context {
    pub fn init(
        gl: WebGl2RenderingContext,
        window_width: u32,
        window_height: u32,
        drawable_width: u32,
        drawable_height: u32,
        war3_mpq: Option<&mut webrpg_mpq::Archive>,
    ) -> Result<Self, Error> {
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.clear_color(0., 0., 0., 1.);
        gl.viewport(0, 0, drawable_width as i32, drawable_height as i32);

        let shaders = Shaders::init(&gl)?;
        let textures = Textures::init(&gl, war3_mpq)?;
        let buffers = Buffers::init(&gl)?;
        let particles = Particles::init();
        let depth_map = RenderTarget::new(
            &gl,
            1024,
            1024,
            WebGl2RenderingContext::DEPTH_COMPONENT,
            WebGl2RenderingContext::FLOAT,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
        )?;

        Ok(Self {
            gl,
            render_phase: RenderPhase::Solid,
            shaders,
            textures,
            buffers,
            particles,
            msaa_samples: 0,
            depth_map,
            window_width,
            window_height,
            drawable_width,
            drawable_height,
            render_stats: RefCell::new(RenderStats::default()),
        })
    }

    pub fn set_alpha_key_state(&self, enabled: bool) {
        let gl = &self.gl;
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
