use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::model::mdx::{Node, Tracks, utils::read_value};

const KRAL: u32 = u32::from_le_bytes(*b"KRAL");
const KRCO: u32 = u32::from_le_bytes(*b"KRCO");
const KRHA: u32 = u32::from_le_bytes(*b"KRHA");
const KRHB: u32 = u32::from_le_bytes(*b"KRHB");
const KRTX: u32 = u32::from_le_bytes(*b"KRTX");
const KRVS: u32 = u32::from_le_bytes(*b"KRVS");

#[repr(C)]
struct RibbonEmitterData {
    height_above: f32,
    height_below: f32,
    alpha: f32,
    color: shared::math3::Vector3,
    lifespan: f32,
    texture_slot: u32,
    emission_rate: u32,
    rows: u32,
    columns: u32,
    material_id: u32,
    gravity: f32,
}

#[derive(Debug)]
pub struct RibbonEmitter {
    pub node: sync::Arc<Node>,
    pub height_above: f32,
    pub height_below: f32,
    pub alpha: f32,
    pub color: shared::math3::Vector3,
    pub lifespan: f32,
    pub texture_slot: u32,
    pub emission_rate: u32,
    pub rows: u32,
    pub columns: u32,
    pub material_id: u32,
    pub gravity: f32,

    pub visibility: Option<Tracks<f32>>,
    pub height_above_tracks: Option<Tracks<f32>>,
    pub height_below_tracks: Option<Tracks<f32>>,
    pub alpha_tracks: Option<Tracks<f32>>,
    pub color_tracks: Option<Tracks<shared::math3::Vector3>>,
    pub texture_tracks: Option<Tracks<u32>>,
}

impl RibbonEmitter {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid ribbon emitter size")
            })?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(&mut payload)?;
        let data: RibbonEmitterData = read_value(&mut payload)?;
        let mut result = Self {
            node,
            height_above: data.height_above,
            height_below: data.height_below,
            alpha: data.alpha,
            color: data.color,
            lifespan: data.lifespan,
            texture_slot: data.texture_slot,
            emission_rate: data.emission_rate,
            rows: data.rows,
            columns: data.columns,
            material_id: data.material_id,
            gravity: data.gravity,
            visibility: None,
            height_above_tracks: None,
            height_below_tracks: None,
            alpha_tracks: None,
            color_tracks: None,
            texture_tracks: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KRVS => result.visibility = Some(Tracks::read(&mut payload)?),
                KRHA => result.height_above_tracks = Some(Tracks::read(&mut payload)?),
                KRHB => result.height_below_tracks = Some(Tracks::read(&mut payload)?),
                KRAL => result.alpha_tracks = Some(Tracks::read(&mut payload)?),
                KRCO => result.color_tracks = Some(Tracks::read(&mut payload)?),
                KRTX => result.texture_tracks = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported ribbon emitter track",
                    ));
                }
            }
        }

        Ok(result)
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
    use std::io::Cursor;

    use super::*;
    use crate::model::mdx::TrackType;

    #[test]
    fn reads_ribbon_emitter_tracks() {
        let mut bytes = 304_u32.to_le_bytes().to_vec();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        for value in 1..=7 {
            bytes.extend((value as f32).to_le_bytes());
        }
        for value in 8_u32..=12 {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(13_f32.to_le_bytes());

        for (tag, value) in [
            (b"KRVS", 14_f32),
            (b"KRHA", 15.),
            (b"KRHB", 16.),
            (b"KRAL", 17.),
        ] {
            bytes.extend(tag);
            for data in [1_u32, TrackType::NoInterp as u32, 0, 20] {
                bytes.extend(data.to_le_bytes());
            }
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"KRCO");
        for data in [1_u32, TrackType::NoInterp as u32, 0, 20] {
            bytes.extend(data.to_le_bytes());
        }
        for value in [18_f32, 19., 20.] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"KRTX");
        for data in [1_u32, TrackType::NoInterp as u32, 0, 20, 21] {
            bytes.extend(data.to_le_bytes());
        }

        let values = RibbonEmitter::read_list(&mut Cursor::new(bytes), 304).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].color, (4., 5., 6.).into());
        assert_eq!(values[0].visibility.as_ref().unwrap().values[0].data, 14.);
        assert_eq!(
            values[0].color_tracks.as_ref().unwrap().values[0].data,
            (18., 19., 20.).into()
        );
        assert_eq!(
            values[0].texture_tracks.as_ref().unwrap().values[0].data,
            21
        );
    }
}
