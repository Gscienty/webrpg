use std::{
    io::{self, Read},
    sync,
};

use crate::mdx::{Node, utils::read_value};

#[derive(Debug)]
pub struct Bone {
    pub node: sync::Arc<Node>,
    pub geoset_id: u32,
    pub geoset_animation_id: u32,
}

impl Bone {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let node = Node::read(reader)?;
        let [geoset_id, geoset_animation_id] = read_value(reader)?;

        Ok(Self {
            node,
            geoset_id,
            geoset_animation_id,
        })
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

    #[test]
    fn reads_bone_list() {
        let mut bytes = Vec::new();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        bytes[4..9].copy_from_slice(b"Bone\0");
        for value in [1_u32, 2, 3, 4, 5] {
            bytes.extend(value.to_le_bytes());
        }

        let bones = Bone::read_list(&mut Cursor::new(bytes), 104).unwrap();

        assert_eq!(bones.len(), 1);
        assert_eq!(&*bones[0].node.name, "Bone");
        assert_eq!(bones[0].geoset_id, 4);
        assert_eq!(bones[0].geoset_animation_id, 5);
    }
}
