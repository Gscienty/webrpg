use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    Tracks,
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
pub struct Camera {
    pub name: sync::Arc<str>,
    pub position: shared::math3::Vector3,
    pub field_of_view: f32,
    pub far_clip: f32,
    pub near_clip: f32,
    pub target_position: shared::math3::Vector3,

    pub translation_tracks: Option<Tracks<shared::math3::Vector3>>,
    pub roll_tracks: Option<Tracks<f32>>,
    pub target_translation_tracks: Option<Tracks<shared::math3::Vector3>>,
}

impl Camera {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
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
                KCTR => result.translation_tracks = Some(Tracks::read(&mut payload)?),
                KTTR => result.target_translation_tracks = Some(Tracks::read(&mut payload)?),
                KCRL => result.roll_tracks = Some(Tracks::read(&mut payload)?),
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

    fn push_vector_track(bytes: &mut Vec<u8>, tag: &[u8; 4], values: [f32; 3]) {
        bytes.extend(tag);
        for value in [1_u32, TrackType::NoInterp as u32, 0, 20] {
            bytes.extend(value.to_le_bytes());
        }
        for value in values {
            bytes.extend(value.to_le_bytes());
        }
    }

    #[test]
    fn reads_camera_tracks() {
        let mut bytes = 208_u32.to_le_bytes().to_vec();
        bytes.extend([0; 80]);
        bytes[4..11].copy_from_slice(b"Camera\0");
        for value in 1..=9 {
            bytes.extend((value as f32).to_le_bytes());
        }
        push_vector_track(&mut bytes, b"KCTR", [10., 11., 12.]);
        push_vector_track(&mut bytes, b"KTTR", [13., 14., 15.]);
        bytes.extend(b"KCRL");
        for value in [1_u32, TrackType::NoInterp as u32, 0, 20] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(16_f32.to_le_bytes());

        let values = Camera::read_list(&mut Cursor::new(bytes), 208).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(&*values[0].name, "Camera");
        assert_eq!(values[0].target_position, (7., 8., 9.).into());
        assert_eq!(
            values[0].translation_tracks.as_ref().unwrap().values[0].data,
            (10., 11., 12.).into()
        );
        assert_eq!(values[0].roll_tracks.as_ref().unwrap().values[0].data, 16.);
    }
}
