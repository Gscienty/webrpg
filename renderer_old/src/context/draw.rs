use shared::BlendMode;
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::Vertices,
    context::Context,
    draw,
    shader::{ShaderType, SpriteState},
    texture::Texture,
};

impl Context {
    pub fn draw_fill(
        &self,
        rect: &shared::math3::Rect,
        color: shared::color::Color32,
    ) -> Result<(), Error> {
        let gl = &self.gl;
        if rect.w <= 0. || rect.h <= 0. || color.alpha == 0 {
            return Ok(());
        }

        let mut vertices = Vertices::default();
        vertices.add_quad(rect, &shared::math3::Rect::new(0., 0., 1., 1.), color, 0.);

        gl.bind_vertex_array(Some(&self.buffers.ui.vao));
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.buffers.ui.vbo),
        );
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            vertices.as_bytes(),
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        self.textures.white_texture.bind(gl, 0);

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
                self.drawable_width as f32,
                self.drawable_height as f32,
                0.,
                0.,
                100.,
            ),
            model: shared::math3::Matrix4::identity(),
            ..Default::default()
        };

        self.shaders.apply(gl, ShaderType::UI, &state)?;
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_image(
        &self,

        texture: &Texture,
        shader_type: ShaderType,
        alpha_mode: BlendMode,
        active_glow: f32,
        radial_shade: f32,
        clip: Option<shared::math3::Rect>,
        vertices: &Vertices,
        repeat: bool,
    ) -> Result<(), Error> {
        let gl = &self.gl;
        if vertices.is_empty() {
            return Ok(());
        }

        let scene = self.scene_rect();
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

        gl.bind_vertex_array(Some(&self.buffers.ui.vao));
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.buffers.ui.vbo),
        );
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            vertices.as_bytes(),
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        gl.depth_mask(false);
        gl.enable(WebGl2RenderingContext::BLEND);

        draw::set_blending(gl, alpha_mode);
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
            self.set_ui_clip_scissor(clip);
        }

        self.render_stats
            .borrow_mut()
            .draw(WebGl2RenderingContext::TRIANGLES, vertices.len(), 1);
        self.shaders.apply(gl, shader_type, &state)?;
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
        if clip.is_some() {
            self.reset_ui_scissor();
        }

        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        Ok(())
    }
}
