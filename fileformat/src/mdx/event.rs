use std::io::{self, Cursor, Read};

use crate::mdx::{
    model::Model,
    node::Node,
    utils::{read_value, read_values},
};

const KEVT: u32 = u32::from_le_bytes(*b"KEVT");

#[derive(Debug)]
pub struct Event<'s> {
    pub node: Node<'s>,
    pub event_track: Option<EventTrack>,
}

#[derive(Debug)]
pub struct EventTrack {
    pub global_sequence_id: u32,
    pub keys: Vec<u32>,
}

impl<'s> Event<'s> {
    pub fn read(model: &'s Model, reader: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let node = Node::read(model, reader)?;
        let mut event_track = None;
        if reader.position() + 4 <= reader.get_ref().len() as u64 {
            let position = reader.position();
            let tag: u32 = read_value(reader)?;
            if tag == KEVT {
                let [count, global_sequence_id]: [u32; 2] = read_value(reader)?;
                event_track = Some(EventTrack {
                    global_sequence_id,
                    keys: read_values(reader, count as usize)?,
                });
            } else {
                reader.set_position(position);
            }
        }

        Ok(Self { node, event_track })
    }

    pub fn read_list(
        model: &'s Model,
        reader: &mut impl Read,
        size: usize,
    ) -> io::Result<Vec<Self>> {
        let mut bytes = vec![0; size];
        reader.read_exact(&mut bytes)?;
        let mut cursor = Cursor::new(bytes.as_slice());
        let mut values = Vec::new();
        while cursor.position() < size as u64 {
            values.push(Self::read(model, &mut cursor)?);
        }
        Ok(values)
    }
}
