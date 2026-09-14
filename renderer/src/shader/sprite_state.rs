pub struct SpriteState {
    pub(super) view_projection: shared::math3::Matrix4,
    pub(super) model: shared::math3::Matrix4,
    pub(super) texture: i32,
    pub(super) active_glow: f32,
    pub(super) radial_shade: f32,
    pub(super) fog_enable: bool,
    pub(super) fog_color: shared::math3::Vector3,
    pub(super) fog_params: shared::math3::Vector2,
}
