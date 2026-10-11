use std::{
    io::{self, Read},
    mem::size_of,
};

use crate::model::mdx::{Tracks, utils::read_value};

const KTAR: u32 = u32::from_le_bytes(*b"KTAR");
const KTAS: u32 = u32::from_le_bytes(*b"KTAS");
const KTAT: u32 = u32::from_le_bytes(*b"KTAT");

#[derive(Debug)]
pub struct TextureAnimation {
    pub translation: Option<Tracks<shared::math3::Vector3>>,
    pub rotation: Option<Tracks<shared::math3::Quaternion>>,
    pub scale: Option<Tracks<shared::math3::Vector3>>,
}

impl TextureAnimation {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid texture animation size")
            })?;
        let mut payload = reader.take(payload_size as u64);
        let mut result = Self {
            translation: None,
            rotation: None,
            scale: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KTAT => result.translation = Some(Tracks::read(&mut payload)?),
                KTAR => result.rotation = Some(Tracks::read(&mut payload)?),
                KTAS => result.scale = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported texture animation track",
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
    fn reads_texture_animation_tracks() {
        let mut bytes = 104_u32.to_le_bytes().to_vec();
        for (tag, values) in [
            (b"KTAT", &[1_f32, 2., 3.][..]),
            (b"KTAR", &[4_f32, 5., 6., 7.][..]),
            (b"KTAS", &[8_f32, 9., 10.][..]),
        ] {
            bytes.extend(tag);
            for value in [1_u32, TrackType::NoInterp as u32, 0, 10] {
                bytes.extend(value.to_le_bytes());
            }
            for value in values {
                bytes.extend(value.to_le_bytes());
            }
        }

        let values = TextureAnimation::read_list(&mut Cursor::new(bytes), 104).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0].translation.as_ref().unwrap().values[0].data,
            (1., 2., 3.).into()
        );
        assert_eq!(
            values[0].rotation.as_ref().unwrap().values[0].data,
            (4., 5., 6., 7.).into()
        );
        assert_eq!(
            values[0].scale.as_ref().unwrap().values[0].data,
            (8., 9., 10.).into()
        );
    }
}
