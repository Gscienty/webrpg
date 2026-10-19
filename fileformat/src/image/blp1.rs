use jpeg_decoder::{ColorTransform, Decoder, PixelFormat};
use shared::color::Color32;

use super::{
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
            _ if is_jpeg => BLPAlphaDepth::Depth0,
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

        let mip_levels = offsets
            .iter()
            .zip(&lengths)
            .take_while(|&(&offset, &length)| offset != 0 && length != 0)
            .count();

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

        let width = self.header.width >> mip_level;
        if width == 0 { 1 } else { width }
    }

    pub const fn height(&self, mut mip_level: usize) -> u32 {
        if mip_level >= self.infos.mip_levels {
            mip_level = self.infos.mip_levels - 1;
        }

        let height = self.header.height >> mip_level;
        if height == 0 { 1 } else { height }
    }

    pub const fn mip_level(&self) -> usize {
        self.infos.mip_levels
    }

    pub fn convert(&self, mip_level: usize, buffer: &[u8]) -> Result<Vec<Color32>, Error> {
        if mip_level >= self.infos.mip_levels {
            return Err(Error::MipOutOfBounds);
        }
        let offset = *self
            .header
            .offsets
            .get(mip_level)
            .ok_or(Error::MipOutOfBounds)?;
        let size = *self
            .header
            .lengths
            .get(mip_level)
            .ok_or(Error::MipOutOfBounds)?;
        let end = offset.checked_add(size).ok_or(Error::MipOutOfBounds)?;
        let chunk = buffer.get(offset..end).ok_or(Error::MipOutOfBounds)?;

        let width = self.width(mip_level);
        let height = self.height(mip_level);
        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(Error::InvalidDimensions)?;

        let expected_size = match self.format() {
            BLPFormat::PalettedNoAlpha => Some(pixel_count),
            BLPFormat::PalettedAlpha1 => pixel_count.checked_add(pixel_count.div_ceil(8)),
            BLPFormat::PalettedAlpha4 => pixel_count.checked_add(pixel_count.div_ceil(2)),
            BLPFormat::PalettedAlpha8 => pixel_count.checked_mul(2),
            _ => None,
        };
        let chunk = match expected_size {
            Some(expected) => chunk.get(..expected).ok_or(Error::MipOutOfBounds)?,
            None => chunk,
        };

        match self.format() {
            BLPFormat::Jpeg => self.convert_jpeg(width, chunk),
            BLPFormat::PalettedNoAlpha => Ok(self.convert_paletted_no_alpha(width, height, chunk)),
            BLPFormat::PalettedAlpha1 => Ok(self.convert_paletted_alpha1(width, height, chunk)),
            BLPFormat::PalettedAlpha4 => Ok(self.convert_paletted_alpha4(width, height, chunk)),
            BLPFormat::PalettedAlpha8 => Ok(self.convert_paletted_alpha8(width, height, chunk)),
            _ => unreachable!(),
        }
    }

    fn convert_jpeg(&self, width: u32, buffer: &[u8]) -> Result<Vec<Color32>, Error> {
        let BLP1InfoExtra::Jpeg(header) = &self.infos.extra else {
            unreachable!()
        };
        let mut jpeg = Vec::with_capacity(header.len() + buffer.len());
        if !header.is_empty() {
            jpeg.extend_from_slice(header);
        }
        jpeg.extend_from_slice(buffer);

        let mut decoder = Decoder::new(jpeg.as_slice());
        decoder.read_info().map_err(|_| Error::InvalidJpegData)?;
        let format = decoder.info().ok_or(Error::InvalidJpegData)?.pixel_format;
        if format == PixelFormat::CMYK32 {
            decoder.set_color_transform(ColorTransform::None);
        }
        let pixels = decoder.decode().map_err(|_| Error::InvalidJpegData)?;
        Ok(pixels_to_colors(&pixels, format, width as usize))
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

fn pixels_to_colors(pixels: &[u8], format: PixelFormat, width: usize) -> Vec<Color32> {
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
                red: pixel[0],
                green: pixel[1],
                blue: pixel[2],
                alpha: 255,
            })
            .collect(),
        // With ColorTransform::None jpeg-decoder returns each scanline as
        // BBB...GGG...RRR...AAA..., while WebGL needs interleaved RGBA pixels.
        PixelFormat::CMYK32 => pixels
            .chunks_exact(width * 4)
            .flat_map(|row| {
                (0..width).map(move |x| Color32 {
                    red: row[width * 2 + x],
                    green: row[width + x],
                    blue: row[x],
                    alpha: row[width * 3 + x],
                })
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
        for value in [4_u32].into_iter().chain(std::iter::repeat_n(0, 15)) {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        buffer.resize(mip_offset, 0);
        buffer[HEADER_SIZE + 4..HEADER_SIZE + 8].copy_from_slice(&[30, 20, 10, 40]);
        buffer.extend_from_slice(&[1, 1, 0xf0, 0]);

        let blp = match InternalBLPInfos::load(&buffer) {
            Ok(blp) => blp,
            Err(error) => panic!("failed to load test BLP1: {error}"),
        };
        assert_eq!((blp.width(0), blp.height(0), blp.mip_level()), (2, 1, 1));
        assert!(matches!(blp.format(), BLPFormat::PalettedAlpha4));
        assert_eq!(
            blp.convert(0, &buffer[..mip_offset + 2]),
            Err(Error::MipOutOfBounds)
        );
        assert_eq!(blp.convert(1, &buffer), Err(Error::MipOutOfBounds));
        assert_eq!(
            blp.convert(0, &buffer).unwrap(),
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

        let mut jpeg = vec![0; 39 * 4 + 4];
        jpeg[..4].copy_from_slice(b"BLP1");
        jpeg[8..12].copy_from_slice(&7_u32.to_le_bytes());
        assert!(matches!(
            InternalBLPInfos::load(&jpeg).map(|blp| blp.format()),
            Ok(BLPFormat::Jpeg)
        ));

        jpeg[4..8].copy_from_slice(&1_u32.to_le_bytes());
        assert!(matches!(
            InternalBLPInfos::load(&jpeg),
            Err(Error::InvalidAlphaDepth)
        ));
    }

    #[test]
    fn ignores_trailing_zero_length_mip() {
        const HEADER_SIZE: usize = 39 * 4;
        let mut buffer = vec![0; HEADER_SIZE + 5];
        buffer[..4].copy_from_slice(b"BLP1");
        buffer[8..12].copy_from_slice(&8_u32.to_le_bytes());
        buffer[12..16].copy_from_slice(&1_u32.to_le_bytes());
        buffer[16..20].copy_from_slice(&1_u32.to_le_bytes());
        buffer[28..32].copy_from_slice(&160_u32.to_le_bytes());
        buffer[32..36].copy_from_slice(&161_u32.to_le_bytes());
        buffer[92..96].copy_from_slice(&1_u32.to_le_bytes());

        let blp = InternalBLPInfos::load(&buffer).expect("valid BLP1 header");
        assert_eq!(blp.mip_level(), 1);
    }

    #[test]
    fn converts_decoded_pixels_to_color32() {
        let rgb = pixels_to_colors(&[10, 20, 30, 40, 50, 60], PixelFormat::RGB24, 2);
        assert_eq!(
            (rgb[0].red, rgb[0].green, rgb[0].blue, rgb[0].alpha),
            (10, 20, 30, 255)
        );
        assert_eq!(
            (rgb[1].red, rgb[1].green, rgb[1].blue, rgb[1].alpha),
            (40, 50, 60, 255)
        );

        let gray = pixels_to_colors(&[42], PixelFormat::L8, 1);
        assert_eq!(
            (gray[0].red, gray[0].green, gray[0].blue, gray[0].alpha),
            (42, 42, 42, 255)
        );

        let bgra_rows = pixels_to_colors(&[30, 70, 20, 60, 10, 50, 40, 80], PixelFormat::CMYK32, 2);
        assert_eq!(
            (
                bgra_rows[0].red,
                bgra_rows[0].green,
                bgra_rows[0].blue,
                bgra_rows[0].alpha
            ),
            (10, 20, 30, 40)
        );
        assert_eq!(
            (
                bgra_rows[1].red,
                bgra_rows[1].green,
                bgra_rows[1].blue,
                bgra_rows[1].alpha
            ),
            (50, 60, 70, 80)
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
                mip_levels: 10,
                extra: BLP1InfoExtra::Palette(Box::new(palette)),
            },
        };

        assert_eq!((blp.width(9), blp.height(9)), (1, 1));

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
