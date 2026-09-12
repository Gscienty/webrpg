use shared::color::Color32;

use crate::texture::{
    Error,
    blp::{BLPAlphaDepth, BLPAlphaEncoding, BLPEncoding, BLPFormat},
};

pub(super) struct BLP2Header {
    is_jpeg: bool,

    encoding: BLPEncoding,
    alpha_depth: BLPAlphaDepth,
    alpha_encoding: BLPAlphaEncoding,
    mip_levels: usize,

    width: u32,
    height: u32,
    offsets: [usize; 16],
    lengths: [usize; 16],
    palette: [Color32; 256],
}

impl BLP2Header {
    pub fn load(buffer: &[u8]) -> Result<Self, Error> {
        const HEADER_SIZE: usize = 5 * 4 + 16 * 4 + 16 * 4 + 256 * 4;
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

        if read_u32()? != u32::from_le_bytes(*b"BLP2") {
            return Err(Error::InvalidMagic);
        }

        let is_jpeg = match read_u32()? {
            0 => true,
            1 => false,
            _ => return Err(Error::InvalidType),
        };

        let attributes = read_u32()?.to_le_bytes();
        let encoding = match attributes[0] {
            0 if is_jpeg => BLPEncoding::Uncompressed,
            1 => BLPEncoding::Uncompressed,
            2 => BLPEncoding::Dxt,
            3 => BLPEncoding::UncompressedRawBGRA,
            _ => return Err(Error::InvalidEncoding),
        };
        let alpha_depth = match attributes[1] {
            0 => BLPAlphaDepth::Depth0,
            1 => BLPAlphaDepth::Depth1,
            4 => BLPAlphaDepth::Depth4,
            8 => BLPAlphaDepth::Depth8,
            _ => return Err(Error::InvalidAlphaDepth),
        };
        let alpha_encoding = match attributes[2] {
            0 => BLPAlphaEncoding::Dxt1,
            1 => BLPAlphaEncoding::Dxt3,
            7 => BLPAlphaEncoding::Dxt5,
            _ => return Err(Error::InvalidAlphaEncoding),
        };
        let _has_mipmaps = attributes[3] != 0;

        let width = read_u32()?;
        let height = read_u32()?;

        let mut offsets = [0; 16];
        for offset in &mut offsets {
            *offset = read_u32()? as usize;
        }
        let mut lengths = [0; 16];
        for length in &mut lengths {
            *length = read_u32()? as usize;
        }
        let mip_levels = offsets.iter().take_while(|&&offset| offset != 0).count();

        let palette = std::array::from_fn(|_| {
            let color = Color32 {
                red: buffer[cursor + 2],
                green: buffer[cursor + 1],
                blue: buffer[cursor],
                alpha: buffer[cursor + 3],
            };
            cursor += 4;
            color
        });

        Ok(Self {
            is_jpeg,
            encoding,
            alpha_depth,
            alpha_encoding,
            mip_levels,
            width,
            height,
            offsets,
            lengths,
            palette,
        })
    }

    pub const fn width(&self, mut mip_level: usize) -> u32 {
        if mip_level >= self.mip_levels {
            mip_level = self.mip_levels - 1;
        }

        self.width >> mip_level
    }

    pub const fn height(&self, mut mip_level: usize) -> u32 {
        if mip_level >= self.mip_levels {
            mip_level = self.mip_levels - 1;
        }

        self.height >> mip_level
    }

    pub const fn mip_level(&self) -> usize {
        self.mip_levels
    }

    pub const fn format(&self) -> BLPFormat {
        if self.is_jpeg {
            return BLPFormat::Jpeg;
        }

        match self.encoding {
            BLPEncoding::Uncompressed => match self.alpha_depth {
                BLPAlphaDepth::Depth0 => BLPFormat::PalettedNoAlpha,
                BLPAlphaDepth::Depth1 => BLPFormat::PalettedAlpha1,
                BLPAlphaDepth::Depth4 => BLPFormat::PalettedAlpha4,
                BLPAlphaDepth::Depth8 => BLPFormat::PalettedAlpha8,
            },
            BLPEncoding::UncompressedRawBGRA => BLPFormat::RawBGRA,
            BLPEncoding::Dxt => match self.alpha_depth {
                BLPAlphaDepth::Depth0 => match self.alpha_encoding {
                    BLPAlphaEncoding::Dxt1 => BLPFormat::Dxt1NoAlpha,
                    BLPAlphaEncoding::Dxt3 => BLPFormat::Dxt3NoAlpha,
                    _ => unreachable!(),
                },
                BLPAlphaDepth::Depth1 => match self.alpha_encoding {
                    BLPAlphaEncoding::Dxt1 => BLPFormat::Dxt1Alpha1,
                    _ => unreachable!(),
                },
                BLPAlphaDepth::Depth4 => match self.alpha_encoding {
                    BLPAlphaEncoding::Dxt3 => BLPFormat::Dxt3Alpha4,
                    _ => unreachable!(),
                },
                BLPAlphaDepth::Depth8 => match self.alpha_encoding {
                    BLPAlphaEncoding::Dxt5 => BLPFormat::Dxt5Alpha8,
                    _ => unreachable!(),
                },
            },
        }
    }

    pub fn convert(&self, mip_level: usize, buffer: &[u8]) -> Vec<Color32> {
        let offset = self.offsets[mip_level];
        let size = self.lengths[mip_level];
        let chunk = &buffer[offset..offset + size];

        let width = self.width(mip_level);
        let height = self.height(mip_level);

        match self.format() {
            BLPFormat::Jpeg => unreachable!(),
            BLPFormat::PalettedNoAlpha => self.convert_paletted_no_alpha(width, height, chunk),
            BLPFormat::PalettedAlpha1 => self.convert_paletted_alpha1(width, height, chunk),
            BLPFormat::PalettedAlpha4 => self.convert_paletted_alpha4(width, height, chunk),
            BLPFormat::PalettedAlpha8 => self.convert_paletted_alpha8(width, height, chunk),
            BLPFormat::RawBGRA => self.convert_raw_bgra(width, height, chunk),
            BLPFormat::Dxt1NoAlpha => self.convert_dxt(width, height, 0, chunk),
            BLPFormat::Dxt1Alpha1 | BLPFormat::Dxt3NoAlpha => {
                self.convert_dxt(width, height, 1, chunk)
            }
            BLPFormat::Dxt3Alpha4 | BLPFormat::Dxt3Alpha8 => {
                self.convert_dxt(width, height, 3, chunk)
            }
            BLPFormat::Dxt5Alpha8 => self.convert_dxt(width, height, 5, chunk),
        }
    }

    fn convert_paletted_no_alpha(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        assert_eq!(buffer.len(), width as usize * height as usize);

        buffer
            .iter()
            .map(|&index| {
                let mut color = self.palette[index as usize];
                color.alpha = 255;
                color
            })
            .collect()
    }

    fn convert_paletted_alpha1(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count.div_ceil(8));

        indices
            .iter()
            .enumerate()
            .map(|(pixel, &index)| {
                let mut color = self.palette[index as usize];
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
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count.div_ceil(2));

        indices
            .iter()
            .enumerate()
            .map(|(pixel, &index)| {
                let mut color = self.palette[index as usize];
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
        let pixel_count = width as usize * height as usize;
        let (indices, alpha) = buffer.split_at(pixel_count);
        assert_eq!(alpha.len(), pixel_count);

        indices
            .iter()
            .zip(alpha)
            .map(|(&index, &alpha)| {
                let mut color = self.palette[index as usize];
                color.alpha = alpha;
                color
            })
            .collect()
    }

    fn convert_raw_bgra(&self, width: u32, height: u32, buffer: &[u8]) -> Vec<Color32> {
        assert_eq!(buffer.len(), width as usize * height as usize * 4);

        buffer
            .chunks_exact(4)
            .map(|pixel| Color32 {
                red: pixel[2],
                green: pixel[1],
                blue: pixel[0],
                alpha: pixel[3],
            })
            .collect()
    }

    fn convert_dxt(&self, width: u32, height: u32, format: u8, mut buffer: &[u8]) -> Vec<Color32> {
        let mut pixels = vec![Color32::default(); (width * height) as usize];
        let blocks_x = width.div_ceil(4);
        let blocks_y = height.div_ceil(4);

        (0..blocks_y).for_each(|by| {
            (0..blocks_x).for_each(|bx| {
                let (alpha, color_block) = if format == 3 {
                    let alpha = decode_alpha_dxt3(buffer);
                    let color_block = &buffer[8..];
                    buffer = &buffer[16..];

                    (alpha, color_block)
                } else if format == 5 {
                    let alpha = decode_alpha_dxt5(buffer);
                    let color_block = &buffer[8..];
                    buffer = &buffer[16..];

                    (alpha, color_block)
                } else {
                    let color_block = buffer;
                    buffer = &buffer[8..];

                    ([255; 16], color_block)
                };

                let colors = decode_color_block(color_block, format != 1);
                let indices = (color_block[4] as usize)
                    | ((color_block[5] as usize) << 8)
                    | ((color_block[6] as usize) << 16)
                    | ((color_block[7] as usize) << 24);

                (0..4).for_each(|py| {
                    (0..4).for_each(|px| {
                        let i = (py * 4 + px) as usize;
                        let mut color = colors[(indices >> (2 * i)) & 3];
                        if format != 1 {
                            color.alpha = alpha[i];
                        }
                        write_pixel(&mut pixels, width, height, bx * 4 + px, by * 4 + py, color);
                    })
                });
            })
        });

        pixels
    }
}

#[inline(always)]
fn write_pixel(pixels: &mut [Color32], width: u32, height: u32, x: u32, y: u32, color: Color32) {
    if x >= width || y >= height {
        return;
    }

    pixels[(y * width + x) as usize] = color;
}

#[inline(always)]
fn decode_color_block(color_block: &[u8], four_color: bool) -> [Color32; 4] {
    let c0 = u16::from_le_bytes([color_block[0], color_block[1]]);
    let c1 = u16::from_le_bytes([color_block[2], color_block[3]]);
    let mut colors = [
        convert_to_color(c0, 255),
        convert_to_color(c1, 255),
        Color32::default(),
        Color32::default(),
    ];

    if four_color || c0 > c1 {
        colors[2] = Color32 {
            red: ((2 * u16::from(colors[0].red) + u16::from(colors[1].red)) / 3) as u8,
            green: ((2 * u16::from(colors[0].green) + u16::from(colors[1].green)) / 3) as u8,
            blue: ((2 * u16::from(colors[0].blue) + u16::from(colors[1].blue)) / 3) as u8,
            alpha: 255,
        };
        colors[3] = Color32 {
            red: ((u16::from(colors[0].red) + 2 * u16::from(colors[1].red)) / 3) as u8,
            green: ((u16::from(colors[0].green) + 2 * u16::from(colors[1].green)) / 3) as u8,
            blue: ((u16::from(colors[0].blue) + 2 * u16::from(colors[1].blue)) / 3) as u8,
            alpha: 255,
        };
    } else {
        colors[2] = Color32 {
            red: ((u16::from(colors[0].red) + u16::from(colors[1].red)) / 2) as u8,
            green: ((u16::from(colors[0].green) + u16::from(colors[1].green)) / 2) as u8,
            blue: ((u16::from(colors[0].blue) + u16::from(colors[1].blue)) / 2) as u8,
            alpha: 255,
        };
    }

    colors
}

#[inline(always)]
const fn convert_to_color(color: u16, alpha: u8) -> Color32 {
    Color32 {
        red: (((color >> 11) & 0x1f) * 255 / 31) as u8,
        green: (((color >> 5) & 0x3f) * 255 / 63) as u8,
        blue: ((color & 0x1f) * 255 / 31) as u8,
        alpha,
    }
}

fn decode_alpha_dxt3(buffer: &[u8]) -> [u8; 16] {
    std::array::from_fn(|pixel| {
        let value = (buffer[pixel / 2] >> ((pixel & 1) * 4)) & 0x0f;
        (value << 4) | value
    })
}

fn decode_alpha_dxt5(buffer: &[u8]) -> [u8; 16] {
    let a0 = u16::from(buffer[0]);
    let a1 = u16::from(buffer[1]);
    let mut table = [a0 as u8, a1 as u8, 0, 0, 0, 0, 0, 0];

    if a0 > a1 {
        for i in 1..=6 {
            table[i + 1] = (((7 - i) as u16 * a0 + i as u16 * a1) / 7) as u8;
        }
    } else {
        for i in 1..=4 {
            table[i + 1] = (((5 - i) as u16 * a0 + i as u16 * a1) / 5) as u8;
        }
        table[6] = 0;
        table[7] = 255;
    }

    let bits = u64::from_le_bytes([
        buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7], 0, 0,
    ]);
    std::array::from_fn(|pixel| table[((bits >> (3 * pixel)) & 7) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_blp2_header() {
        const HEADER_SIZE: usize = 1172;
        let mut buffer = Vec::new();
        buffer.extend_from_slice(b"BLP2");
        buffer.extend_from_slice(&1_u32.to_le_bytes());
        buffer.extend_from_slice(&[2, 8, 7, 1]);
        buffer.extend_from_slice(&4_u32.to_le_bytes());
        buffer.extend_from_slice(&2_u32.to_le_bytes());
        for value in [HEADER_SIZE as u32, (HEADER_SIZE + 8) as u32]
            .into_iter()
            .chain(std::iter::repeat_n(0, 14))
        {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for value in [8_u32, 4].into_iter().chain(std::iter::repeat_n(0, 14)) {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        buffer.resize(HEADER_SIZE + 12, 0);
        buffer[148..152].copy_from_slice(&[10, 20, 30, 40]);

        let header = BLP2Header::load(&buffer).expect("valid BLP2 header");
        assert!(!header.is_jpeg);
        assert!(matches!(header.encoding, BLPEncoding::Dxt));
        assert!(matches!(header.alpha_depth, BLPAlphaDepth::Depth8));
        assert!(matches!(header.alpha_encoding, BLPAlphaEncoding::Dxt5));
        assert_eq!((header.width, header.height, header.mip_levels), (4, 2, 2));
        assert_eq!(header.offsets[1], HEADER_SIZE + 8);
        assert_eq!(header.lengths[1], 4);
        assert_eq!(
            header.palette[0],
            Color32 {
                red: 30,
                green: 20,
                blue: 10,
                alpha: 40
            }
        );

        let alphas = |colors: Vec<Color32>| {
            colors
                .into_iter()
                .map(|color| color.alpha)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            alphas(header.convert_paletted_no_alpha(2, 2, &[0; 4])),
            [255; 4]
        );
        assert_eq!(
            alphas(header.convert_paletted_alpha1(2, 2, &[0, 0, 0, 0, 0b1010])),
            [0, 255, 0, 255]
        );
        assert_eq!(
            alphas(header.convert_paletted_alpha4(2, 2, &[0, 0, 0, 0, 0xf0, 0x84])),
            [0, 255, 68, 136]
        );
        assert_eq!(
            alphas(header.convert_paletted_alpha8(2, 2, &[0, 0, 0, 0, 1, 2, 3, 4])),
            [1, 2, 3, 4]
        );
        assert_eq!(
            header.convert_raw_bgra(2, 1, &[30, 20, 10, 40, 70, 60, 50, 80]),
            [
                Color32 {
                    red: 10,
                    green: 20,
                    blue: 30,
                    alpha: 40
                },
                Color32 {
                    red: 50,
                    green: 60,
                    blue: 70,
                    alpha: 80
                }
            ]
        );
    }

    #[test]
    fn rejects_invalid_blp2_header() {
        assert!(matches!(BLP2Header::load(&[]), Err(Error::TruncatedHeader)));
        assert!(matches!(
            BLP2Header::load(&[0; 1171]),
            Err(Error::TruncatedHeader)
        ));
        assert!(matches!(
            BLP2Header::load(&[0; 1172]),
            Err(Error::InvalidMagic)
        ));
    }

    #[test]
    fn converts_rgb565_to_color32() {
        assert_eq!(
            convert_to_color(0xf800, 10),
            Color32 {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 10
            }
        );
        assert_eq!(
            convert_to_color(0x07e0, 20),
            Color32 {
                red: 0,
                green: 255,
                blue: 0,
                alpha: 20
            }
        );
        assert_eq!(
            convert_to_color(0x001f, 30),
            Color32 {
                red: 0,
                green: 0,
                blue: 255,
                alpha: 30
            }
        );
    }

    #[test]
    fn decodes_dxt_color_block() {
        let four = decode_color_block(&[0x00, 0xf8, 0x1f, 0x00], true);
        assert_eq!(
            four,
            [
                Color32 {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255
                },
                Color32 {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha: 255
                },
                Color32 {
                    red: 170,
                    green: 0,
                    blue: 85,
                    alpha: 255
                },
                Color32 {
                    red: 85,
                    green: 0,
                    blue: 170,
                    alpha: 255
                },
            ]
        );

        let three = decode_color_block(&[0x00, 0x00, 0xff, 0xff], false);
        assert_eq!(
            three[2],
            Color32 {
                red: 127,
                green: 127,
                blue: 127,
                alpha: 255
            }
        );
        assert_eq!(three[3], Color32::default());
    }

    #[test]
    fn decodes_dxt3_alpha() {
        assert_eq!(
            decode_alpha_dxt3(&[0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe]),
            [
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff,
            ]
        );
    }

    #[test]
    fn decodes_dxt5_alpha() {
        let bits = (0..16).fold(0_u64, |bits, pixel| {
            bits | ((pixel % 8) as u64) << (3 * pixel)
        });
        let indices = bits.to_le_bytes();

        let block = [
            255, 0, indices[0], indices[1], indices[2], indices[3], indices[4], indices[5],
        ];
        assert_eq!(
            decode_alpha_dxt5(&block),
            [
                255, 0, 218, 182, 145, 109, 72, 36, 255, 0, 218, 182, 145, 109, 72, 36
            ]
        );

        let block = [
            0, 255, indices[0], indices[1], indices[2], indices[3], indices[4], indices[5],
        ];
        assert_eq!(
            decode_alpha_dxt5(&block),
            [
                0, 255, 51, 102, 153, 204, 0, 255, 0, 255, 51, 102, 153, 204, 0, 255
            ]
        );
    }
}
