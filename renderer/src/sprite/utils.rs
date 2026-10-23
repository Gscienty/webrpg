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
