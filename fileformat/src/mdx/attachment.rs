use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    model::Model,
    node::Node,
    track::Tracks,
    utils::{fixed_string, read_value},
};

const KATV: u32 = u32::from_le_bytes(*b"KATV");

#[derive(Debug)]
pub struct Attachment<'s> {
    pub node: Node<'s>,
    pub path: sync::Arc<str>,
    pub attachment_id: u32,
    pub visibility: Option<Tracks<'s, f32>>,
}

impl<'s> Attachment<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid attachment size"))?;
        let mut payload = reader.take(payload_size as u64);

        let node = Node::read(model, &mut payload)?;
        let path = fixed_string(&mut payload, 256)?;
        let _: u32 = read_value(&mut payload)?;
        let attachment_id = read_value(&mut payload)?;
        let mut result = Self {
            node,
            path,
            attachment_id,
            visibility: None,
        };

        if payload.limit() > 0 {
            let tag: u32 = read_value(&mut payload)?;
            if tag == KATV {
                result.visibility = Some(Tracks::read(model, &mut payload)?);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported attachment track",
                ));
            }
        }

        if payload.limit() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid attachment size",
            ));
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
