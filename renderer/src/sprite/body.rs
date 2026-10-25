use std::collections::BTreeMap;

use fileformat::mdx;
use shared::{color, math3};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    Error, shader,
    sprite::{
        RenderParams,
        bound::Bound,
        material::{self, Material},
        utils,
    },
};

pub struct Body<'s> {
    pub(super) bound: Bound,

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
    pub lights: [math3::Matrix4; 8],
    pub lights_count: u32,
}

const MAX_SKIN_BONES: usize = 4;

struct Geoset<'s> {
    geoset: &'s mdx::Geoset<'s>,
    vertex_array: Vec<WebGlVertexArrayObject>,
    triangles_count: usize,
    index_bytes_offset: usize,
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

    pub fn destroy(self, gl: &WebGl2RenderingContext) {
        for geoset in &self.geosets {
            for vertex_array in &geoset.vertex_array {
                gl.delete_vertex_array(Some(vertex_array));
            }
        }
        gl.delete_buffer(Some(&self.vertex_buffer));
        gl.delete_buffer(Some(&self.index_buffer));
        if let Some(uv_buffer) = &self.uv_buffer {
            gl.delete_buffer(Some(uv_buffer));
        }
    }

    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
        param: BodyRenderParams,
    ) -> Result<(), Error> {
        gl.depth_func(WebGl2RenderingContext::LEQUAL);

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
            lights: param.lights,
            lights_count: param.lights_count,
            layer_alpha: 1.,

            ..Default::default()
        };

        for geoset in self.geosets.iter() {
            if !geoset.visible(render_param.global_time, render_param.frame) {
                continue;
            }

            if render_param
                .render_phase
                .includes_pass(false, translucent_instance)
                && geoset.material.has_pass(false)
            {
                geoset.render(gl, &param, render_param, state.clone(), tint, false)?;
            }
        }

        let layer_count = self
            .geosets
            .iter()
            .fold(0, |acc, geoset| acc + geoset.material.layers_count());

        if layer_count == 0
            || !render_param
                .render_phase
                .includes_pass(true, translucent_instance)
        {
            return Ok(());
        }

        let mut draw_order = Vec::with_capacity(self.geosets.len());
        for geoset in self.geosets.iter() {
            if !geoset.visible(render_param.global_time, render_param.frame) {
                continue;
            }

            if !geoset.material.has_pass(true) {
                continue;
            }

            draw_order.push(GeosetDrawOrder {
                geoset,
                layer: 0,
                order: draw_order.len(),
            })
        }
        draw_order.sort_by(|lhs, rhs| {
            let left_blend = lhs
                .geoset
                .material
                .layer(lhs.layer as usize)
                .map_or(shared::BlendMode::None as u32, |layer| {
                    layer.blend_mode as u32
                });
            let right_blend = rhs
                .geoset
                .material
                .layer(lhs.layer as usize)
                .map_or(shared::BlendMode::None as u32, |layer| {
                    layer.blend_mode as u32
                });

            lhs.geoset
                .material
                .priority()
                .cmp(&rhs.geoset.material.priority())
                .then_with(|| left_blend.cmp(&right_blend))
                .then_with(|| lhs.order.cmp(&rhs.order))
        });

        for draw in draw_order.into_iter() {
            draw.geoset
                .render(gl, &param, render_param, state.clone(), tint, true)?;
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
                geoset,
                vertex_array,
                triangles_count: geoset.triangles.len(),
                index_bytes_offset: index_offset * size_of::<u16>(),
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

        let color = self
            .geoset
            .evaluate_color(render_param.global_time, render_param.frame);
        state.geoset_color = (
            color[0] * tint[0],
            color[1] * tint[1],
            color[2] * tint[2],
            color[3] * tint[3],
        )
            .into();
        state.bind_geoset_matrix_palette(body_param.bone_matrices, body_param.model, &self.palette);

        for (id, layer) in self.material.layers().enumerate() {
            if layer.is_blended() != blended_pass {
                continue;
            }
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            state.alpha_key = false;

            if render_param.no_face_cull || body_param.model.cameras.is_empty() {
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

            let alpha = layer.evaluate_alpha(
                self.material.material,
                render_param.global_time,
                render_param.frame,
            );
            if alpha < f32::EPSILON {
                continue;
            }
            state.layer_alpha = alpha;
            state.uv_matrix =
                layer.bind_texture_animation(render_param.global_time, render_param.frame);

            let model_texture_id =
                layer.evaluate_texture_id(render_param.global_time, render_param.frame);
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
        self.geoset.visible(global_time, frame)
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

pub struct GeosetDrawOrder<'s> {
    geoset: &'s Geoset<'s>,
    layer: u32,
    order: usize,
}
