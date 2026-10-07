use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    Node, Tracks,
    utils::{fixed_string_bytes, read_value},
};

const KPEE: u32 = u32::from_le_bytes(*b"KPEE");
const KPEG: u32 = u32::from_le_bytes(*b"KPEG");
const KPEL: u32 = u32::from_le_bytes(*b"KPEL");
const KPEV: u32 = u32::from_le_bytes(*b"KPEV");
const KPLN: u32 = u32::from_le_bytes(*b"KPLN");
const KPLT: u32 = u32::from_le_bytes(*b"KPLT");
const KPES: u32 = u32::from_le_bytes(*b"KPES");

#[repr(C)]
#[derive(Clone, Copy)]
struct ParticleEmitterData {
    emission_rate: f32,
    gravity: f32,
    logitude: f32,
    latitude: f32,
    path: [u8; 260],
    lifespan: f32,
    speed: f32,
}

#[derive(Debug)]
pub struct ParticleEmitter {
    pub node: sync::Arc<Node>,
    pub emission_rate: f32,
    pub gravity: f32,
    pub logitude: f32,
    pub latitude: f32,
    pub path: sync::Arc<str>,
    pub lifespan: f32,
    pub speed: f32,

    pub emission_rate_tracks: Option<Tracks<f32>>,
    pub gravity_tracks: Option<Tracks<f32>>,
    pub logitude_tracks: Option<Tracks<f32>>,
    pub latitude_tracks: Option<Tracks<f32>>,
    pub lifespan_tracks: Option<Tracks<f32>>,
    pub speed_tracks: Option<Tracks<f32>>,
    pub visibility: Option<Tracks<f32>>,
}

impl ParticleEmitter {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid particle emitter size")
            })?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(&mut payload)?;
        let data: ParticleEmitterData = read_value(&mut payload)?;
        let mut result = Self {
            node,
            emission_rate: data.emission_rate,
            gravity: data.gravity,
            logitude: data.logitude,
            latitude: data.latitude,
            path: fixed_string_bytes(&data.path),
            lifespan: data.lifespan,
            speed: data.speed,
            emission_rate_tracks: None,
            gravity_tracks: None,
            logitude_tracks: None,
            latitude_tracks: None,
            lifespan_tracks: None,
            speed_tracks: None,
            visibility: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KPEE => result.emission_rate_tracks = Some(Tracks::read(&mut payload)?),
                KPEG => result.gravity_tracks = Some(Tracks::read(&mut payload)?),
                KPLN => result.logitude_tracks = Some(Tracks::read(&mut payload)?),
                KPLT => result.latitude_tracks = Some(Tracks::read(&mut payload)?),
                KPEL => result.lifespan_tracks = Some(Tracks::read(&mut payload)?),
                KPES => result.speed_tracks = Some(Tracks::read(&mut payload)?),
                KPEV => result.visibility = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported particle emitter track",
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

    #[test]
    fn reads_particle_emitter_tracks() {
        assert_eq!(size_of::<ParticleEmitterData>(), 284);

        let mut bytes = 552_u32.to_le_bytes().to_vec();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [4_f32, 5., 6., 7.] {
            bytes.extend(value.to_le_bytes());
        }
        let path_start = bytes.len();
        bytes.extend([0; 260]);
        bytes[path_start..path_start + 10].copy_from_slice(b"smoke.mdl\0");
        for value in [8_f32, 9.] {
            bytes.extend(value.to_le_bytes());
        }
        for (tag, value) in [
            (b"KPEE", 10_f32),
            (b"KPEG", 11.),
            (b"KPLN", 12.),
            (b"KPLT", 13.),
            (b"KPEL", 14.),
            (b"KPES", 15.),
            (b"KPEV", 16.),
        ] {
            bytes.extend(tag);
            for data in [1_u32, TrackType::NoInterp as u32, 0, 20] {
                bytes.extend(data.to_le_bytes());
            }
            bytes.extend(value.to_le_bytes());
        }

        let values = ParticleEmitter::read_list(&mut Cursor::new(bytes), 552).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(&*values[0].path, "smoke.mdl");
        assert_eq!(
            values[0].emission_rate_tracks.as_ref().unwrap().values[0].data,
            10.
        );
        assert_eq!(
            values[0].lifespan_tracks.as_ref().unwrap().values[0].data,
            14.
        );
        assert_eq!(values[0].visibility.as_ref().unwrap().values[0].data, 16.);
    }
}
