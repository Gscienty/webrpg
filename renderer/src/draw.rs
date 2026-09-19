use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{RenderBuffer, Vertices},
    global::Global,
    shader::{ShaderType, SpriteState},
    texture::Texture,
};

#[derive(Clone, Copy)]
pub enum BlendMode {
    None,
    AlphaKey,
    Blend,
    Add,
    AddAlpha,
    Modulate,
    Modulate2X,
}

pub struct DrawImage<'s> {
    pub texture: &'s Texture,
    pub shader_type: ShaderType,
    pub alpha_mode: BlendMode,
    pub screen: shared::math3::Rect,
    pub uv: shared::math3::Rect,
    pub color: shared::color::Color32,
    pub angle: f32,
    pub active_glow: f32,
    pub radial_shade: f32,
    pub clip: Option<shared::math3::Rect>,
}

pub const DRAW_CLIP: u8 = 1 << 0;

fn scene_rect_for_size(drawable_width: u32, drawable_height: u32) -> shared::math3::Rect {
    if drawable_height > 0 {
        let aspect = drawable_width as f32 / drawable_height as f32;

        if aspect > 4. / 3. {
            return shared::math3::Rect::new(0., 0., 0.8 * aspect, 0.6 * aspect);
        }
    }

    shared::math3::Rect::new(0., 0., 0.8, 0.6)
}

pub fn scene_rect(global: &Global) -> shared::math3::Rect {
    scene_rect_for_size(global.drawable_width, global.drawable_height)
}

pub fn draw_fill(
    global: &Global,
    gl: &WebGl2RenderingContext,
    rect: &shared::math3::Rect,
    color: shared::color::Color32,
) -> Result<(), Error> {
    if rect.w <= 0. || rect.h <= 0. || color.alpha == 0 {
        return Ok(());
    }

    let mut vertices = Vertices::default();
    vertices.add_quad(rect, &shared::math3::Rect::new(0., 0., 1., 1.), color, 0.);

    gl.bind_vertex_array(Some(&global.ui_buffer.vao));
    gl.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&global.ui_buffer.vbo),
    );
    gl.buffer_data_with_u8_array(
        WebGl2RenderingContext::ARRAY_BUFFER,
        vertices.as_bytes(),
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );

    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    global.white_texture.bind(gl, 0);

    gl.disable(WebGl2RenderingContext::CULL_FACE);
    gl.disable(WebGl2RenderingContext::DEPTH_TEST);
    gl.depth_mask(false);
    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    let state = SpriteState {
        view_projection: shared::math3::Matrix4::ortho(
            0.,
            global.drawable_width as f32,
            global.drawable_height as f32,
            0.,
            0.,
            100.,
        ),
        model: shared::math3::Matrix4::identity(),
        ..Default::default()
    };

    global.shader.apply(gl, ShaderType::UI, &state)?;
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);

    Ok(())
}

pub fn set_blending(gl: &WebGl2RenderingContext, mode: BlendMode) {
    if matches!(mode, BlendMode::Add) {
        gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
    } else {
        gl.blend_func(
            WebGl2RenderingContext::ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
    }
}

fn ui_clip_scissor(
    drawable_width: u32,
    drawable_height: u32,
    clip: &shared::math3::Rect,
) -> (i32, i32, i32, i32) {
    let scene = scene_rect_for_size(drawable_width, drawable_height);
    let x = ((clip.x - scene.x) / scene.w).clamp(0., 1.);
    let y = (1. - (clip.y + clip.h - scene.y) / scene.h).clamp(0., 1.);
    let w = (clip.w / scene.w).clamp(0., 1. - x);
    let h = (clip.h / scene.h).clamp(0., 1. - y);

    (
        (x * drawable_width as f32) as i32,
        (y * drawable_height as f32) as i32,
        (w * drawable_width as f32) as i32,
        (h * drawable_height as f32) as i32,
    )
}

pub fn set_ui_clip_scissor(gl: &WebGl2RenderingContext, clip: &shared::math3::Rect) {
    let scissor = ui_clip_scissor(
        gl.drawing_buffer_width().max(0) as u32,
        gl.drawing_buffer_height().max(0) as u32,
        clip,
    );

    gl.enable(WebGl2RenderingContext::SCISSOR_TEST);
    gl.scissor(scissor.0, scissor.1, scissor.2, scissor.3);
}

pub fn reset_ui_scissor(global: &Global, gl: &WebGl2RenderingContext) {
    gl.scissor(
        0,
        0,
        global.drawable_width as i32,
        global.drawable_height as i32,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn draw_image(
    global: &Global,
    gl: &WebGl2RenderingContext,
    buffer: &RenderBuffer,

    texture: &Texture,
    shader_type: ShaderType,
    alpha_mode: BlendMode,
    active_glow: f32,
    radial_shade: f32,
    clip: Option<shared::math3::Rect>,
    vertices: &Vertices,
    repeat: bool,
) -> Result<(), Error> {
    if vertices.is_empty() {
        return Ok(());
    }

    let scene = scene_rect(global);
    let view_projection = shared::math3::Matrix4::ortho(
        scene.x,
        scene.x + scene.w,
        scene.y + scene.h,
        scene.y,
        0.,
        100.,
    );
    let model = shared::math3::Matrix4::identity();

    gl.disable(WebGl2RenderingContext::CULL_FACE);

    let state = SpriteState {
        view_projection,
        model,

        active_glow,
        radial_shade,

        ..Default::default()
    };

    gl.bind_vertex_array(Some(&buffer.vao));
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer.vbo));
    gl.buffer_data_with_u8_array(
        WebGl2RenderingContext::ARRAY_BUFFER,
        vertices.as_bytes(),
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );
    gl.disable(WebGl2RenderingContext::DEPTH_TEST);
    gl.depth_mask(false);
    gl.enable(WebGl2RenderingContext::BLEND);

    set_blending(gl, alpha_mode);
    texture.bind(gl, 0);

    if repeat {
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::REPEAT as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::REPEAT as i32,
        );
    } else {
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
    }
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.disable(WebGl2RenderingContext::CULL_FACE);
    gl.enable(WebGl2RenderingContext::BLEND);
    if let Some(ref clip) = clip {
        set_ui_clip_scissor(gl, clip);
    }

    global
        .render_stats
        .borrow_mut()
        .draw(WebGl2RenderingContext::TRIANGLES, vertices.len(), 1);
    global.shader.apply(gl, shader_type, &state)?;
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
    if clip.is_some() {
        reset_ui_scissor(global, gl);
    }

    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    Ok(())
}

impl<'s> DrawImage<'s> {
    pub fn new(
        texture: &'s Texture,
        screen: shared::math3::Rect,
        uv: Option<shared::math3::Rect>,
        color: shared::color::Color32,
    ) -> Self {
        Self {
            texture,
            screen,
            uv: uv.unwrap_or(shared::math3::Rect::new(0., 0., 1., 1.)),
            color,
            shader_type: ShaderType::UI,
            alpha_mode: BlendMode::None,
            active_glow: 0.,
            angle: 0.,
            clip: None,
            radial_shade: 0.,
        }
    }

    pub fn draw(
        &self,
        global: &Global,
        gl: &WebGl2RenderingContext,
        buffer: &RenderBuffer,
    ) -> Result<(), Error> {
        let mut vertices = Vertices::default();
        vertices.add_quad(&self.screen, &self.uv, self.color, 0.);

        if self.angle != 0. {
            vertices.rotate_2d(
                self.screen.x + self.screen.w * 0.5,
                self.screen.y + self.screen.h * 0.5,
                self.angle.to_radians(),
            );
        }

        draw_image(
            global,
            gl,
            buffer,
            self.texture,
            self.shader_type,
            self.alpha_mode,
            self.active_glow,
            self.radial_shade,
            self.clip,
            &vertices,
            self.uv.w > 1. || self.uv.h > 1.,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_and_clamps_ui_clip_to_scissor_pixels() {
        assert_eq!(
            ui_clip_scissor(800, 600, &shared::math3::Rect::new(0.2, 0.15, 0.4, 0.3)),
            (200, 150, 400, 300)
        );
        assert_eq!(
            ui_clip_scissor(800, 600, &shared::math3::Rect::new(-1., -1., 3., 3.)),
            (0, 0, 800, 600)
        );
    }
}
