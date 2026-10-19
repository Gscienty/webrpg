use crate::texture::Error;

const DDS_HEADER_SIZE: u32 = 124;
const DDS_MAGIC: u32 = u32::from_le_bytes(*b"DDS ");
const DDS_PIXEL_FORMAT_SIZE: u32 = 32;
const DDS_FILE_HEADER_SIZE: usize = 4 + DDS_HEADER_SIZE as usize;
const DDS_PIXEL_FORMAT_OFFSET: usize = 76;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DDSHeader {
    header_size: usize,
    height: u32,
    width: u32,
    mipmap_count: usize,

    flags: u32,
    fourcc: u32,
    rgb_bit_count: u32,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
}

pub(super) struct DDSRGBLayout {
    pub bytes_per_pixel: usize,
    pub swap_red_blue: bool,
}

impl DDSHeader {
    pub fn load(buffer: &[u8]) -> Result<Self, Error> {
        if buffer.len() < DDS_FILE_HEADER_SIZE {
            return Err(Error::TruncatedHeader);
        }
        if read_u32(buffer, 0) != DDS_MAGIC {
            return Err(Error::InvalidMagic);
        }

        let header_size = read_u32(buffer, 4);
        if header_size != DDS_HEADER_SIZE {
            return Err(Error::InvalidHeaderSize);
        }

        if read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET) != DDS_PIXEL_FORMAT_SIZE {
            return Err(Error::InvalidPixelFormatSize);
        }

        Ok(Self {
            header_size: header_size as usize,
            height: read_u32(buffer, 12),
            width: read_u32(buffer, 16),
            mipmap_count: read_u32(buffer, 28) as usize,
            flags: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 4),
            fourcc: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 8),
            rgb_bit_count: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 12),
            red_mask: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 16),
            green_mask: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 20),
            blue_mask: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 24),
            alpha_mask: read_u32(buffer, DDS_PIXEL_FORMAT_OFFSET + 28),
        })
    }

    pub const fn is_four_cc(&self) -> bool {
        (self.flags & 0x4) != 0
    }

    pub const fn is_rgb(&self) -> bool {
        (self.flags & 0x40) != 0
    }

    pub const fn get_four_cc(&self) -> u32 {
        self.fourcc
    }

    pub const fn get_flags(&self) -> u32 {
        self.flags
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn mipmap_count(&self) -> usize {
        self.mipmap_count
    }

    pub const fn pixel_offset(&self) -> usize {
        4 + self.header_size
    }

    pub const fn rgb_layout(&self) -> Result<DDSRGBLayout, Error> {
        match (
            self.rgb_bit_count,
            self.red_mask,
            self.green_mask,
            self.blue_mask,
            self.alpha_mask,
        ) {
            (32, 0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0xff00_0000) => Ok(DDSRGBLayout {
                bytes_per_pixel: 4,
                swap_red_blue: true,
            }),
            (24, 0x00ff_0000, 0x0000_ff00, 0x0000_00ff, _) => Ok(DDSRGBLayout {
                bytes_per_pixel: 3,
                swap_red_blue: true,
            }),
            (32, 0x0000_00ff, 0x0000_ff00, 0x00ff_0000, 0xff00_0000) => Ok(DDSRGBLayout {
                bytes_per_pixel: 4,
                swap_red_blue: false,
            }),
            (24, 0x0000_00ff, 0x0000_ff00, 0x00ff_0000, _) => Ok(DDSRGBLayout {
                bytes_per_pixel: 3,
                swap_red_blue: false,
            }),
            _ => Err(Error::UnsupportedTextureFormat),
        }
    }
}

fn read_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dds_header_and_pixel_format() {
        let mut buffer = [0_u8; DDS_FILE_HEADER_SIZE];
        for (offset, value) in [
            (0, DDS_MAGIC),
            (4, DDS_HEADER_SIZE),
            (12, 256),
            (16, 512),
            (28, 9),
            (76, DDS_PIXEL_FORMAT_SIZE),
            (80, 0x4),
            (84, u32::from_le_bytes(*b"DXT5")),
            (88, 32),
            (92, 0x00ff_0000),
            (96, 0x0000_ff00),
            (100, 0x0000_00ff),
            (104, 0xff00_0000),
        ] {
            buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        let header = DDSHeader::load(&buffer).unwrap();
        assert_eq!(header.header_size, 124);
        assert_eq!(
            (header.width, header.height, header.mipmap_count),
            (512, 256, 9)
        );
        assert_eq!(
            (header.flags, header.fourcc),
            (0x4, u32::from_le_bytes(*b"DXT5"))
        );
        assert_eq!(header.rgb_bit_count, 32);
        assert_eq!(
            (
                header.red_mask,
                header.green_mask,
                header.blue_mask,
                header.alpha_mask
            ),
            (0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0xff00_0000)
        );

        let layout = header.rgb_layout().unwrap();
        assert_eq!((layout.bytes_per_pixel, layout.swap_red_blue), (4, true));

        let rgba = DDSHeader {
            red_mask: 0x0000_00ff,
            blue_mask: 0x00ff_0000,
            ..header
        };
        let layout = rgba.rgb_layout().unwrap();
        assert_eq!((layout.bytes_per_pixel, layout.swap_red_blue), (4, false));

        let rgb = DDSHeader {
            rgb_bit_count: 24,
            alpha_mask: 0,
            ..rgba
        };
        let layout = rgb.rgb_layout().unwrap();
        assert_eq!((layout.bytes_per_pixel, layout.swap_red_blue), (3, false));

        let unsupported = DDSHeader {
            rgb_bit_count: 16,
            ..header
        };
        assert_eq!(
            unsupported.rgb_layout().err(),
            Some(Error::UnsupportedTextureFormat)
        );
    }

    #[test]
    fn rejects_invalid_dds_headers() {
        assert_eq!(DDSHeader::load(&[]), Err(Error::TruncatedHeader));

        let mut buffer = [0_u8; DDS_FILE_HEADER_SIZE];
        assert_eq!(DDSHeader::load(&buffer), Err(Error::InvalidMagic));

        buffer[..4].copy_from_slice(b"DDS ");
        assert_eq!(DDSHeader::load(&buffer), Err(Error::InvalidHeaderSize));

        buffer[4..8].copy_from_slice(&DDS_HEADER_SIZE.to_le_bytes());
        assert_eq!(DDSHeader::load(&buffer), Err(Error::InvalidPixelFormatSize));
    }
}
