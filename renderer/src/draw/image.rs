use shared::BlendMode;
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{RenderBuffer, Vertices},
    draw::draw_image,
    global::Global,
    shader::ShaderType,
    texture::Texture,
};

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
