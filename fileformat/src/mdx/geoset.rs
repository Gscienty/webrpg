use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use shared::math3;

use crate::mdx::{
    bounds::Bounds,
    geoset_animation::GeosetAnimation,
    model::Model,
    utils::{read_list, read_value},
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
pub struct Geoset<'s> {
    pub vertices: Vec<math3::Vector3>,
    pub normals: Vec<math3::Vector3>,
    pub texcoord: Vec<Vec<math3::Vector2>>,
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

    pub anim: Option<&'s GeosetAnimation<'s>>,
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

impl<'s> Geoset<'s> {
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
}
