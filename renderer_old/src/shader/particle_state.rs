#[repr(C)]
pub struct ParticleState {
    pub view_projection: shared::math3::Matrix4,
    pub texture_matrix: shared::math3::Matrix4,
    pub model: shared::math3::Matrix4,
    pub eye: shared::math3::Vector3,
    pub texture: i32,
    pub fog_of_war: i32,
    pub alpha_key: bool,
    pub alpha_cutoff: f32,
}

impl Default for ParticleState {
    fn default() -> Self {
        Self {
            view_projection: shared::math3::Matrix4::zero(),
            texture_matrix: shared::math3::Matrix4::zero(),
            model: shared::math3::Matrix4::zero(),
            eye: shared::math3::Vector3::default(),
            texture: 0,
            fog_of_war: 1,
            alpha_key: false,
            alpha_cutoff: 0.5,
        }
    }
}
