use web_sys::WebGl2RenderingContext;

use crate::{context::Context, draw};

impl Context {
    pub fn reset_ui_scissor(&self) {
        let gl = &self.gl;
        gl.scissor(
            0,
            0,
            self.drawable_width as i32,
            self.drawable_height as i32,
        );
    }

    pub fn scene_rect(&self) -> shared::math3::Rect {
        draw::scene_rect_for_size(self.drawable_width, self.drawable_height)
    }

    pub fn set_ui_clip_scissor(&self, clip: &shared::math3::Rect) {
        let gl = &self.gl;

        let scissor = draw::ui_clip_scissor(
            gl.drawing_buffer_width().max(0) as u32,
            gl.drawing_buffer_height().max(0) as u32,
            clip,
        );

        gl.enable(WebGl2RenderingContext::SCISSOR_TEST);
        gl.scissor(scissor.0, scissor.1, scissor.2, scissor.3);
    }
}
