use std::collections::BTreeMap;

use fileformat::mdx;
use shared::math3;

use crate::{shader, sprite::Sprite};

impl Sprite<'_> {
    pub(super) fn pack_light(
        &self,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        light_matrix: math3::Matrix4,
        transform: math3::Matrix4,
        global_time: u32,
        frame: u32,
    ) -> ([math3::Matrix4; 8], u32) {
        let mut lighting = shader::ModelLighting::default();

        lighting.lights.extend(self.collect_model_lights(
            bone_matrices,
            transform,
            global_time,
            frame,
        ));

        let ambient = if lighting.lights.is_empty() { 0.35 } else { 0. };
        lighting.ambient = (ambient, ambient, ambient).into();

        if lighting.lights.is_empty() {
            let directional = 0.75;
            lighting.lights.push(shader::ModelLight {
                dir: (
                    -light_matrix[0][2],
                    -light_matrix[1][2],
                    -light_matrix[2][2],
                )
                    .into(),
                color: (directional, directional, directional).into(),
                intensity: 1.,
                typ: shader::LightType::Direct,

                ..Default::default()
            });
        }

        let packed_lights = lighting.pack();
        let lights_count = packed_lights.len().min(8);
        let mut lights = [math3::Matrix4::zero(); 8];
        lights[..lights_count].copy_from_slice(&packed_lights[..lights_count]);

        (lights, lights_count as u32)
    }

    fn collect_model_lights(
        &self,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        transform: math3::Matrix4,
        global_time: u32,
        frame: u32,
    ) -> Vec<shader::ModelLight> {
        self.model
            .lights
            .iter()
            .filter_map(|light| {
                self.evaluate_light(light, bone_matrices, transform, global_time, frame, true)
            })
            .collect()
    }

    fn evaluate_light(
        &self,
        light: &mdx::Light,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        transform: math3::Matrix4,
        global_time: u32,
        frame: u32,
        visibility: bool,
    ) -> Option<shader::ModelLight> {
        let visibility = if visibility {
            light
                .visibility_tracks
                .as_ref()
                .and_then(|track| track.get_value(global_time, frame))
                .unwrap_or(1.)
        } else {
            1.
        };
        if visibility < f32::EPSILON {
            return None;
        }

        let color = light
            .color_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .unwrap_or(light.color);
        let intensity = light
            .intensity_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .unwrap_or(light.intensity);
        let ambient = light
            .ambient_color_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .unwrap_or(light.ambient_color);
        let ambient_intensity = light
            .ambient_intensity_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .unwrap_or(light.ambient_intensity);
        let atten_start = light
            .attenuation_start_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .unwrap_or(light.attenuation_start);

        let pivot = self
            .model
            .pivots
            .get(light.node.object_id as usize)
            .copied()
            .unwrap_or_default();
        let mut local_pos = pivot;
        let mut local_dir_target = pivot + math3::Vector3::from((0., 0., -1.));
        if let Some(&node_matrix) = bone_matrices.get(&light.node.object_id) {
            local_pos = node_matrix * local_pos;
            local_dir_target = node_matrix * local_dir_target;
        }

        let world_pos = transform * local_pos;
        let world_dir_target = transform * local_dir_target;
        let world_dir = world_dir_target - world_pos;
        let world_dir = if world_dir.length_sq() < f32::EPSILON {
            math3::Vector3::from((0., 0., -1.))
        } else {
            world_dir.normalize()
        };

        Some(shader::ModelLight {
            pos: world_pos,
            dir: -world_dir,
            color,
            ambient,
            atten_start,
            intensity: intensity * visibility,
            ambient_intensity: ambient_intensity * visibility,
            typ: match light.type_ {
                mdx::LightType::Omni => shader::LightType::Omni,
                mdx::LightType::Direct => shader::LightType::Direct,
                mdx::LightType::Ambient => shader::LightType::Ambient,
            },
        })
    }
}
