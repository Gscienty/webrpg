use std::collections::BTreeMap;

use fileformat::mdx;
use shared::{
    color,
    math3::{self, FrustumContains},
};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    Error, shader,
    sprite::{
        bound::Bound,
        material::{self, Material},
        render_params::RenderParams,
        utils,
    },
};

pub struct Body<'s> {
    bound: Bound,

    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,

    uv_buffer: Option<WebGlBuffer>,

    geosets: Vec<Geoset<'s>>,
}

pub struct BodyRenderParams<'a, 's> {
    pub model: &'s mdx::Model<'s>,
    pub bone_matrices: &'a BTreeMap<u32, math3::Matrix4>,
    pub textures: &'a [usize],
    pub transform: math3::Matrix4,
}

const MAX_SKIN_BONES: usize = 4;

struct Geoset<'s> {
    vertex_array: Vec<WebGlVertexArrayObject>,
    triangles_count: usize,
    index_bytes_offset: usize,
    animation: Option<&'s mdx::GeosetAnimation<'s>>,
    material: Material<'s>,
    palette: Vec<u32>,
}

impl<'s> Body<'s> {
    pub fn new(gl: &WebGl2RenderingContext, model: &'s mdx::Model<'s>) -> Result<Self, Error> {
        let bound = Bound {
            radius: model.bounds.radius,
            cube: model.bounds.cube,
        };

        let vertex_buffer = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        let index_buffer = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;

        let uvcount = model.geosets.iter().fold(0, |acc, geoset| {
            if geoset.texcoord.is_empty() {
                return acc;
            }
            acc + ((geoset.texcoord.len() - 1) * geoset.vertices.len())
        });
        let uv_buffer = if uvcount != 0 {
            let uvbuffer = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;

            Some(uvbuffer)
        } else {
            None
        };

        let mut geosets = Vec::with_capacity(model.geosets.len());

        let mut index_offset = 0;
        let mut vertex_offset = 0;
        let mut uv_offset = 0;

        let mut vertices: Vec<shader::Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut uvs: Vec<shared::math3::Vector2> = Vec::new();

        for mdx_geoset in model.geosets.iter() {
            let Some(material) = model.materials.get(mdx_geoset.material_id as usize) else {
                continue;
            };

            uvs.extend(
                mdx_geoset
                    .texcoord
                    .iter()
                    .skip(1)
                    .flat_map(|channel| channel.iter().copied()),
            );

            let (geoset, vertices_chunk, indices_chunk) = Geoset::new(
                gl,
                mdx_geoset,
                material,
                &vertex_buffer,
                &index_buffer,
                uv_buffer.as_ref(),
                vertex_offset,
                index_offset,
                uv_offset,
            )?;

            index_offset += indices_chunk.len();
            vertex_offset += vertices_chunk.len();

            if !mdx_geoset.texcoord.is_empty() {
                uv_offset += (mdx_geoset.texcoord.len() - 1) * mdx_geoset.vertices.len();
            }

            geosets.push(geoset);

            vertices.extend(vertices_chunk);
            indices.extend(indices_chunk);
        }

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            shader::Vertices::from(vertices).as_bytes(),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        if let Some(uv_buffer) = &uv_buffer {
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(uv_buffer));
            let uv_bytes = unsafe {
                std::slice::from_raw_parts(
                    uvs.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(uvs.as_slice()),
                )
            };
            gl.buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                uv_bytes,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        gl.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&index_buffer),
        );
        let index_bytes = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(indices.as_slice()),
            )
        };
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            index_bytes,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        Ok(Self {
            bound,

            geosets,

            vertex_buffer,
            index_buffer,

            uv_buffer,
        })
    }

    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
        param: BodyRenderParams,
    ) -> Result<(), Error> {
        if !render_param.no_frustum_cull {
            let sphere = math3::Sphere3 {
                center: param.transform * self.bound.cube.center(),
                radius: self.bound.radius * render_param.scale,
            };

            if !render_param.frustum.contains(sphere)
                || !render_param
                    .frustum
                    .contains((self.bound.cube, param.transform))
            {
                return Ok(());
            }
        }

        let (lights, lights_count) = pack_light(
            param.model,
            param.bone_matrices,
            render_param.light,
            param.transform,
            render_param.global_time,
            render_param.frame,
        );

        let tint: math3::Vector4 = (
            render_param.tint.red as f32 / 255.,
            render_param.tint.green as f32 / 255.,
            render_param.tint.blue as f32 / 255.,
            render_param.tint.alpha as f32 / 255.,
        )
            .into();
        let translucent_instance = tint[3] < 1. - f32::EPSILON;

        let state = shader::ModelState {
            model: param.transform,
            normal_matrix: param.transform.normal(),
            unshaded: render_param.no_lighting,
            view_projection: render_param.view_projection,
            texture_matrix: render_param.texture,
            light_matrix: render_param.light,
            fog_enable: render_param.fog.is_some(),
            fog_color: render_param.fog.map(|(color, _)| color).unwrap_or_default(),
            fog_params: render_param.fog.map(|(_, range)| range).unwrap_or_default(),
            lights,
            lights_count,
            layer_alpha: 1.,

            ..Default::default()
        };

        for blended_pass in [false, true] {
            if !render_param
                .render_phase
                .includes_pass(blended_pass, translucent_instance)
            {
                continue;
            }

            for geoset in self.geosets.iter() {
                if !geoset.visible(render_param.global_time, render_param.frame) {
                    continue;
                }

                if geoset.material.has_pass(blended_pass) {
                    geoset.render(gl, &param, render_param, state.clone(), tint, blended_pass)?;
                }
            }
        }

        Ok(())
    }
}

impl<'s> Geoset<'s> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gl: &WebGl2RenderingContext,
        geoset: &'s mdx::Geoset<'s>,
        material: &'s mdx::Material<'s>,
        vertex_buffer: &WebGlBuffer,
        index_buffer: &WebGlBuffer,
        uv_buffer: Option<&WebGlBuffer>,

        vertex_offset: usize,
        index_offset: usize,
        uv_offset: usize,
    ) -> Result<(Self, shader::Vertices, Vec<u16>), Error> {
        let (vertices, palette) = pack_vertices(geoset);

        let indices = geoset.triangles.clone();

        let channels = geoset.texcoord.len().max(1);

        let mut vertex_array = Vec::with_capacity(channels);
        for channel in 0..channels {
            let vao = gl
                .create_vertex_array()
                .ok_or(Error::CreateVertexArrayFailed)?;
            gl.bind_vertex_array(Some(&vao));
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(vertex_buffer));
            gl.bind_buffer(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                Some(index_buffer),
            );

            shader::Vertices::bind_attributes(gl, vertex_offset);

            if let Some(uv_buffer) = uv_buffer
                && channel != 0
            {
                gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(uv_buffer));
                gl.vertex_attrib_pointer_with_i32(
                    shader::AttributeID::Texcoord as u32,
                    2,
                    WebGl2RenderingContext::FLOAT,
                    false,
                    size_of::<shared::math3::Vector2>() as i32,
                    ((uv_offset + (channel - 1) * geoset.vertices.len())
                        * size_of::<shared::math3::Vector2>()) as i32,
                );
            }

            vertex_array.push(vao);
        }

        Ok((
            Self {
                vertex_array,
                triangles_count: geoset.triangles.len(),
                index_bytes_offset: index_offset * size_of::<u16>(),
                animation: geoset.anim,
                material: Material::new(material),
                palette,
            },
            vertices,
            indices,
        ))
    }

    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        body_param: &BodyRenderParams,
        render_param: &RenderParams,

        mut state: shader::ModelState,
        tint: math3::Vector4,
        blended_pass: bool,
    ) -> Result<(), Error> {
        if !self.material.has_pass(blended_pass) {
            return Ok(());
        }

        let color = self.evaluate_color(render_param.global_time, render_param.frame);
        state.geoset_color = (
            color[0] * tint[0],
            color[1] * tint[1],
            color[2] * tint[2],
            color[3] * tint[3],
        )
            .into();
        state.bind_geoset_matrix_palette(body_param.bone_matrices, body_param.model, &self.palette);

        for (id, layer) in self.material.layers().enumerate() {
            if material::layer_is_blended(layer) != blended_pass {
                continue;
            }
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            state.alpha_key = false;

            if body_param.model.cameras.is_empty() {
                gl.disable(WebGl2RenderingContext::CULL_FACE);
            } else {
                gl.enable(WebGl2RenderingContext::CULL_FACE);
                gl.cull_face(WebGl2RenderingContext::BACK);
            }
            gl.depth_mask(true);

            if !material::layer_set_blended(
                gl,
                id,
                layer,
                render_param.render_phase,
                render_param.msaa_samples,
                &mut state,
            ) {
                continue;
            }

            if tint[3] < 1. - f32::EPSILON
                && matches!(
                    layer.blend_mode,
                    shared::BlendMode::None | shared::BlendMode::AlphaKey
                )
            {
                state.alpha_key = false;
                utils::set_alpha_key_state(
                    gl,
                    render_param.render_phase,
                    render_param.msaa_samples,
                    false,
                );
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(
                    WebGl2RenderingContext::SRC_ALPHA,
                    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                );
                gl.depth_mask(false);
            }

            material::layer_apply_flags(gl, layer);

            state.unshaded =
                render_param.no_lighting || layer.flags & mdx::GeoFlag::Unshared as u32 != 0;
            state.fog_enable = render_param.fog.is_some()
                && layer.flags & mdx::GeoFlag::Unfogged as u32 == 0
                && matches!(
                    layer.blend_mode,
                    shared::BlendMode::None
                        | shared::BlendMode::AddAlpha
                        | shared::BlendMode::Blend
                );

            let alpha = self.material.evaluate_layer_alpha(
                layer,
                render_param.global_time,
                render_param.frame,
            );
            if alpha < f32::EPSILON {
                continue;
            }
            state.layer_alpha = alpha;
            state.uv_matrix = layer_bind_texture_animation(
                body_param.model,
                layer,
                render_param.global_time,
                render_param.frame,
            );

            let model_texture_id = evaluate_layer_texture_id(
                layer,
                body_param.model,
                render_param.global_time,
                render_param.frame,
            );
            let Some(model_texture) = body_param.model.textures.get(model_texture_id) else {
                continue;
            };
            let Some(&texture_id) = body_param.textures.get(model_texture_id) else {
                continue;
            };

            let Some(tex) = render_param.texture_set.get_texture(
                texture_id,
                render_param.team,
                model_texture.replaceable_id,
                render_param.override_texture,
                render_param.override_slot,
            ) else {
                continue;
            };
            tex.bind(gl, 0);

            let Some(vao) = self.vertex_array.get(layer.coord_id as usize) else {
                continue;
            };

            gl.bind_vertex_array(Some(vao));

            render_param.model_program.apply(gl, &state)?;

            gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                self.triangles_count as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                self.index_bytes_offset as i32,
            );
        }

        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.enable(WebGl2RenderingContext::CULL_FACE);
        utils::set_alpha_key_state(
            gl,
            render_param.render_phase,
            render_param.msaa_samples,
            false,
        );
        gl.cull_face(WebGl2RenderingContext::BACK);
        gl.depth_mask(true);

        Ok(())
    }

    pub fn visible(&self, global_time: u32, frame: u32) -> bool {
        if self.animation.is_some() {
            let color = self.evaluate_color(global_time, frame);
            return color[3] >= f32::EPSILON;
        }

        true
    }

    pub fn evaluate_color(&self, global_time: u32, frame: u32) -> math3::Vector4 {
        let Some(animation) = self.animation else {
            return (1., 1., 1., 1.).into();
        };

        let mut color: shared::math3::Vector4 = (
            1.,
            1.,
            1.,
            if animation.alpha.is_infinite() {
                1.
            } else {
                animation.alpha
            },
        )
            .into();

        if let Some(ref alphas) = animation.geoset_alphas
            && let Some(alpha) = alphas.get_value(global_time, frame)
        {
            color[3] = alpha;
        }

        if animation.flags & 0x02 != 0 {
            let geoset_color = if let Some(ref colors) = animation.geoset_colors
                && let Some(color) = colors.get_value(global_time, frame)
            {
                color
            } else {
                animation.color
            };

            color[0] = geoset_color[0];
            color[1] = geoset_color[1];
            color[2] = geoset_color[2];
        }

        color[0] = color[0].clamp(0., 1.);
        color[1] = color[1].clamp(0., 1.);
        color[2] = color[2].clamp(0., 1.);
        color[3] = color[3].clamp(0., 1.);

        color
    }
}

fn pack_vertices(geoset: &mdx::Geoset) -> (shader::Vertices, Vec<u32>) {
    let matrix_group_count = if geoset.matrix_group_sizes.is_empty() || geoset.matrices.is_empty() {
        1
    } else {
        geoset.matrix_group_sizes.len()
    };
    let mut matrix_groups = vec![[0_u8; MAX_SKIN_BONES]; matrix_group_count];
    let mut matrix_palette = Vec::with_capacity(128);
    let mut matrix_offset = 0;

    for (group_index, matrix_group) in matrix_groups.iter_mut().enumerate() {
        if matrix_group_count == 1
            && (geoset.matrix_group_sizes.is_empty() || geoset.matrices.is_empty())
        {
            matrix_group[0] = add_geoset_matrix_palette_entry(&mut matrix_palette, 0);
            continue;
        }

        let source_group_size = geoset.matrix_group_sizes[group_index] as usize;
        let group_size = source_group_size
            .min(MAX_SKIN_BONES)
            .min(geoset.matrices.len().saturating_sub(matrix_offset));
        (0..group_size).for_each(|matrix_index| {
            matrix_group[matrix_index] = add_geoset_matrix_palette_entry(
                &mut matrix_palette,
                geoset.matrices[matrix_offset + matrix_index],
            );
        });
        matrix_offset = matrix_offset.saturating_add(source_group_size);
    }

    let vertices = geoset
        .vertices
        .iter()
        .enumerate()
        .map(|(vertex_index, &position)| {
            let mut matrix_group_index = 0;
            let mut matrix_group_size = 1;
            let mut leftover = 255_u8;
            let mut leftover_size = 1_u8;

            if !geoset.vertex_groups.is_empty() && !geoset.matrix_group_sizes.is_empty() {
                matrix_group_index = geoset.vertex_groups[vertex_index] as usize;
                matrix_group_index = matrix_group_index.min(matrix_group_count - 1);
                matrix_group_size = (geoset.matrix_group_sizes[matrix_group_index] as usize)
                    .clamp(1, MAX_SKIN_BONES)
                    .min(geoset.matrices.len());
                leftover_size = matrix_group_size as u8;
            }

            let matrix_group = matrix_groups[matrix_group_index];
            let mut raw_weight = [0_u8; MAX_SKIN_BONES];
            if matrix_group_count == 1 && matrix_group[0] == 0 {
                raw_weight[0] = 255;
            } else {
                for weight in raw_weight.iter_mut().take(matrix_group_size) {
                    *weight = (leftover as f32 / leftover_size as f32) as u8;
                    leftover = leftover.saturating_sub(*weight);
                    leftover_size = leftover_size.saturating_sub(1).max(1);
                }
            }

            let mut skin = [0; MAX_SKIN_BONES];
            let mut bone_weight = [0; MAX_SKIN_BONES];
            for (matrix, weight) in matrix_group.into_iter().zip(raw_weight) {
                if weight == 0 {
                    continue;
                }
                if let Some(index) = bone_weight.iter().position(|&value| weight > value) {
                    skin[index..].rotate_right(1);
                    bone_weight[index..].rotate_right(1);
                    skin[index] = matrix;
                    bone_weight[index] = weight;
                }
            }

            let weight_sum: u32 = bone_weight.iter().map(|&weight| weight as u32).sum();
            if weight_sum != 0 && weight_sum != 255 {
                let mut normalized_sum = 0;
                for weight in &mut bone_weight {
                    *weight = (*weight as u32 * 255 / weight_sum) as u8;
                    normalized_sum += *weight as u32;
                }
                bone_weight[0] += (255 - normalized_sum) as u8;
            }

            shader::Vertex {
                position,
                normal: geoset.normals[vertex_index],
                texcoord: geoset.texcoord[0][vertex_index],
                color: color::Color32::white(),
                skin,
                bone_weight,
            }
        })
        .collect::<Vec<_>>()
        .into();

    (vertices, matrix_palette)
}

fn add_geoset_matrix_palette_entry(matrix_palette: &mut Vec<u32>, matrix_id: u32) -> u8 {
    if let Some(index) = matrix_palette.iter().position(|&value| value == matrix_id) {
        return index as u8;
    }

    if matrix_palette.len() >= 128 {
        return 0;
    }

    matrix_palette.push(matrix_id);

    (matrix_palette.len() - 1) as u8
}

fn evaluate_layer_texture_id(
    layer: &mdx::Layer,
    model: &mdx::Model,
    global_time: u32,
    frame: u32,
) -> usize {
    layer
        .flipbook_tracks
        .as_ref()
        .and_then(|track| track.get_value(global_time, frame))
        .filter(|&texture_id| (texture_id as usize) < model.textures.len())
        .unwrap_or(layer.texture_id) as usize
}

fn layer_bind_texture_animation(
    model: &mdx::Model,
    layer: &mdx::Layer,
    global_time: u32,
    frame: u32,
) -> math3::Matrix3 {
    let mut translation = shared::math3::Vector3::default();
    let mut rotation = shared::math3::Quaternion::default();
    let mut scale = shared::math3::Vector3::from((1., 1., 1.));

    if let Some(animation) = model.texture_animations.get(layer.transform_id as usize) {
        if let Some(value) = animation
            .translation
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            translation = value;
        }
        if let Some(value) = animation
            .rotation
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            rotation = value;
        }
        if let Some(value) = animation
            .scale
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            scale = value;
        }
    }

    if !translation[0].is_finite() || !translation[1].is_finite() {
        translation = shared::math3::Vector3::default();
    }
    if !rotation.z.is_finite() || !rotation.w.is_finite() {
        rotation = shared::math3::Quaternion::default();
    }
    if !scale[0].is_finite() || !scale[1].is_finite() {
        scale = shared::math3::Vector3::from((1., 1., 1.));
    }

    let c = rotation.w * rotation.w - rotation.z * rotation.z;
    let s = 2. * rotation.z * rotation.w;
    let tx = scale[0] * (c * (translation[0] - 0.5) - s * (translation[1] - 0.5)) + 0.5;
    let ty = scale[1] * (s * (translation[0] - 0.5) + c * (translation[1] - 0.5)) + 0.5;
    let mut matrix = shared::math3::Matrix3::zero();
    matrix[0] = (scale[0] * c, scale[1] * s, 0.).into();
    matrix[1] = (-scale[0] * s, scale[1] * c, 0.).into();
    matrix[2] = (tx, ty, 1.).into();
    matrix
}

fn pack_light(
    model: &mdx::Model,
    bone_matrices: &BTreeMap<u32, math3::Matrix4>,
    light_matrix: math3::Matrix4,
    transform: math3::Matrix4,
    global_time: u32,
    frame: u32,
) -> ([math3::Matrix4; 8], u32) {
    let mut lighting = shader::ModelLighting::default();

    lighting.lights.extend(collect_model_lights(
        model,
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
    let mut lights = [shared::math3::Matrix4::zero(); 8];
    lights[..lights_count].copy_from_slice(&packed_lights[..lights_count]);

    (lights, lights_count as u32)
}

fn collect_model_lights(
    model: &mdx::Model,
    bone_matrices: &BTreeMap<u32, math3::Matrix4>,
    transform: math3::Matrix4,
    global_time: u32,
    frame: u32,
) -> Vec<shader::ModelLight> {
    model
        .lights
        .iter()
        .filter_map(|light| {
            evaluate_light(
                model,
                light,
                bone_matrices,
                transform,
                global_time,
                frame,
                true,
            )
        })
        .collect()
}

fn evaluate_light(
    model: &mdx::Model,
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

    let pivot = model
        .pivots
        .get(light.node.object_id as usize)
        .copied()
        .unwrap_or_default();
    let mut local_pos = pivot;
    let mut local_dir_target = pivot + shared::math3::Vector3::from((0., 0., -1.));
    if let Some(&node_matrix) = bone_matrices.get(&light.node.object_id) {
        local_pos = node_matrix * local_pos;
        local_dir_target = node_matrix * local_dir_target;
    }

    let world_pos = transform * local_pos;
    let world_dir_target = transform * local_dir_target;
    let world_dir = world_dir_target - world_pos;
    let world_dir = if world_dir.length_sq() < f32::EPSILON {
        shared::math3::Vector3::from((0., 0., -1.))
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
