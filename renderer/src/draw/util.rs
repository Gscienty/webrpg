use shared::BlendMode;
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{RenderBuffer, Vertices},
    draw::set_blending,
    global::Global,
    shader::{ShaderType, SpriteState},
    texture::Texture,
};

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

    global.builtin_shader.apply(gl, ShaderType::UI, &state)?;
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);

    Ok(())
}

pub(super) fn ui_clip_scissor(
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
    global.builtin_shader.apply(gl, shader_type, &state)?;
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
