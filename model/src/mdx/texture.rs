use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::utils::{fixed_string_bytes, read_value};

#[derive(Debug)]
pub struct Texture {
    pub replaceable_id: u32,
    pub path: sync::Arc<str>,
    pub flags: u32,

    pub texture_id: Option<usize>,
}

#[repr(C)]
struct TextureData {
    replaceable_id: u32,
    path: [u8; 260],
    flags: u32,
}

impl Texture {
    const BYTE_SIZE: usize = size_of::<TextureData>();

    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let data: TextureData = read_value(reader)?;
        Ok(Self {
            replaceable_id: data.replaceable_id,
            path: fixed_string_bytes(&data.path),
            flags: data.flags,

            texture_id: None,
        })
    }

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        if !size.is_multiple_of(Self::BYTE_SIZE) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "texture list size is not aligned",
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
    fn reads_texture_list() {
        let mut bytes = 1_u32.to_le_bytes().to_vec();
        let mut path = [0; 260];
        path[..15].copy_from_slice(b"Textures/a.blp\0");
        bytes.extend(path);
        bytes.extend(2_u32.to_le_bytes());

        let textures = Texture::read_list(&mut Cursor::new(bytes), Texture::BYTE_SIZE).unwrap();

        assert_eq!(textures.len(), 1);
        assert_eq!(textures[0].replaceable_id, 1);
        assert_eq!(&*textures[0].path, "Textures/a.blp");
        assert_eq!(textures[0].flags, 2);
        assert!(Texture::read_list(&mut Cursor::new([]), 1).is_err());
    }
}
