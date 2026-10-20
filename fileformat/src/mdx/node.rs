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

const KGRT: u32 = u32::from_le_bytes(*b"KGRT");
const KGSC: u32 = u32::from_le_bytes(*b"KGSC");
const KGTR: u32 = u32::from_le_bytes(*b"KGTR");

#[derive(Debug)]
pub struct Node<'s> {
    pub name: sync::Arc<str>,
    pub object_id: u32,
    pub parent_id: u32,
    pub flags: u32,

    pub translation: Option<Tracks<'s, math3::Vector3>>,
    pub rotation: Option<Tracks<'s, math3::Quaternion>>,
    pub scale: Option<Tracks<'s, math3::Vector3>>,
}

#[repr(C)]
struct NodeData {
    name: [u8; 80],
    object_id: u32,
    parent_id: u32,
    flags: u32,
}

impl<'s> Node<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid node size"))?;
        if payload_size < size_of::<NodeData>() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid node size",
            ));
        }

        let mut payload = reader.take(payload_size as u64);
        let data: NodeData = read_value(&mut payload)?;
        let mut result = Self {
            name: fixed_string_bytes(&data.name),
            object_id: data.object_id,
            parent_id: data.parent_id,
            flags: data.flags,
            translation: None,
            rotation: None,
            scale: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KGTR => result.translation = Some(Tracks::read(model, &mut payload)?),
                KGRT => result.rotation = Some(Tracks::read(model, &mut payload)?),
                KGSC => result.scale = Some(Tracks::read(model, &mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported node track",
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
