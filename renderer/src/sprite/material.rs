use fileformat::mdx;
use web_sys::WebGl2RenderingContext;

use crate::{
    shader::ModelState,
    sprite::{render_phase::RenderPhase, utils},
};

pub(super) struct Material<'s> {
    pub material: &'s mdx::Material<'s>,
}

impl<'s> Material<'s> {
    pub fn new(material: &'s mdx::Material<'s>) -> Self {
        Self { material }
    }

    pub fn has_pass(&self, blended_pass: bool) -> bool {
        self.material
            .layers
            .iter()
            .any(|layer| layer.is_blended() == blended_pass)
    }

    pub fn layers(&self) -> impl Iterator<Item = &mdx::Layer<'s>> {
        self.material.layers.iter()
    }

    pub fn layer(&self, index: usize) -> Option<&mdx::Layer<'s>> {
        self.material.layers.get(index)
    }

    pub fn layers_count(&self) -> usize {
        self.material.layers.len()
    }

    pub fn priority(&self) -> u32 {
        self.material.priority
    }
}

pub(super) fn layer_set_blended(
    gl: &WebGl2RenderingContext,
    layer_id: usize,
    layer: &mdx::Layer,
    render_phase: RenderPhase,
    msaa_samples: u32,
    state: &mut ModelState,
) -> bool {
    utils::set_alpha_key_state(gl, render_phase, msaa_samples, false);

    if render_phase == RenderPhase::Lights && layer.is_blended() {
        return false;
    }

    match layer.blend_mode {
        shared::BlendMode::None => {
            gl.disable(WebGl2RenderingContext::BLEND);
            if layer_id == 0 {
                gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
            } else {
                gl.blend_func(
                    WebGl2RenderingContext::SRC_ALPHA,
                    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                );
            }
            gl.depth_mask(true);
        }
        shared::BlendMode::AlphaKey => {
            state.alpha_key = true;
            utils::set_alpha_key_state(gl, render_phase, msaa_samples, true);
        }
        shared::BlendMode::Blend => {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
            gl.depth_mask(false);
        }
        shared::BlendMode::Add => {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
            gl.depth_mask(false);
        }
        shared::BlendMode::AddAlpha => {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE,
            );
            gl.depth_mask(false);
        }
        shared::BlendMode::Modulate => {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::DST_COLOR,
                WebGl2RenderingContext::ZERO,
            );
            gl.depth_mask(false);
        }
        shared::BlendMode::Modulate2X => {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::DST_COLOR,
                WebGl2RenderingContext::SRC_COLOR,
            );
            gl.depth_mask(false);
        }
    }

    true
}

pub(super) fn layer_apply_flags(gl: &WebGl2RenderingContext, layer: &mdx::Layer) {
    if layer.flags & mdx::GeoFlag::TwoSided as u32 != 0 {
        gl.disable(WebGl2RenderingContext::CULL_FACE);
    }
    if layer.flags & mdx::GeoFlag::NoDepthTest as u32 != 0 {
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
    }
    if layer.flags & mdx::GeoFlag::NoDepthSet as u32 != 0 {
        gl.depth_mask(false);
    }
}
