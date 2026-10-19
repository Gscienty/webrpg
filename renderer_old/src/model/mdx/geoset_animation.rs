use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::model::mdx::{Tracks, utils::read_value};

const KGAC: u32 = u32::from_le_bytes(*b"KGAC");
const KGAO: u32 = u32::from_le_bytes(*b"KGAO");

#[derive(Debug)]
pub struct GeosetAnimation {
    pub alpha: f32,
    pub flags: u32,
    pub color: shared::math3::Vector3,
    pub geoset_id: u32,

    pub geoset_alphas: Option<Tracks<f32>>,
    pub geoset_colors: Option<Tracks<shared::math3::Vector3>>,
}

#[repr(C)]
struct GeosetAnimationData {
    alpha: f32,
    flags: u32,
    color: shared::math3::Vector3,
    geoset_id: u32,
}

impl GeosetAnimation {
    pub fn read(reader: &mut impl Read) -> io::Result<sync::Arc<Self>> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid geoset animation size")
            })?;
        if payload_size < size_of::<GeosetAnimationData>() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid geoset animation size",
            ));
        }

        let mut payload = reader.take(payload_size as u64);
        let data: GeosetAnimationData = read_value(&mut payload)?;
        let mut result = Self {
            alpha: data.alpha,
            flags: data.flags,
            color: data.color,
            geoset_id: data.geoset_id,
            geoset_alphas: None,
            geoset_colors: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KGAO => result.geoset_alphas = Some(Tracks::read(&mut payload)?),
                KGAC => result.geoset_colors = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported geoset animation track",
                    ));
                }
            }
        }

        Ok(sync::Arc::new(result))
    }

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<sync::Arc<Self>>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(&mut payload)?);
        }
        Ok(values)
    }
}
