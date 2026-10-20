use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    bounds::Bounds,
    utils::{animation_ratio, fixed_string_bytes, read_value},
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

    pub fn frame(&self, anim: &str, animation_time: u32) -> u32 {
        let mut sequence_length = self.interval[1] - self.interval[0];
        if sequence_length == 0 {
            sequence_length = 1;
        }

        let Some(ratio) = animation_ratio(anim) else {
            if self.flags & 1 != 0 {
                return self.interval[0] + animation_time.min(self.interval[1] - self.interval[0]);
            }
            return self.interval[0] + (animation_time % sequence_length);
        };

        let mut offset = (ratio * sequence_length as f32) as u32;
        if offset >= sequence_length {
            offset = sequence_length - 1;
        }

        self.interval[0] + offset
    }
}
