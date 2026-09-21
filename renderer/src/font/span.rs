use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use shared::{color::Color32, math3::Rect};
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::Vertices,
    global::Global,
    shader::{ShaderType, SpriteState},
    texture::Texture,
};

pub struct Span<'s> {
    font: &'s fontdue::Font,
    layout: Layout<Color32>,
    color: Color32,
    size: f32,
}

impl<'s> Span<'s> {
    pub fn new(font: &'s fontdue::Font) -> Self {
        Self {
            font,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            color: Color32::white(),
            size: 16.,
        }
    }

    pub fn set_color(&mut self, color: Color32) {
        self.color = color;
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn write(&mut self, text: &str) {
        self.layout.append(
            std::slice::from_ref(&self.font),
            &TextStyle::with_user_data(text, self.size, 0, self.color),
        );
    }

    pub fn draw(
        &self,
        global: &Global,
        gl: &WebGl2RenderingContext,
        x: f32,
        y: f32,
    ) -> Result<(), Error> {
        let glyphs = self.layout.glyphs();
        let glyph_width: usize = glyphs.iter().map(|glyph| glyph.width).sum();
        let glyph_height = glyphs.iter().map(|glyph| glyph.height).max().unwrap_or(0);
        if glyph_width == 0 || glyph_height == 0 {
            return Ok(());
        }

        let atlas_width = glyph_width + glyphs.len() + 1;
        let atlas_height = glyph_height + 2;
        let mut pixels = vec![Color32::default(); atlas_width * atlas_height];
        let mut vertices = Vertices::default();
        let mut atlas_x = 1;

        for glyph in glyphs {
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let (_, bitmap) = self.font.rasterize_config(glyph.key);
            for (row, source) in bitmap.chunks_exact(glyph.width).enumerate() {
                let destination = (row + 1) * atlas_width + atlas_x;
                for (pixel, alpha) in pixels[destination..destination + glyph.width]
                    .iter_mut()
                    .zip(source)
                {
                    *pixel = Color32 {
                        red: 255,
                        green: 255,
                        blue: 255,
                        alpha: *alpha,
                    };
                }
            }

            vertices.add_quad(
                &Rect::new(
                    x + glyph.x,
                    y + glyph.y,
                    glyph.width as f32,
                    glyph.height as f32,
                ),
                &Rect::new(
                    atlas_x as f32 / atlas_width as f32,
                    1. / atlas_height as f32,
                    glyph.width as f32 / atlas_width as f32,
                    glyph.height as f32 / atlas_height as f32,
                ),
                glyph.user_data,
                0.,
            );
            atlas_x += glyph.width + 1;
        }

        let texture = Texture::from_pixels(gl, &pixels, atlas_width as u32, atlas_height as u32)?;

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
        texture.bind(gl, 0);

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

        global
            .render_stats
            .borrow_mut()
            .draw(WebGl2RenderingContext::TRIANGLES, vertices.len(), 1);
        let result = global.shader.apply(gl, ShaderType::UI, &state);
        if result.is_ok() {
            gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
        }
        texture.drop(gl);

        result.map_err(Into::into)
    }
}
