use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{Node, Tracks, utils::read_value};

const KLAE: u32 = u32::from_le_bytes(*b"KLAE");
const KLAI: u32 = u32::from_le_bytes(*b"KLAI");
const KLAC: u32 = u32::from_le_bytes(*b"KLAC");
const KLAS: u32 = u32::from_le_bytes(*b"KLAS");
const KLAV: u32 = u32::from_le_bytes(*b"KLAV");
const KLBC: u32 = u32::from_le_bytes(*b"KLBC");
const KLBI: u32 = u32::from_le_bytes(*b"KLBI");

#[derive(Clone, Copy, Debug)]
pub enum LightType {
    Omni = 0,
    Direct = 1,
    Ambient = 2,
}

#[repr(C)]
struct LightData {
    type_: u32,
    attenuation_start: f32,
    attenuation_end: f32,
    color: shared::math3::Vector3,
    intensity: f32,
    ambient_color: shared::math3::Vector3,
    ambient_intensity: f32,
}

#[derive(Debug)]
pub struct Light {
    pub node: sync::Arc<Node>,
    pub type_: LightType,
    pub attenuation_start: f32,
    pub attenuation_end: f32,
    pub color: shared::math3::Vector3,
    pub intensity: f32,
    pub ambient_color: shared::math3::Vector3,
    pub ambient_intensity: f32,

    pub visibility_tracks: Option<Tracks<f32>>,
    pub color_tracks: Option<Tracks<shared::math3::Vector3>>,
    pub intensity_tracks: Option<Tracks<f32>>,
    pub ambient_color_tracks: Option<Tracks<shared::math3::Vector3>>,
    pub ambient_intensity_tracks: Option<Tracks<f32>>,
    pub attenuation_start_tracks: Option<Tracks<f32>>,
    pub attenuation_end_tracks: Option<Tracks<f32>>,
}

impl Light {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid light size"))?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(&mut payload)?;
        let data: LightData = read_value(&mut payload)?;
        let mut result = Self {
            node,
            type_: match data.type_ {
                0 => LightType::Omni,
                1 => LightType::Direct,
                2 => LightType::Ambient,

                _ => LightType::Omni,
            },
            attenuation_start: data.attenuation_start,
            attenuation_end: data.attenuation_end,
            color: data.color,
            intensity: data.intensity,
            ambient_color: data.ambient_color,
            ambient_intensity: data.ambient_intensity,
            visibility_tracks: None,
            color_tracks: None,
            intensity_tracks: None,
            ambient_color_tracks: None,
            ambient_intensity_tracks: None,
            attenuation_start_tracks: None,
            attenuation_end_tracks: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KLAV => result.visibility_tracks = Some(Tracks::read(&mut payload)?),
                KLAC => result.color_tracks = Some(Tracks::read(&mut payload)?),
                KLAI => result.intensity_tracks = Some(Tracks::read(&mut payload)?),
                KLBC => result.ambient_color_tracks = Some(Tracks::read(&mut payload)?),
                KLBI => result.ambient_intensity_tracks = Some(Tracks::read(&mut payload)?),
                KLAS => result.attenuation_start_tracks = Some(Tracks::read(&mut payload)?),
                KLAE => result.attenuation_end_tracks = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported light track",
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
    use crate::mdx::TrackType;

    fn push_track(bytes: &mut Vec<u8>, tag: &[u8; 4], value: f32) {
        bytes.extend(tag);
        for data in [1_u32, TrackType::NoInterp as u32, 0, 20] {
            bytes.extend(data.to_le_bytes());
        }
        bytes.extend(value.to_le_bytes());
    }

    #[test]
    fn reads_light_tracks() {
        let mut bytes = 296_u32.to_le_bytes().to_vec();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 92]);
        bytes.extend((LightType::Direct as u32).to_le_bytes());
        for value in 1..=10 {
            bytes.extend((value as f32).to_le_bytes());
        }
        push_track(&mut bytes, b"KLAV", 11.);
        push_track(&mut bytes, b"KLAI", 12.);
        bytes.extend(b"KLBC");
        for data in [1_u32, TrackType::NoInterp as u32, 0, 20] {
            bytes.extend(data.to_le_bytes());
        }
        for value in [13_f32, 14., 15.] {
            bytes.extend(value.to_le_bytes());
        }
        push_track(&mut bytes, b"KLBI", 16.);
        push_track(&mut bytes, b"KLAS", 17.);
        push_track(&mut bytes, b"KLAE", 18.);

        let values = Light::read_list(&mut Cursor::new(bytes), 296).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].ambient_color, (7., 8., 9.).into());
        assert_eq!(
            values[0].ambient_color_tracks.as_ref().unwrap().values[0].data,
            (13., 14., 15.).into()
        );
        assert_eq!(
            values[0].attenuation_end_tracks.as_ref().unwrap().values[0].data,
            18.
        );
    }
}
