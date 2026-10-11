use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    Error, buffer,
    model::mdx::{
        Bounds, GeosetAnimation,
        utils::{read_list, read_value},
    },
    shader,
};

const GNDX: u32 = u32::from_le_bytes(*b"GNDX");
const MATS: u32 = u32::from_le_bytes(*b"MATS");
const MTGC: u32 = u32::from_le_bytes(*b"MTGC");
const NRMS: u32 = u32::from_le_bytes(*b"NRMS");
const PCNT: u32 = u32::from_le_bytes(*b"PCNT");
const PTYP: u32 = u32::from_le_bytes(*b"PTYP");
const PVTX: u32 = u32::from_le_bytes(*b"PVTX");
const UVAS: u32 = u32::from_le_bytes(*b"UVAS");
const UVBS: u32 = u32::from_le_bytes(*b"UVBS");
const VRTX: u32 = u32::from_le_bytes(*b"VRTX");
const MAX_SKIN_BONES: usize = 4;

#[derive(Debug)]
pub struct MDXGeoset {
    pub vertices: Vec<shared::math3::Vector3>,
    pub normals: Vec<shared::math3::Vector3>,
    pub texcoord: Vec<Vec<shared::math3::Vector2>>,
    pub primitive_types: Vec<u32>,
    pub primitive_counts: Vec<u32>,
    pub triangles: Vec<u16>,
    pub vertex_groups: Vec<u8>,
    pub matrix_group_sizes: Vec<u32>,

    pub matrices: Vec<u32>,
    pub material_id: u32,
    pub group: u32,
    pub selectable: u32,
    pub default_bounds: Bounds,
    pub bounds: Vec<Bounds>,

    pub anim: Option<sync::Arc<GeosetAnimation>>,
}

fn expect_tag(reader: &mut impl Read, expected: u32) -> io::Result<()> {
    let tag: u32 = read_value(reader)?;
    if tag == expected {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected geoset tag",
        ))
    }
}

fn missing_tag() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "missing geoset tag")
}

impl MDXGeoset {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid geoset size"))?;
        let mut payload = reader.take(payload_size as u64);

        let mut vertices = None;
        let mut normals = None;
        let mut texcoord = None;
        let mut primitive_types = None;
        let mut primitive_counts = None;
        let mut triangles = None;
        let mut vertex_groups = None;
        let mut matrix_group_sizes = None;
        let mut materials = None;

        while payload.limit() > 0 {
            let tag: u32 = read_value(&mut payload)?;
            match tag {
                VRTX => vertices = Some(read_list(&mut payload)?),
                NRMS => normals = Some(read_list(&mut payload)?),
                PTYP => primitive_types = Some(read_list(&mut payload)?),
                PCNT => primitive_counts = Some(read_list(&mut payload)?),
                PVTX => triangles = Some(read_list(&mut payload)?),
                GNDX => vertex_groups = Some(read_list(&mut payload)?),
                MTGC => matrix_group_sizes = Some(read_list(&mut payload)?),
                MATS => {
                    let matrices = read_list(&mut payload)?;
                    let ids = read_value(&mut payload)?;
                    let default_bounds = Bounds::read(&mut payload)?;
                    let bounds = read_list(&mut payload)?;
                    materials = Some((matrices, ids, default_bounds, bounds));
                }
                UVAS => {
                    let uv_count: u32 = read_value(&mut payload)?;
                    let mut layers = Vec::new();
                    for _ in 0..uv_count {
                        expect_tag(&mut payload, UVBS)?;
                        layers.push(read_list(&mut payload)?);
                    }
                    texcoord = Some(layers);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported geoset tag",
                    ));
                }
            }
        }

        let (matrices, [material_id, group, selectable], default_bounds, bounds) =
            materials.ok_or_else(missing_tag)?;

        Ok(Self {
            vertices: vertices.ok_or_else(missing_tag)?,
            normals: normals.ok_or_else(missing_tag)?,
            texcoord: texcoord.ok_or_else(missing_tag)?,
            primitive_types: primitive_types.ok_or_else(missing_tag)?,
            primitive_counts: primitive_counts.ok_or_else(missing_tag)?,
            triangles: triangles.ok_or_else(missing_tag)?,
            vertex_groups: vertex_groups.ok_or_else(missing_tag)?,
            matrix_group_sizes: matrix_group_sizes.ok_or_else(missing_tag)?,
            matrices,
            material_id,
            group,
            selectable,
            default_bounds,
            bounds,

            anim: None,
        })
    }

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(&mut payload)?);
        }
        Ok(values)
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

    fn pack_vertices(&self) -> buffer::Vertices {
        let matrix_group_count = if self.matrix_group_sizes.is_empty() || self.matrices.is_empty() {
            1
        } else {
            self.matrix_group_sizes.len()
        };
        let mut matrix_groups = vec![[0_u8; MAX_SKIN_BONES]; matrix_group_count];
        let mut matrix_palette = Vec::with_capacity(128);
        let mut matrix_offset = 0;

        for (group_index, matrix_group) in matrix_groups.iter_mut().enumerate() {
            if matrix_group_count == 1
                && (self.matrix_group_sizes.is_empty() || self.matrices.is_empty())
            {
                matrix_group[0] = Self::add_geoset_matrix_palette_entry(&mut matrix_palette, 0);
                continue;
            }

            let source_group_size = self.matrix_group_sizes[group_index] as usize;
            let group_size = source_group_size
                .min(MAX_SKIN_BONES)
                .min(self.matrices.len().saturating_sub(matrix_offset));
            (0..group_size).for_each(|matrix_index| {
                matrix_group[matrix_index] = Self::add_geoset_matrix_palette_entry(
                    &mut matrix_palette,
                    self.matrices[matrix_offset + matrix_index],
                );
            });
            matrix_offset = matrix_offset.saturating_add(source_group_size);
        }

        self.vertices
            .iter()
            .enumerate()
            .map(|(vertex_index, &position)| {
                let mut matrix_group_index = 0;
                let mut matrix_group_size = 1;
                let mut leftover = 255_u8;
                let mut leftover_size = 1_u8;

                if !self.vertex_groups.is_empty() && !self.matrix_group_sizes.is_empty() {
                    matrix_group_index = self.vertex_groups[vertex_index] as usize;
                    matrix_group_index = matrix_group_index.min(matrix_group_count - 1);
                    matrix_group_size = (self.matrix_group_sizes[matrix_group_index] as usize)
                        .clamp(1, MAX_SKIN_BONES)
                        .min(self.matrices.len());
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

                buffer::Vertex {
                    position,
                    normal: self.normals[vertex_index],
                    texcoord: self.texcoord[0][vertex_index],
                    color: shared::color::Color32::white(),
                    skin,
                    bone_weight,
                }
            })
            .collect::<Vec<_>>()
            .into()
    }

    pub fn pack_geometry(&mut self) -> (buffer::Vertices, Vec<u32>) {
        (
            self.pack_vertices(),
            self.triangles.iter().map(|&index| index as u32).collect(),
        )
    }
}

#[derive(Debug)]
pub struct Geoset {
    pub property: MDXGeoset,
    pub vao: WebGlVertexArrayObject,
    pub index_offset: usize,
}

impl Geoset {
    pub fn pack_geometry(
        gl: &WebGl2RenderingContext,
        ibo: &WebGlBuffer,
        geoset: MDXGeoset,
        base: usize,
    ) -> Result<(Self, buffer::Vertices, Vec<u32>), Error> {
        let vertices = geoset.pack_vertices();
        let indices = geoset.triangles.iter().map(|&index| index as u32).collect();

        let vao = gl
            .create_vertex_array()
            .ok_or(shader::Error::CreateVertexArrayFailed)?;
        gl.bind_vertex_array(Some(&vao));
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(ibo));

        buffer::Vertices::bind_attributes(gl, base);

        Ok((
            Self {
                property: geoset,
                vao,
                index_offset: base,
            },
            vertices,
            indices,
        ))
    }
}
