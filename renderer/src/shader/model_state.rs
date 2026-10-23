use std::collections::BTreeMap;

use fileformat::mdx;
use shared::math3;

#[derive(Clone)]
#[repr(C)]
pub struct ModelState {
    pub bones: [math3::Matrix4; 128],
    pub bones_count: u32,
    pub view_projection: math3::Matrix4,
    pub light_matrix: math3::Matrix4,
    pub texture_matrix: math3::Matrix4,
    pub first_bone_lookup_index: f32,
    pub lights: [math3::Matrix4; 8],
    pub lights_count: u32,
    pub grass_params: math3::Matrix4,
    pub model: math3::Matrix4,
    pub normal_matrix: math3::Matrix3,
    pub texture: i32,
    pub shadowmap: i32,
    pub fog_of_war: i32,
    pub layer_alpha: f32,
    pub geoset_color: math3::Vector4,
    pub uv_matrix: math3::Matrix3,
    pub alpha_key: bool,
    pub alpha_cutoff: f32,
    pub unshaded: bool,
    pub fog_enable: bool,
    pub fog_color: math3::Vector3,
    pub fog_params: math3::Vector2,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            bones: [math3::Matrix4::zero(); 128],
            bones_count: 0,
            view_projection: math3::Matrix4::zero(),
            light_matrix: math3::Matrix4::zero(),
            texture_matrix: math3::Matrix4::zero(),
            first_bone_lookup_index: 0.,
            lights: [math3::Matrix4::zero(); 8],
            lights_count: 0,
            grass_params: math3::Matrix4::zero(),
            model: math3::Matrix4::zero(),
            normal_matrix: math3::Matrix3::default(),
            texture: 0,
            shadowmap: 1,
            fog_of_war: 2,
            layer_alpha: 0.,
            geoset_color: math3::Vector4::default(),
            uv_matrix: math3::Matrix3::default(),
            alpha_key: false,
            alpha_cutoff: 0.5,
            unshaded: false,
            fog_enable: false,
            fog_color: math3::Vector3::default(),
            fog_params: math3::Vector2::default(),
        }
    }
}

impl ModelState {
    pub fn bind_geoset_matrix_palette(
        &mut self,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        model: &mdx::Model,
        palette: &[u32],
    ) {
        let identity = shared::math3::Matrix4::identity();
        let count = palette.len().min(self.bones.len());

        self.bones.fill(identity);
        for (bone, &id) in self.bones[..count].iter_mut().zip(palette) {
            if model.node(id).is_some() {
                *bone = bone_matrices.get(&id).copied().unwrap_or(identity);
            }
        }
        self.bones_count = count as u32;
    }
}
