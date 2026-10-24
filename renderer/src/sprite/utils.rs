use shared::math3;
use web_sys::WebGl2RenderingContext;

use crate::sprite::render_phase::RenderPhase;

pub(super) fn set_alpha_key_state(
    gl: &WebGl2RenderingContext,
    render_phase: RenderPhase,
    msaa_samples: u32,
    enabled: bool,
) {
    if !enabled {
        gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        return;
    }

    if matches!(render_phase, RenderPhase::Lights) {
        gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        gl.enable(WebGl2RenderingContext::BLEND);
        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        gl.depth_mask(false);
    }

    if msaa_samples != 0 {
        gl.disable(WebGl2RenderingContext::BLEND);
        gl.enable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        gl.depth_mask(true);
        gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
    } else {
        gl.disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        gl.disable(WebGl2RenderingContext::BLEND);
        gl.depth_mask(true);
        gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
    }
}

pub fn evaluate_sprite_transform(angle: f32, origin: math3::Vector3, scale: f32) -> math3::Matrix4 {
    let angle = math3::Orientation::from((angle, 0., 0.));
    let rotation = math3::Quaternion::from(&angle);

    math3::Matrix4::from_rotation_translation_scale_origin(
        rotation,
        origin,
        (scale, scale, scale).into(),
        (0., 0., 0.).into(),
    )
}
