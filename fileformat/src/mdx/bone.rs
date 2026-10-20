use std::io::{self, Read};

use crate::mdx::{model::Model, node::Node, utils::read_value};

#[derive(Debug)]
pub struct Bone<'s> {
    pub node: Node<'s>,
    pub geoset_id: u32,
    pub geoset_animation_id: u32,
}

impl<'s> Bone<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let node = Node::read(model, reader)?;
        let [geoset_id, geoset_animation_id] = read_value(reader)?;

        Ok(Self {
            node,
            geoset_id,
            geoset_animation_id,
        })
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
