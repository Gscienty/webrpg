use std::io::{self, Read};

use crate::mdx::{
    Node,
    utils::{read_value, read_values},
};

#[derive(Clone, Copy, Debug)]
pub enum CollisionShapeType {
    Box_ = 0,
    Plane = 1,
    Sphere = 2,
    Cylinder = 3,
}

#[derive(Debug)]
pub struct CollisionShape {
    pub node: Node,
    pub type_: CollisionShapeType,
    pub vertices: Vec<shared::math3::Vector3>,
    pub radius: Option<f32>,
}

impl TryFrom<u32> for CollisionShapeType {
    type Error = io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Box_),
            1 => Ok(Self::Plane),
            2 => Ok(Self::Sphere),
            3 => Ok(Self::Cylinder),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid collision shape type",
            )),
        }
    }
}

impl CollisionShape {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let node = Node::read(reader)?;
        let type_ = CollisionShapeType::try_from(read_value::<u32>(reader)?)?;
        let vertex_count = if !matches!(type_, CollisionShapeType::Sphere) {
            2
        } else {
            1
        };
        let vertices = read_values(reader, vertex_count)?;
        let radius = if matches!(
            type_,
            CollisionShapeType::Sphere | CollisionShapeType::Cylinder
        ) {
            Some(read_value(reader)?)
        } else {
            None
        };

        Ok(Self {
            node,
            type_,
            vertices,
            radius,
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

    fn push_node(bytes: &mut Vec<u8>) {
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 92]);
    }

    #[test]
    fn reads_collision_shapes() {
        let mut bytes = Vec::new();
        push_node(&mut bytes);
        bytes.extend((CollisionShapeType::Box_ as u32).to_le_bytes());
        for value in 1..=6 {
            bytes.extend((value as f32).to_le_bytes());
        }
        push_node(&mut bytes);
        bytes.extend((CollisionShapeType::Sphere as u32).to_le_bytes());
        for value in [7_f32, 8., 9., 10.] {
            bytes.extend(value.to_le_bytes());
        }

        let values = CollisionShape::read_list(&mut Cursor::new(bytes), 240).unwrap();

        assert_eq!(values.len(), 2);
        assert_eq!(values[0].vertices.len(), 2);
        assert_eq!(values[0].radius, None);
        assert_eq!(values[1].vertices, [(7., 8., 9.).into()]);
        assert_eq!(values[1].radius, Some(10.));
    }
}
