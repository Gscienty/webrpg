use shared::color::Color32;

use super::Error;

pub(super) struct PCXInfo {
    width: u32,
    height: u32,
    bytes_per_line: u32,
    palette_pos: usize,
    palette: [Color32; 256],
}

impl PCXInfo {
    pub fn load(data: &[u8]) -> Result<Self, Error> {
        if !Self::is_pcx(data) {
            return Err(Error::InvalidMagic);
        }
        let width = u32::from(read_u16(data, 8) - read_u16(data, 4)) + 1;
        let height = u32::from(read_u16(data, 10) - read_u16(data, 6)) + 1;
        let bytes_per_line = u32::from(read_u16(data, 66));
        let palette_offset = data
            .len()
            .checked_sub(769)
            .filter(|&offset| data.len() >= 897 && data[offset] == 0x0c);
        let palette = std::array::from_fn(|index| {
            if let Some(offset) = palette_offset {
                let color = offset + 1 + index * 3;
                Color32 {
                    red: data[color],
                    green: data[color + 1],
                    blue: data[color + 2],
                    alpha: 255,
                }
            } else {
                let value = index as u8;
                Color32 {
                    red: value,
                    green: value,
                    blue: value,
                    alpha: 255,
                }
            }
        });

        Ok(Self {
            width,
            height,
            bytes_per_line,
            palette_pos: palette_offset.unwrap_or(data.len()),
            palette,
        })
    }

    pub fn is_pcx(data: &[u8]) -> bool {
        if data.len() < 128 {
            return false;
        }

        let x_min = read_u16(data, 4);
        let y_min = read_u16(data, 6);
        let x_max = read_u16(data, 8);
        let y_max = read_u16(data, 10);
        let Some(width) = x_max.checked_sub(x_min).map(|value| value as u32 + 1) else {
            return false;
        };
        let Some(height) = y_max.checked_sub(y_min).map(|value| value as u32 + 1) else {
            return false;
        };
        let bytes_per_line = read_u16(data, 66) as u32;

        data[0] == 0x0a
            && data[2] == 1
            && data[3] == 8
            && data[65] == 1
            && width > 0
            && height > 0
            && bytes_per_line >= width
    }

    pub fn convert(&self, data: &[u8]) -> Result<Vec<Color32>, Error> {
        let row_size = (self.bytes_per_line * self.height) as usize;
        let source = &data[128..self.palette_pos];

        let mut rows = Vec::with_capacity(row_size);
        let mut cursor = 0;

        while rows.len() < row_size && cursor < source.len() {
            let byte = source[cursor];
            cursor += 1;
            let (run, value) = if byte & 0xc0 == 0xc0 {
                let value = *source.get(cursor).ok_or(Error::InvalidPcxData)?;
                cursor += 1;
                ((byte & 0x3f) as usize, value)
            } else {
                (1, byte)
            };
            rows.extend(std::iter::repeat_n(value, run.min(row_size - rows.len())));
        }
        if rows.len() != row_size {
            return Err(Error::InvalidPcxData);
        }

        let mut pixels = Vec::with_capacity((self.width * self.height) as usize);
        for row in rows.chunks_exact(self.bytes_per_line as usize) {
            for &index in &row[..self.width as usize] {
                let mut color = self.palette[index as usize];
                color.alpha = if index == 255 { 0 } else { 255 };
                pixels.push(color);
            }
        }
        Ok(pixels)
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_header() -> [u8; 128] {
        let mut data = [0_u8; 128];
        data[0] = 0x0a;
        data[2] = 1;
        data[3] = 8;
        data[65] = 1;
        data[4..6].copy_from_slice(&10_u16.to_le_bytes());
        data[6..8].copy_from_slice(&20_u16.to_le_bytes());
        data[8..10].copy_from_slice(&109_u16.to_le_bytes());
        data[10..12].copy_from_slice(&69_u16.to_le_bytes());
        data[66..68].copy_from_slice(&100_u16.to_le_bytes());
        data
    }

    #[test]
    fn detects_valid_pcx_header() {
        assert!(PCXInfo::is_pcx(&valid_header()));
    }

    #[test]
    fn loads_dimensions_and_palette() {
        let info = PCXInfo::load(&valid_header()).unwrap();
        assert_eq!(
            (info.width, info.height, info.bytes_per_line),
            (100, 50, 100)
        );
        assert_eq!(
            info.palette[42],
            Color32 {
                red: 42,
                green: 42,
                blue: 42,
                alpha: 255,
            }
        );

        let mut data = valid_header().to_vec();
        data.extend_from_slice(&[0; 769]);
        data[128] = 0x0c;
        data[129 + 42 * 3..129 + 42 * 3 + 3].copy_from_slice(&[10, 20, 30]);
        let info = PCXInfo::load(&data).unwrap();
        assert_eq!(
            info.palette[42],
            Color32 {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255,
            }
        );
    }

    #[test]
    fn decodes_rle_pixels_and_row_padding() {
        let mut header = valid_header();
        header[4..8].fill(0);
        header[8..10].copy_from_slice(&1_u16.to_le_bytes());
        header[10..12].fill(0);
        header[66..68].copy_from_slice(&3_u16.to_le_bytes());

        let mut data = header.to_vec();
        data.extend_from_slice(&[42, 0xc1, 255, 0]);
        let palette_pos = data.len();
        data.push(0x0c);
        data.extend_from_slice(&[0; 768]);
        data[palette_pos + 1 + 42 * 3..palette_pos + 1 + 42 * 3 + 3].copy_from_slice(&[10, 20, 30]);

        let info = PCXInfo::load(&data).unwrap();
        assert_eq!(
            info.convert(&data).unwrap(),
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
                    alpha: 0,
                },
            ]
        );

        let mut truncated = header.to_vec();
        truncated.push(0xc2);
        let info = PCXInfo::load(&truncated).unwrap();
        assert_eq!(info.convert(&truncated), Err(Error::InvalidPcxData));
    }

    #[test]
    fn rejects_invalid_pcx_headers() {
        assert!(!PCXInfo::is_pcx(&[]));

        let mut data = valid_header();
        data[8..10].copy_from_slice(&9_u16.to_le_bytes());
        assert!(!PCXInfo::is_pcx(&data));

        let mut data = valid_header();
        data[66..68].copy_from_slice(&99_u16.to_le_bytes());
        assert!(!PCXInfo::is_pcx(&data));
    }
}
