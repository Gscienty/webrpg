use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    Node, Tracks,
    utils::{fixed_string, read_value},
};

const KATV: u32 = u32::from_le_bytes(*b"KATV");

#[derive(Debug)]
pub struct Attachment {
    pub node: sync::Arc<Node>,
    pub path: sync::Arc<str>,
    pub attachment_id: u32,
    pub visibility: Option<Tracks<f32>>,
}

impl Attachment {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid attachment size"))?;
        let mut payload = reader.take(payload_size as u64);

        let node = Node::read(&mut payload)?;
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
                result.visibility = Some(Tracks::read(&mut payload)?);
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
    fn reads_attachment_list() {
        let mut bytes = Vec::new();
        bytes.extend(388_u32.to_le_bytes());
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        bytes[8..19].copy_from_slice(b"Chest Ref\0x");
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        let path_start = bytes.len();
        bytes.extend([0; 256]);
        bytes[path_start..path_start + 10].copy_from_slice(b"sword.mdx\0");
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(b"KATV");
        for value in [1_u32, TrackType::NoInterp as u32, 5, 6] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(0.75_f32.to_le_bytes());

        bytes.extend(364_u32.to_le_bytes());
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        for value in [7_u32, 8, 9] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([0; 256]);
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(10_u32.to_le_bytes());

        let attachments = Attachment::read_list(&mut Cursor::new(bytes), 752).unwrap();

        assert_eq!(attachments.len(), 2);
        assert_eq!(&*attachments[0].path, "sword.mdx");
        assert_eq!(attachments[0].attachment_id, 4);
        assert_eq!(
            attachments[0].visibility.as_ref().unwrap().values[0].data,
            0.75
        );
        assert_eq!(attachments[1].attachment_id, 10);
        assert!(attachments[1].visibility.is_none());
    }
}
