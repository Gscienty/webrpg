use crate::{shader::ModelLighting, view::View};

#[repr(C)]
pub struct DefaultState {
    pub view_projection: shared::math3::Matrix4,
    pub model: shared::math3::Matrix4,
    pub texture_matrix: shared::math3::Matrix4,
    pub light_matrix: shared::math3::Matrix4,
    pub normal_matrix: shared::math3::Matrix3,
    pub light_count: i32,
    pub lights: [shared::math3::Matrix4; 8],
    pub texture: i32,
    pub shadowmap: i32,
    pub fog_of_war: i32,
    pub fog_enable: bool,
    pub fog_color: shared::math3::Vector3,
    pub fog_params: shared::math3::Vector2,
}

impl Default for DefaultState {
    fn default() -> Self {
        Self {
            view_projection: shared::math3::Matrix4::zero(),
            model: shared::math3::Matrix4::zero(),
            texture_matrix: shared::math3::Matrix4::zero(),
            light_matrix: shared::math3::Matrix4::zero(),
            normal_matrix: shared::math3::Matrix3::zero(),
            light_count: 0,
            lights: [shared::math3::Matrix4::zero(); 8],
            texture: 0,
            shadowmap: 1,
            fog_of_war: 2,
            fog_enable: false,
            fog_color: shared::math3::Vector3::default(),
            fog_params: shared::math3::Vector2::default(),
        }
    }
}

impl DefaultState {
    pub fn new(view: &View<'_>) -> Self {
        Self {
            texture_matrix: view.default_state_init_value.texture_matrix,
            model: view.default_state_init_value.model,
            light_matrix: view.default_state_init_value.light_matrix,
            normal_matrix: view.default_state_init_value.normal_matrix,

            ..Default::default()
        }
    }

    pub fn set_lighting(&mut self, lighting: &ModelLighting, is_default: bool) {
        let count = lighting.lights.len();
        if count == 0 {
            if is_default {
                self.light_count = 0;
            }
            return;
        }
        if count > self.lights.len() {
            return;
        }

        self.lights[..count].copy_from_slice(&lighting.pack());
        self.light_count = count as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::{LightType, ModelLight};

    fn light() -> ModelLight {
        ModelLight {
            pos: (1., 2., 3.).into(),
            dir: (4., 5., 6.).into(),
            color: (7., 8., 9.).into(),
            ambient: (10., 11., 12.).into(),
            atten_start: 13.,
            intensity: 14.,
            ambient_intensity: 2.,
            typ: LightType::Direct,
        }
    }

    #[test]
    fn sets_lighting_with_count_and_capacity_checks() {
        let mut state = DefaultState::default();
        state.set_lighting(
            &ModelLighting {
                lights: vec![light()],
                ambient: (1., 2., 3.).into(),
            },
            false,
        );
        assert_eq!(state.light_count, 1);
        assert_eq!(state.lights[0][3], (21., 24., 27., 1.).into());

        state.set_lighting(
            &ModelLighting {
                lights: Vec::new(),
                ambient: (0., 0., 0.).into(),
            },
            false,
        );
        assert_eq!(state.light_count, 1);

        state.set_lighting(
            &ModelLighting {
                lights: Vec::new(),
                ambient: (0., 0., 0.).into(),
            },
            true,
        );
        assert_eq!(state.light_count, 0);

        state.light_count = 7;
        state.set_lighting(
            &ModelLighting {
                lights: (0..9).map(|_| light()).collect(),
                ambient: (0., 0., 0.).into(),
            },
            false,
        );
        assert_eq!(state.light_count, 7);
    }
}
