use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    bounds::Bounds,
    utils::{fixed_string_bytes, read_value},
};

#[derive(Debug)]
pub struct Sequence {
    pub name: sync::Arc<str>,
    pub interval: [u32; 2],
    pub movespeed: f32,
    pub flags: u32,
    pub rarity: f32,
    pub syncpoint: i32,
    pub bounds: Bounds,
}

#[repr(C)]
struct SequenceData {
    name: [u8; 80],
    interval: [u32; 2],
    movespeed: f32,
    flags: u32,
    rarity: f32,
    syncpoint: i32,
    bounds: Bounds,
}

impl Sequence {
    const BYTE_SIZE: usize = size_of::<SequenceData>();

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let data: SequenceData = read_value(reader)?;

        Ok(Self {
            name: fixed_string_bytes(&data.name),
            interval: data.interval,
            movespeed: data.movespeed,
            flags: data.flags,
            rarity: data.rarity,
            syncpoint: data.syncpoint,
            bounds: data.bounds,
        })
    }

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        if !size.is_multiple_of(Self::BYTE_SIZE) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "sequence list size is not aligned",
            ));
        }

        (0..size / Self::BYTE_SIZE)
            .map(|_| Self::read(reader))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_sequence_and_truncates_name() {
        let mut bytes = vec![0; 80];
        bytes[..12].copy_from_slice(b"Stand\0hidden");
        for value in [1_u32, 2] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(3_f32.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(5_f32.to_le_bytes());
        bytes.extend(6_i32.to_le_bytes());
        for value in 7..=13 {
            bytes.extend((value as f32).to_le_bytes());
        }

        let sequence = Sequence::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(&*sequence.name, "Stand");
        assert_eq!(sequence.interval, [1, 2]);
        assert_eq!(sequence.movespeed, 3.);
        assert_eq!(sequence.flags, 4);
        assert_eq!(sequence.rarity, 5.);
        assert_eq!(sequence.syncpoint, 6);
        assert_eq!(sequence.bounds.radius, 7.);
        assert_eq!(sequence.bounds.box_.max, (11., 12., 13.).into());
    }

    #[test]
    fn reads_list_by_byte_size() {
        let bytes = vec![0; Sequence::BYTE_SIZE * 2];
        let sequences =
            Sequence::read_list(&mut Cursor::new(bytes), Sequence::BYTE_SIZE * 2).unwrap();

        assert_eq!(sequences.len(), 2);
        assert!(Sequence::read_list(&mut Cursor::new([]), 1).is_err());
    }
}
