use shared::BlendMode;
use web_sys::WebGl2RenderingContext;

pub fn set_blending(gl: &WebGl2RenderingContext, mode: BlendMode) {
    if matches!(mode, BlendMode::Add) {
        gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
    } else {
        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }
}
