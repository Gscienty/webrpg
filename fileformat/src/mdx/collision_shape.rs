use std::{
    io::{self, Read},
    sync,
};

use shared::math3;

use crate::mdx::{
    model::Model,
    node::Node,
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
pub struct CollisionShape<'s> {
    pub node: Node<'s>,
    pub type_: CollisionShapeType,
    pub vertices: Vec<math3::Vector3>,
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

impl<'s> CollisionShape<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let node = Node::read(model, reader)?;
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

    pub fn read_list(model: &'s Model, reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}
