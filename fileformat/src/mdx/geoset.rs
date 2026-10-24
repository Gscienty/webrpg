use std::{
    io::{self, Read},
    mem::size_of,
};

use shared::math3::{self, Line3Intersect};

use crate::mdx::{
    bounds::Bounds,
    geoset_animation::GeosetAnimation,
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
    pub fn visible(&self, global_time: u32, frame: u32) -> bool {
        let Some(animation) = self.anim else {
            return true;
        };
        let mut alpha = if animation.alpha.is_infinite() {
            1.
        } else {
            animation.alpha
        };
        if let Some(value) = animation
            .geoset_alphas
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            alpha = value;
        }
        alpha.clamp(0., 1.) >= f32::EPSILON
    }

    pub fn trace(&self, line: &math3::Line3) -> Option<math3::Vector3> {
        self.triangles
            .chunks_exact(3)
            .filter_map(|indices| {
                let triangle = math3::Triangle3::from((
                    *self.vertices.get(indices[0] as usize)?,
                    *self.vertices.get(indices[1] as usize)?,
                    *self.vertices.get(indices[2] as usize)?,
                ));
                line.intersect(triangle)
            })
            .min_by(|left, right| {
                line.start()
                    .distance(*left)
                    .total_cmp(&line.start().distance(*right))
            })
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traces_the_nearest_triangle_and_ignores_invalid_indices() {
        let geoset = Geoset {
            vertices: vec![
                (-1., -1., 0.).into(),
                (1., -1., 0.).into(),
                (0., 1., 0.).into(),
                (-1., -1., -1.).into(),
                (1., -1., -1.).into(),
                (0., 1., -1.).into(),
            ],
            triangles: vec![3, 4, 5, 0, 1, 2, 99, 1, 2],
            normals: Vec::new(),
            texcoord: Vec::new(),
            primitive_types: Vec::new(),
            primitive_counts: Vec::new(),
            vertex_groups: Vec::new(),
            matrix_group_sizes: Vec::new(),
            matrices: Vec::new(),
            material_id: 0,
            group: 0,
            selectable: 0,
            default_bounds: Bounds::default(),
            bounds: Vec::new(),
            anim: None,
        };
        let line = math3::Line3::from(((0., 0., 1.).into(), (0., 0., -2.).into()));

        assert_eq!(geoset.trace(&line), Some((0., 0., 0.).into()));
        assert!(geoset.visible(0, 0));
    }
}
