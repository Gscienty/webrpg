use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    Tracks,
    utils::{fixed_string_bytes, read_value},
};

const KGRT: u32 = u32::from_le_bytes(*b"KGRT");
const KGSC: u32 = u32::from_le_bytes(*b"KGSC");
const KGTR: u32 = u32::from_le_bytes(*b"KGTR");

#[derive(Debug)]
pub struct Node {
    pub name: sync::Arc<str>,
    pub object_id: u32,
    pub parent_id: u32,
    pub flags: u32,

    pub translation: Option<Tracks<shared::math3::Vector3>>,
    pub rotation: Option<Tracks<shared::math3::Quaternion>>,
    pub scale: Option<Tracks<shared::math3::Vector3>>,
}

#[repr(C)]
struct NodeData {
    name: [u8; 80],
    object_id: u32,
    parent_id: u32,
    flags: u32,
}

impl Node {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
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
                KGTR => result.translation = Some(Tracks::read(&mut payload)?),
                KGRT => result.rotation = Some(Tracks::read(&mut payload)?),
                KGSC => result.scale = Some(Tracks::read(&mut payload)?),
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
    fn reads_node_tracks() {
        let mut bytes = Vec::new();
        bytes.extend(196_u32.to_le_bytes());
        bytes.extend([0; 80]);
        bytes[4..9].copy_from_slice(b"Bone\0");
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }

        bytes.extend(b"KGTR");
        for value in [1_u32, TrackType::NoInterp as u32, 4, 5] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [1_f32, 2., 3.] {
            bytes.extend(value.to_le_bytes());
        }

        bytes.extend(b"KGRT");
        for value in [1_u32, TrackType::NoInterp as u32, 6, 7] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [4_f32, 5., 6., 7.] {
            bytes.extend(value.to_le_bytes());
        }

        bytes.extend(b"KGSC");
        for value in [1_u32, TrackType::NoInterp as u32, 8, 9] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [8_f32, 9., 10.] {
            bytes.extend(value.to_le_bytes());
        }

        let node = Node::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(&*node.name, "Bone");
        assert_eq!(node.object_id, 1);
        assert_eq!(
            node.translation.as_ref().unwrap().values[0].data,
            (1., 2., 3.).into()
        );
        assert_eq!(
            node.rotation.as_ref().unwrap().values[0].data,
            (4., 5., 6., 7.).into()
        );
        assert_eq!(
            node.scale.as_ref().unwrap().values[0].data,
            (8., 9., 10.).into()
        );
    }

    #[test]
    fn reads_node_list() {
        let mut bytes = 96_u32.to_le_bytes().to_vec();
        bytes.extend([0; 80]);
        bytes[4..11].copy_from_slice(b"Helper\0");
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }

        let nodes = Node::read_list(&mut Cursor::new(bytes), 96).unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(&*nodes[0].name, "Helper");
        assert_eq!(nodes[0].object_id, 1);
    }
}
