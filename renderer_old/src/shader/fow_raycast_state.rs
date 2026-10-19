#[repr(C)]
pub struct FowRaycastState {
    pub view_projection: shared::math3::Matrix4,
    pub model: shared::math3::Matrix4,
    pub eye_position: shared::math3::Vector2,
}
