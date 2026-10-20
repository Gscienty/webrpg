use std::{
    io::{self, Read},
    mem::size_of,
};

use shared::math3;

use crate::mdx::{model::Model, track::Tracks, utils::read_value};

const KTAR: u32 = u32::from_le_bytes(*b"KTAR");
const KTAS: u32 = u32::from_le_bytes(*b"KTAS");
const KTAT: u32 = u32::from_le_bytes(*b"KTAT");

#[derive(Debug)]
pub struct TextureAnimation<'s> {
    pub translation: Option<Tracks<'s, math3::Vector3>>,
    pub rotation: Option<Tracks<'s, math3::Quaternion>>,
    pub scale: Option<Tracks<'s, math3::Vector3>>,
}

impl<'s> TextureAnimation<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
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
                KTAT => result.translation = Some(Tracks::read(model, &mut payload)?),
                KTAR => result.rotation = Some(Tracks::read(model, &mut payload)?),
                KTAS => result.scale = Some(Tracks::read(model, &mut payload)?),
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

    pub fn read_list(
        model: &'s Model,
        reader: &mut impl Read,
        size: usize,
    ) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}
