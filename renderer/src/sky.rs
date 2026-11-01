use std::pin::Pin;

use fileformat::mdx;
use shared::{color::Color32, math3};
use web_sys::WebGl2RenderingContext;

use crate::{
    Error, shader,
    sprite::{self, RenderParams, render_phase::RenderPhase},
    texture_set::TextureSet,
};

pub struct Sky<'s> {
    sky: sprite::Sprite<'s>,
    animation_interval: Option<[u32; 2]>,
}

impl<'s> Sky<'s> {
    pub fn new(
        gl: &WebGl2RenderingContext,
        model: Pin<Box<mdx::Model<'s>>>,

        mpq: Option<&mut dyn webrpg_mpq::Archive>,
        texture_set: &mut TextureSet,
    ) -> Result<Self, Error> {
        let animation_interval = model.sequences.first().map(|sequence| sequence.interval);
        let sky = sprite::Sprite::new(gl, model, mpq, texture_set)?;

        Ok(Self {
            sky,
            animation_interval,
        })
    }

    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        eye: math3::Vector3,
        global_time: u32,
        view_projection: math3::Matrix4,
        texture_set: &TextureSet,
        model_program: &shader::Program,
    ) -> Result<(), Error> {
        let frame = self
            .animation_interval
            .filter(|interval| interval[1] > interval[0])
            .map(|interval| interval[0] + global_time % (interval[1] - interval[0]))
            .unwrap_or_default();
        let render_param = RenderParams {
            origin: eye,
            angle: 0.,
            scale: 1.,
            no_frustum_cull: true,
            no_face_cull: false,
            no_lighting: true,
            no_depth_test: true,
            no_depth_write: true,
            not_selectable: false,
            emission_paused: false,
            lerpfrac: 0.,
            tint: Color32::white(),
            team: 0,
            global_time,
            frame,
            previous_frame: frame,
            delta: 0,
            view_projection,
            texture: math3::Matrix4::identity(),
            light: math3::Matrix4::identity(),
            fog: None,
            override_texture: None,
            override_slot: 0,
            frustum: math3::Frustum3::calculate(view_projection),
            texture_set,
            model_program,
            render_phase: RenderPhase::Solid,
            msaa_samples: 0,
        };

        let depth_test_enabled = gl.is_enabled(WebGl2RenderingContext::DEPTH_TEST);
        let depth_write_enabled = gl
            .get_parameter(WebGl2RenderingContext::DEPTH_WRITEMASK)
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        gl.depth_mask(false);
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        render_param.texture_set.white.bind(gl, 2);

        let result = self.sky.render(gl, render_param, None);

        if depth_test_enabled {
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        } else {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
        gl.depth_mask(depth_write_enabled);

        result
    }
}
