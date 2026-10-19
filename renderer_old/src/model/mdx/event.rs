use std::{
    io::{self, Cursor, Read},
    sync,
};

use crate::model::mdx::{
    Node,
    utils::{read_value, read_values},
};

const KEVT: u32 = u32::from_le_bytes(*b"KEVT");

#[derive(Debug)]
pub struct Event {
    pub node: sync::Arc<Node>,
    pub event_track: Option<EventTrack>,
}

#[derive(Debug)]
pub struct EventTrack {
    pub global_sequence_id: u32,
    pub keys: Vec<u32>,
}

impl Event {
    pub fn read(reader: &mut Cursor<&[u8]>) -> io::Result<Self> {
        let node = Node::read(reader)?;
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

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut bytes = vec![0; size];
        reader.read_exact(&mut bytes)?;
        let mut cursor = Cursor::new(bytes.as_slice());
        let mut values = Vec::new();
        while cursor.position() < size as u64 {
            values.push(Self::read(&mut cursor)?);
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_event_list() {
        let mut bytes = Vec::new();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        bytes[4..10].copy_from_slice(b"Event\0");
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"KEVT");
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        for value in [10_u32, 20, 30] {
            bytes.extend(value.to_le_bytes());
        }

        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 92]);

        let values = Event::read_list(&mut Cursor::new(bytes), 216).unwrap();

        assert_eq!(values.len(), 2);
        assert_eq!(&*values[0].node.name, "Event");
        let event_track = values[0].event_track.as_ref().unwrap();
        assert_eq!(event_track.global_sequence_id, 4);
        assert_eq!(event_track.keys, [10, 20, 30]);
        assert!(values[1].event_track.is_none());
    }
}
