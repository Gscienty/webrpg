use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{
    bounds::Bounds,
    utils::{fixed_string, read_value},
};

#[derive(Debug, Default)]
pub struct Info {
    pub name: sync::Arc<str>,
    pub animation_file: sync::Arc<str>,
    pub bounds: Bounds,
    pub blend_time: u32,
}

#[repr(C)]
struct InfoData {
    bounds: Bounds,
    blend_time: u32,
}

impl Info {
    const BYTE_SIZE: usize = 80 + 260 + size_of::<InfoData>();

    pub fn read(reader: &mut impl Read, size: usize) -> io::Result<Self> {
        if size != Self::BYTE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid model info size",
            ));
        }

        let name = fixed_string(reader, 80)?;
        let animation_file = fixed_string(reader, 260)?;

        let data: InfoData = read_value(reader)?;

        Ok(Self {
            name,
            animation_file,
            bounds: data.bounds,
            blend_time: data.blend_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_info_strings_and_data() {
        let mut bytes = vec![0; 340];
        bytes[..10].copy_from_slice(b"Footman\0xx");
        bytes[80..90].copy_from_slice(b"model.mdx\0");
        for value in 1..=7 {
            bytes.extend((value as f32).to_le_bytes());
        }
        bytes.extend(8_u32.to_le_bytes());

        let info = Info::read(&mut Cursor::new(bytes), Info::BYTE_SIZE).unwrap();

        assert_eq!(&*info.name, "Footman");
        assert_eq!(&*info.animation_file, "model.mdx");
        assert_eq!(info.bounds.radius, 1.);
        assert_eq!(info.blend_time, 8);
        assert!(Info::read(&mut Cursor::new([]), 0).is_err());
    }
}
