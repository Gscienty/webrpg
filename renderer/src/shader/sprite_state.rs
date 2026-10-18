use crate::view::View;

#[repr(C)]
pub struct SpriteState {
    pub view_projection: shared::math3::Matrix4,
    pub model: shared::math3::Matrix4,
    pub texture: i32,
    pub active_glow: f32,
    pub radial_shade: f32,
    pub fog_enable: bool,
    pub fog_color: shared::math3::Vector3,
    pub fog_params: shared::math3::Vector2,
}

impl Default for SpriteState {
    fn default() -> Self {
        Self {
            view_projection: shared::math3::Matrix4::zero(),
            model: shared::math3::Matrix4::zero(),
            texture: 0,
            active_glow: 0.,
            radial_shade: 0.,
            fog_enable: false,
            fog_color: shared::math3::Vector3::default(),
            fog_params: shared::math3::Vector2::default(),
        }
    }
}

impl SpriteState {
    pub fn new(view: &View<'_>) -> Self {
        Self {
            view_projection: view.sprite_state_init_value.view_projection,
            model: view.sprite_state_init_value.model,

            ..Default::default()
        }
    }
}
