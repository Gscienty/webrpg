use std::collections::BTreeMap;

use fileformat::mdx;
use shared::math3;

pub(super) fn get_node_global_matrix(
    model: &mdx::Model,
    node: &mdx::Node,
    transform: math3::Matrix4,
    local_matrices: &BTreeMap<u32, math3::Matrix4>,
    global_matrices: &mut BTreeMap<u32, math3::Matrix4>,
) -> math3::Matrix4 {
    if let Some(&matrix) = global_matrices.get(&node.object_id) {
        return matrix;
    }

    let local_matrix = local_matrices
        .get(&node.object_id)
        .copied()
        .unwrap_or_default();
    let parent = model.node(node.parent_id);
    let mut global_matrix = if let Some(parent) = parent {
        get_node_global_matrix(model, parent, transform, local_matrices, global_matrices)
            * local_matrix
    } else {
        local_matrix
    };

    if node.flags & 8 != 0 {
        let pivot = model
            .pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or_default();
        let transformed_pivot = global_matrix * pivot;

        if let Some(parent) = parent {
            let parent_matrix =
                get_node_global_matrix(model, parent, transform, local_matrices, global_matrices);
            let rotation = shared::math3::Quaternion::from(parent_matrix);
            let inverse_rotation = -rotation;
            global_matrix =
                shared::math3::Matrix4::from_rotation_origin(inverse_rotation, transformed_pivot)
                    * global_matrix;
        }

        let view_matrix = shared::math3::Matrix4::identity().rotate(
            shared::math3::Vector3::from((30_f32.to_radians(), 0., 90_f32.to_radians())),
            shared::math3::RotationOrder::XYZ,
        ) * transform;
        let rotation = shared::math3::Quaternion::from(view_matrix);
        let inverse_rotation = -rotation;
        global_matrix =
            shared::math3::Matrix4::from_rotation_origin(inverse_rotation, transformed_pivot)
                * global_matrix;
    }

    global_matrices.insert(node.object_id, global_matrix);

    global_matrix
}

pub(super) fn evaluate_node_matrix(
    lerpfrac: f32,
    model: &mdx::Model,
    node: &mdx::Node,
    global_time: u32,
    frame1: u32,
    frame0: u32,
) -> math3::Matrix4 {
    let mut translation = math3::Vector3::default();
    let mut rotation = math3::Quaternion::default();
    let mut scale = math3::Vector3::from((1., 1., 1.));
    let pivot = model
        .pivots
        .get(node.object_id as usize)
        .copied()
        .unwrap_or_default();

    if frame0 != frame1 {
        if let Some(track) = &node.translation {
            let left = track.get_value(global_time, frame0).unwrap_or_default();
            let right = track.get_value(global_time, frame1).unwrap_or_default();
            translation = shared::math3::Vector3::lerp(left, right, lerpfrac);
        }
        if let Some(track) = &node.rotation {
            let left = track.get_value(global_time, frame0).unwrap_or_default();
            let right = track.get_value(global_time, frame1).unwrap_or_default();
            rotation = shared::math3::Quaternion::slerp(left, right, lerpfrac);
        }
        if let Some(track) = &node.scale {
            let left = track
                .get_value(global_time, frame0)
                .unwrap_or((1., 1., 1.).into());
            let right = track
                .get_value(global_time, frame1)
                .unwrap_or((1., 1., 1.).into());
            scale = shared::math3::Vector3::lerp(left, right, lerpfrac);
        }
    } else {
        translation = node
            .translation
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame1))
            .unwrap_or_default();
        rotation = node
            .rotation
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame1))
            .unwrap_or_default();
        scale = node
            .scale
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame1))
            .unwrap_or((1., 1., 1.).into());
    }

    match (
        node.translation.is_some(),
        node.rotation.is_some(),
        node.scale.is_some(),
    ) {
        (false, false, false) => shared::math3::Matrix4::identity(),
        (true, false, false) => shared::math3::Matrix4::from_translation(translation),
        (false, true, false) => shared::math3::Matrix4::from_rotation_origin(rotation, pivot),
        _ => shared::math3::Matrix4::from_rotation_translation_scale_origin(
            rotation,
            translation,
            scale,
            pivot,
        ),
    }
}
