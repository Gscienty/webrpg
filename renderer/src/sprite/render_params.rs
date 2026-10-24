use shared::{color, math3};

use crate::{shader, sprite::render_phase::RenderPhase, texture, texture_set::TextureSet};

pub struct RenderParams<'s> {
    pub origin: math3::Vector3,
    pub angle: f32,
    pub scale: f32,

    pub no_frustum_cull: bool,
    pub no_face_cull: bool,
    pub no_lighting: bool,

    pub render_phase: RenderPhase,
    pub msaa_samples: u32,
    pub lerpfrac: f32,
    pub tint: color::Color32,
    pub team: u32,

    pub global_time: u32,
    pub frame: u32,
    pub previous_frame: u32,

    pub view_projection: math3::Matrix4,
    pub texture: math3::Matrix4,
    pub light: math3::Matrix4,

    pub fog: Option<(
        math3::Vector3, /* color */
        math3::Vector2, /* range */
    )>,

    pub override_texture: Option<&'s texture::Texture>,
    pub override_slot: u32,

    pub frustum: math3::Frustum3,

    pub texture_set: &'s TextureSet,

    pub model_program: &'s shader::Program,
}

impl<'s> RenderParams<'s> {
    pub fn evaluate_sprite_transform(&self) -> math3::Matrix4 {
        let angle = math3::Orientation::from((self.angle, 0., 0.));
        let rotation = math3::Quaternion::from(&angle);

        math3::Matrix4::from_rotation_translation_scale_origin(
            rotation,
            self.origin,
            (self.scale, self.scale, self.scale).into(),
            (0., 0., 0.).into(),
        )
    }
}
