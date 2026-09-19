use crate::shader::ModelGrass;

#[repr(C)]
pub struct ModelState {
    pub bones: [shared::math3::Matrix4; 128],
    pub bone_count: u32,
    pub view_projection: shared::math3::Matrix4,
    pub light_matrix: shared::math3::Matrix4,
    pub texture_matrix: shared::math3::Matrix4,
    pub light_count: i32,
    pub first_bone_lookup_index: f32,
    pub lights: [shared::math3::Matrix4; 8],
    pub grass_params: shared::math3::Matrix4,
    pub model: shared::math3::Matrix4,
    pub normal_matrix: shared::math3::Matrix3,
    pub texture: i32,
    pub shadowmap: i32,
    pub fog_of_war: i32,
    pub layer_alpha: f32,
    pub geoset_color: shared::math3::Vector4,
    pub uv_matrix: shared::math3::Matrix3,
    pub alpha_key: bool,
    pub alpha_cutoff: f32,
    pub unshaded: bool,
    pub fog_enable: bool,
    pub fog_color: shared::math3::Vector3,
    pub fog_params: shared::math3::Vector2,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            bones: [shared::math3::Matrix4::zero(); 128],
            bone_count: 0,
            view_projection: shared::math3::Matrix4::zero(),
            light_matrix: shared::math3::Matrix4::zero(),
            texture_matrix: shared::math3::Matrix4::zero(),
            light_count: 0,
            first_bone_lookup_index: 0.,
            lights: [shared::math3::Matrix4::zero(); 8],
            grass_params: shared::math3::Matrix4::zero(),
            model: shared::math3::Matrix4::zero(),
            normal_matrix: shared::math3::Matrix3::default(),
            texture: 0,
            shadowmap: 1,
            fog_of_war: 2,
            layer_alpha: 0.,
            geoset_color: shared::math3::Vector4::default(),
            uv_matrix: shared::math3::Matrix3::default(),
            alpha_key: false,
            alpha_cutoff: 0.5,
            unshaded: false,
            fog_enable: false,
            fog_color: shared::math3::Vector3::default(),
            fog_params: shared::math3::Vector2::default(),
        }
    }
}

impl ModelState {
    pub fn new_instance() -> Self {
        Self {
            bones: [shared::math3::Matrix4::identity(); 128],
            bone_count: 128,

            ..Default::default()
        }
    }

    pub fn set_grass(&mut self, grass: &ModelGrass) {
        self.grass_params = grass.pack();
    }
}
