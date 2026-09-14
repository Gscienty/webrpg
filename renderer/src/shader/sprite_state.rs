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
