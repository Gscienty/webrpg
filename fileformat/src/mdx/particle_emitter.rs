use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    model::Model,
    node::Node,
    track::Tracks,
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
pub struct ParticleEmitter<'s> {
    pub node: Node<'s>,
    pub emission_rate: f32,
    pub gravity: f32,
    pub logitude: f32,
    pub latitude: f32,
    pub path: sync::Arc<str>,
    pub lifespan: f32,
    pub speed: f32,

    pub emission_rate_tracks: Option<Tracks<'s, f32>>,
    pub gravity_tracks: Option<Tracks<'s, f32>>,
    pub logitude_tracks: Option<Tracks<'s, f32>>,
    pub latitude_tracks: Option<Tracks<'s, f32>>,
    pub lifespan_tracks: Option<Tracks<'s, f32>>,
    pub speed_tracks: Option<Tracks<'s, f32>>,
    pub visibility: Option<Tracks<'s, f32>>,
}

impl<'s> ParticleEmitter<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid particle emitter size")
            })?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(model, &mut payload)?;
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
                KPEE => result.emission_rate_tracks = Some(Tracks::read(model, &mut payload)?),
                KPEG => result.gravity_tracks = Some(Tracks::read(model, &mut payload)?),
                KPLN => result.logitude_tracks = Some(Tracks::read(model, &mut payload)?),
                KPLT => result.latitude_tracks = Some(Tracks::read(model, &mut payload)?),
                KPEL => result.lifespan_tracks = Some(Tracks::read(model, &mut payload)?),
                KPES => result.speed_tracks = Some(Tracks::read(model, &mut payload)?),
                KPEV => result.visibility = Some(Tracks::read(model, &mut payload)?),
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

    pub fn read_list(model: &'s Model, reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}
