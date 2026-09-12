use jpeg_decoder::{Decoder, PixelFormat};
use shared::color::Color32;

use crate::texture::{
    Error,
    blp::{BLPAlphaDepth, BLPFormat},
};

pub(super) struct BLP1Header {
    is_jpeg: bool,
    alpha_bits: BLPAlphaDepth,
    width: u32,
    height: u32,
    offsets: [usize; 16],
    lengths: [usize; 16],
}

pub(super) enum BLP1InfoExtra {
    Palette(Box<[Color32; 256]>),
    Jpeg(Vec<u8>),
}

pub(super) struct BLP1Infos {
    mip_levels: usize,
    extra: BLP1InfoExtra,
}

pub(super) struct InternalBLPInfos {
    header: BLP1Header,
    infos: BLP1Infos,
}

impl InternalBLPInfos {
    pub fn load(buffer: &[u8]) -> Result<Self, Error> {
        const HEADER_SIZE: usize = 39 * size_of::<u32>();
        if buffer.len() < HEADER_SIZE {
            return Err(Error::TruncatedHeader);
        }

        let mut cursor = 0_usize;
        let mut read_u32 = || {
            let end = cursor.checked_add(4).ok_or(Error::TruncatedHeader)?;
            let bytes = buffer.get(cursor..end).ok_or(Error::TruncatedHeader)?;
            cursor = end;
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        };

        let magic = read_u32()?;
        if magic != u32::from_le_bytes(*b"BLP1") {
            return Err(Error::InvalidMagic);
        }

        let is_jpeg = match read_u32()? {
            0 => true,
            1 => false,
            _ => return Err(Error::InvalidType),
        };
        let alpha_bits = match read_u32()? {
            0 => BLPAlphaDepth::Depth0,
            1 => BLPAlphaDepth::Depth1,
            4 => BLPAlphaDepth::Depth4,
            8 => BLPAlphaDepth::Depth8,
            _ => return Err(Error::InvalidAlphaDepth),
        };
        let width = read_u32()?;
        let height = read_u32()?;

        let _extra = read_u32()?;
        let _has_mipmaps = read_u32()? != 0;

        let mut offsets = [0; 16];
        for offset in &mut offsets {
            *offset = read_u32()? as usize;
        }
        let mut lengths = [0; 16];
        for length in &mut lengths {
            *length = read_u32()? as usize;
        }

        let mut mip_levels = 0;
        while offsets[mip_levels] != 0 && mip_levels < 16 {
            mip_levels += 1;
        }

        let extra = if is_jpeg {
            if buffer.len() < HEADER_SIZE + 4 {
                return Err(Error::MissingJpegHeaderSize);
            }
            let bytes = &buffer[HEADER_SIZE..HEADER_SIZE + 4];
            let size = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            let end = (HEADER_SIZE + 4)
                .checked_add(size)
                .filter(|&end| end <= buffer.len())
                .ok_or(Error::TruncatedJpegHeader)?;
            BLP1InfoExtra::Jpeg(buffer[HEADER_SIZE + 4..end].to_vec())
        } else {
            let end = HEADER_SIZE + 256 * size_of::<Color32>();
            if end > buffer.len() {
                return Err(Error::TruncatedPalette);
            }
            let palette = std::array::from_fn(|index| {
                let offset = HEADER_SIZE + index * 4;
                Color32 {
                    red: buffer[offset],
                    green: buffer[offset + 1],
                    blue: buffer[offset + 2],
                    alpha: buffer[offset + 3],
                }
            });
            BLP1InfoExtra::Palette(Box::new(palette))
        };

        Ok(Self {
            header: BLP1Header {
                is_jpeg,
                alpha_bits,
                width,
                height,
                offsets,
                lengths,
            },
            infos: BLP1Infos { mip_levels, extra },
        })
    }

    pub const fn format(&self) -> BLPFormat {
        if self.header.is_jpeg {
            return BLPFormat::Jpeg;
        }

        match self.header.alpha_bits {
            BLPAlphaDepth::Depth8 => BLPFormat::PalettedAlpha8,
            BLPAlphaDepth::Depth4 => BLPFormat::PalettedAlpha4,
            BLPAlphaDepth::Depth1 => BLPFormat::PalettedAlpha1,
            _ => BLPFormat::PalettedNoAlpha,
        }
    }

    pub const fn width(&self, mut mip_level: usize) -> u32 {
        if mip_level >= self.infos.mip_levels {
            mip_level = self.infos.mip_levels - 1;
        }

        self.header.width >> mip_level
    }

    pub const fn height(&self, mut mip_level: usize) -> u32 {
        if mip_level >= self.infos.mip_levels {
            mip_level = self.infos.mip_levels - 1;
        }

        self.header.height >> mip_level
    }

    pub const fn mip_level(&self) -> usize {
        self.infos.mip_levels
    }

    pub fn convert(&self, mip_level: usize, buffer: &[u8]) -> Vec<Color32> {
        let offset = self.header.offsets[mip_level];
        let size = self.header.lengths[mip_level];
        let chunk = &buffer[offset..offset + size];

        let width = self.width(mip_level);
        let height = self.height(mip_level);

        match self.format() {
            BLPFormat::Jpeg => self.convert_jpeg(chunk),
            BLPFormat::PalettedNoAlpha => self.convert_paletted_no_alpha(width, height, chunk),
            BLPFormat::PalettedAlpha1 => self.convert_paletted_alpha1(width, height, chunk),
            BLPFormat::PalettedAlpha4 => self.convert_paletted_alpha4(width, height, chunk),
            BLPFormat::PalettedAlpha8 => self.convert_paletted_alpha8(width, height, chunk),
            _ => unreachable!(),
        }
    }

    fn convert_jpeg(&self, buffer: &[u8]) -> Vec<Color32> {
        let BLP1InfoExtra::Jpeg(header) = &self.infos.extra else {
            unreachable!()
        };
        let mut jpeg = Vec::with_capacity(header.len() + buffer.len());
        if !header.is_empty() {
            jpeg.extend_from_slice(header);
        }
        jpeg.extend_from_slice(buffer);

        let mut decoder = Decoder::new(jpeg.as_slice());
        let pixels = decoder.decode().expect("invalid BLP JPEG data");
        let format = decoder.info().expect("missing JPEG metadata").pixel_format;
        pixels_to_colors(&pixels, format)
    }

    fn convert_paletted_no_alpha(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        let BLP1InfoExtra::Palette(palette) = &self.infos.extra else {
            unreachable!()
        };
        assert_eq!(buffer.len(), width as usize * height as usize);

        buffer
            .iter()
            .map(|&index| {
                let mut color = palette[index as usize];
                color.alpha = 255;
                color
            })
            .collect()
    }

    fn convert_paletted_alpha1(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        let BLP1InfoExtra::Palette(palette) = &self.infos.extra else {
            unreachable!()
        };
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count.div_ceil(8));

        indices
            .iter()
            .enumerate()
            .map(|(pixel, &index)| {
                let mut color = palette[index as usize];
                color.alpha = if alpha[pixel / 8] & (1 << (pixel % 8)) != 0 {
                    255
                } else {
                    0
                };
                color
            })
            .collect()
    }

    fn convert_paletted_alpha4(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        let BLP1InfoExtra::Palette(palette) = &self.infos.extra else {
            unreachable!()
        };
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count.div_ceil(2));

        indices
            .iter()
            .enumerate()
            .map(|(pixel, &index)| {
                let mut color = palette[index as usize];
                let packed = alpha[pixel / 2];
                let value = if pixel % 2 == 0 {
                    packed & 0x0f
                } else {
                    packed >> 4
                };
                color.alpha = (value << 4) | value;
                color
            })
            .collect()
    }

    fn convert_paletted_alpha8(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        let BLP1InfoExtra::Palette(palette) = &self.infos.extra else {
            unreachable!()
        };
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count);

        indices
            .iter()
            .zip(alpha)
            .map(|(&index, &alpha)| {
                let mut color = palette[index as usize];
                color.alpha = alpha;
                color
            })
            .collect()
    }
}

fn pixels_to_colors(pixels: &[u8], format: PixelFormat) -> Vec<Color32> {
    match format {
        PixelFormat::L8 => pixels
            .iter()
            .map(|&value| Color32 {
                red: value,
                green: value,
                blue: value,
                alpha: 255,
            })
            .collect(),
        PixelFormat::L16 => pixels
            .chunks_exact(2)
            .map(|pixel| {
                let value = (u16::from_ne_bytes([pixel[0], pixel[1]]) >> 8) as u8;
                Color32 {
                    red: value,
                    green: value,
                    blue: value,
                    alpha: 255,
                }
            })
            .collect(),
        PixelFormat::RGB24 => pixels
            .chunks_exact(3)
            .map(|pixel| Color32 {
                red: pixel[2],
                green: pixel[1],
                blue: pixel[0],
                alpha: 255,
            })
            .collect(),
        PixelFormat::CMYK32 => pixels
            .chunks_exact(4)
            .map(|pixel| {
                let inverted_cyan = pixel[0];
                let inverted_magenta = pixel[1];
                let inverted_yellow = pixel[2];
                let inverted_black = pixel[3];
                Color32 {
                    red: (u16::from(inverted_yellow) * u16::from(inverted_black) / 255) as u8,
                    green: (u16::from(inverted_magenta) * u16::from(inverted_black) / 255) as u8,
                    blue: (u16::from(inverted_cyan) * u16::from(inverted_black) / 255) as u8,
                    alpha: 255,
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_little_endian_blp1_header_and_palette() {
        const HEADER_SIZE: usize = 39 * 4;
        const PALETTE_SIZE: usize = 256 * 4;
        let mip_offset = HEADER_SIZE + PALETTE_SIZE;
        let mut buffer = Vec::new();
        for value in [u32::from_le_bytes(*b"BLP1"), 1, 4, 2, 1, 5, 0] {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for value in [mip_offset as u32]
            .into_iter()
            .chain(std::iter::repeat_n(0, 15))
        {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for value in [3_u32].into_iter().chain(std::iter::repeat_n(0, 15)) {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        buffer.resize(mip_offset, 0);
        buffer[HEADER_SIZE + 4..HEADER_SIZE + 8].copy_from_slice(&[30, 20, 10, 40]);
        buffer.extend_from_slice(&[1, 1, 0xf0]);

        let blp = match InternalBLPInfos::load(&buffer) {
            Ok(blp) => blp,
            Err(error) => panic!("failed to load test BLP1: {error}"),
        };
        assert_eq!((blp.width(0), blp.height(0), blp.mip_level()), (2, 1, 1));
        assert!(matches!(blp.format(), BLPFormat::PalettedAlpha4));
        assert_eq!(
            blp.convert(0, &buffer),
            vec![
                Color32 {
                    red: 30,
                    green: 20,
                    blue: 10,
                    alpha: 0,
                },
                Color32 {
                    red: 30,
                    green: 20,
                    blue: 10,
                    alpha: 255,
                },
            ]
        );
    }

    #[test]
    fn reports_invalid_blp1_data() {
        assert!(matches!(
            InternalBLPInfos::load(&[]),
            Err(Error::TruncatedHeader)
        ));
        assert!(matches!(
            InternalBLPInfos::load(&[0; 39 * 4]),
            Err(Error::InvalidMagic)
        ));
    }

    #[test]
    fn converts_decoded_pixels_to_color32() {
        let rgb = pixels_to_colors(&[10, 20, 30, 40, 50, 60], PixelFormat::RGB24);
        assert_eq!(
            (rgb[0].red, rgb[0].green, rgb[0].blue, rgb[0].alpha),
            (10, 20, 30, 255)
        );
        assert_eq!(
            (rgb[1].red, rgb[1].green, rgb[1].blue, rgb[1].alpha),
            (40, 50, 60, 255)
        );

        let gray = pixels_to_colors(&[42], PixelFormat::L8);
        assert_eq!(
            (gray[0].red, gray[0].green, gray[0].blue, gray[0].alpha),
            (42, 42, 42, 255)
        );

        let cmyk = pixels_to_colors(&[255, 128, 0, 255], PixelFormat::CMYK32);
        assert_eq!(
            (cmyk[0].red, cmyk[0].green, cmyk[0].blue, cmyk[0].alpha),
            (255, 128, 0, 255)
        );
    }

    #[test]
    fn converts_palette_indices_and_one_bit_alpha() {
        let mut palette = [Color32 {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }; 256];
        palette[1] = Color32 {
            red: 10,
            green: 20,
            blue: 30,
            alpha: 40,
        };
        let blp = InternalBLPInfos {
            header: BLP1Header {
                is_jpeg: false,
                alpha_bits: BLPAlphaDepth::Depth0,
                width: 2,
                height: 1,
                offsets: [0; 16],
                lengths: [0; 16],
            },
            infos: BLP1Infos {
                mip_levels: 1,
                extra: BLP1InfoExtra::Palette(Box::new(palette)),
            },
        };

        assert_eq!(
            blp.convert_paletted_no_alpha(2, 1, &[1, 0]),
            vec![
                Color32 {
                    red: 10,
                    green: 20,
                    blue: 30,
                    alpha: 255,
                },
                Color32 {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
            ]
        );

        let mut buffer = vec![1; 10];
        buffer.extend_from_slice(&[0b0101_0101, 0b0000_0010]);
        let colors = blp.convert_paletted_alpha1(10, 1, &buffer);
        assert_eq!(
            colors.iter().map(|color| color.alpha).collect::<Vec<_>>(),
            vec![255, 0, 255, 0, 255, 0, 255, 0, 0, 255]
        );

        let mut buffer = vec![1; 3];
        buffer.extend_from_slice(&[0xa3, 0x07]);
        let colors = blp.convert_paletted_alpha4(3, 1, &buffer);
        assert_eq!(
            colors.iter().map(|color| color.alpha).collect::<Vec<_>>(),
            vec![0x33, 0xaa, 0x77]
        );

        let colors = blp.convert_paletted_alpha8(3, 1, &[1, 1, 1, 0, 127, 255]);
        assert_eq!(
            colors.iter().map(|color| color.alpha).collect::<Vec<_>>(),
            vec![0, 127, 255]
        );
    }
}
