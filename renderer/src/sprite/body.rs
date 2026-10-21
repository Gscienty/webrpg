use fileformat::mdx;
use shared::{
    color,
    math3::{self, FrustumContains},
};
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    Error, shader,
    sprite::{bound::Bound, render_params::RenderParams},
};

pub struct Body<'s> {
    bound: Bound,

    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,

    uv_buffer: Option<WebGlBuffer>,

    geosets: Vec<Geoset<'s>>,
}

const MAX_SKIN_BONES: usize = 4;

struct Geoset<'s> {
    vertex_array: Vec<WebGlVertexArrayObject>,
    index_bytes_offset: usize,
    animation: Option<&'s mdx::GeosetAnimation<'s>>,
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
        model: &'s mdx::Model<'_>,
        param: &RenderParams<'_>,
        transform: math3::Matrix4,
    ) {
        if !param.no_frustum_cull {
            let sphere = math3::Sphere3 {
                center: transform * self.bound.cube.center(),
                radius: self.bound.radius * param.scale,
            };

            if !param.frustum.contains(sphere)
                || param.frustum.contains((self.bound.cube, transform))
            {
                return;
            }
        }
    }
}

impl<'s> Geoset<'s> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gl: &WebGl2RenderingContext,
        geoset: &'s mdx::Geoset<'s>,
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
                index_bytes_offset: index_offset * size_of::<u16>(),
                animation: geoset.anim,
                palette,
            },
            vertices,
            indices,
        ))
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
