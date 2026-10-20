use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use shared::math3;

use crate::mdx::{
    model::Model,
    track::Tracks,
    utils::{fixed_string_bytes, read_value},
};

const KCRL: u32 = u32::from_le_bytes(*b"KCRL");
const KCTR: u32 = u32::from_le_bytes(*b"KCTR");
const KTTR: u32 = u32::from_le_bytes(*b"KTTR");

#[repr(C)]
struct CameraData {
    name: [u8; 80],
    position: shared::math3::Vector3,
    field_of_view: f32,
    far_clip: f32,
    near_clip: f32,
    target_position: shared::math3::Vector3,
}

#[derive(Debug)]
pub struct Camera<'s> {
    pub name: sync::Arc<str>,
    pub position: shared::math3::Vector3,
    pub field_of_view: f32,
    pub far_clip: f32,
    pub near_clip: f32,
    pub target_position: shared::math3::Vector3,

    pub translation_tracks: Option<Tracks<'s, math3::Vector3>>,
    pub roll_tracks: Option<Tracks<'s, f32>>,
    pub target_translation_tracks: Option<Tracks<'s, math3::Vector3>>,
}

impl<'s> Camera<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid camera size"))?;
        if payload_size < size_of::<CameraData>() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid camera size",
            ));
        }

        let mut payload = reader.take(payload_size as u64);
        let data: CameraData = read_value(&mut payload)?;
        let mut result = Self {
            name: fixed_string_bytes(&data.name),
            position: data.position,
            field_of_view: data.field_of_view,
            far_clip: data.far_clip,
            near_clip: data.near_clip,
            target_position: data.target_position,
            translation_tracks: None,
            roll_tracks: None,
            target_translation_tracks: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KCTR => result.translation_tracks = Some(Tracks::read(model, &mut payload)?),
                KTTR => result.target_translation_tracks = Some(Tracks::read(model, &mut payload)?),
                KCRL => result.roll_tracks = Some(Tracks::read(model, &mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported camera track",
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
