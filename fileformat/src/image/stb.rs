use shared::color::Color32;

use super::Error;

pub(super) struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color32>,
}

pub(super) fn load_from_memory(data: &[u8]) -> Result<Image, Error> {
    let header = data.get(..18).ok_or(Error::InvalidImageData)?;
    let image_type = header[2];
    let rle = matches!(image_type, 10 | 11);
    let grayscale = matches!(image_type, 3 | 11);
    if !matches!(image_type, 2 | 3 | 10 | 11) || header[1] != 0 {
        return Err(Error::UnsupportedTextureFormat);
    }

    let width = u16::from_le_bytes([header[12], header[13]]) as u32;
    let height = u16::from_le_bytes([header[14], header[15]]) as u32;
    let depth = header[16];
    if width == 0
        || height == 0
        || (!grayscale && !matches!(depth, 16 | 24 | 32))
        || (grayscale && !matches!(depth, 8 | 16))
    {
        return Err(Error::InvalidDimensions);
    }

    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or(Error::InvalidDimensions)?;
    let bytes_per_pixel = (depth / 8) as usize;
    let mut cursor = 18_usize
        .checked_add(header[0] as usize)
        .ok_or(Error::InvalidImageData)?;
    let mut source_pixels = Vec::with_capacity(pixel_count);

    while source_pixels.len() < pixel_count {
        let (count, repeated) = if rle {
            let packet = *data.get(cursor).ok_or(Error::InvalidImageData)?;
            cursor += 1;
            ((packet & 0x7f) as usize + 1, packet & 0x80 != 0)
        } else {
            (1, false)
        };
        if count > pixel_count - source_pixels.len() {
            return Err(Error::InvalidImageData);
        }

        if repeated {
            let pixel = read_pixel(data, &mut cursor, bytes_per_pixel, grayscale, depth)?;
            source_pixels.extend(std::iter::repeat_n(pixel, count));
        } else {
            for _ in 0..count {
                source_pixels.push(read_pixel(
                    data,
                    &mut cursor,
                    bytes_per_pixel,
                    grayscale,
                    depth,
                )?);
            }
        }
    }

    let top_origin = header[17] & 0x20 != 0;
    let right_origin = header[17] & 0x10 != 0;
    let mut pixels = vec![Color32::default(); pixel_count];
    for (index, pixel) in source_pixels.into_iter().enumerate() {
        let source_x = index % width as usize;
        let source_y = index / width as usize;
        let x = if right_origin {
            width as usize - 1 - source_x
        } else {
            source_x
        };
        let y = if top_origin {
            source_y
        } else {
            height as usize - 1 - source_y
        };
        pixels[y * width as usize + x] = pixel;
    }

    Ok(Image {
        width,
        height,
        pixels,
    })
}

fn read_pixel(
    data: &[u8],
    cursor: &mut usize,
    size: usize,
    grayscale: bool,
    depth: u8,
) -> Result<Color32, Error> {
    let end = cursor.checked_add(size).ok_or(Error::InvalidImageData)?;
    let pixel = data.get(*cursor..end).ok_or(Error::InvalidImageData)?;
    *cursor = end;

    if grayscale {
        return Ok(Color32 {
            red: pixel[0],
            green: pixel[0],
            blue: pixel[0],
            alpha: pixel.get(1).copied().unwrap_or(255),
        });
    }
    if depth == 16 {
        let value = u16::from_le_bytes([pixel[0], pixel[1]]);
        let expand = |channel: u32| (((value >> channel) & 0x1f_u16) * 255 / 31) as u8;
        return Ok(Color32 {
            red: expand(10),
            green: expand(5),
            blue: expand(0),
            alpha: if value & 0x8000 != 0 { 255 } else { 0 },
        });
    }
    Ok(Color32 {
        red: pixel[2],
        green: pixel[1],
        blue: pixel[0],
        alpha: pixel.get(3).copied().unwrap_or(255),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_bgr_to_rgba_and_normalizes_origin() {
        let mut data = vec![0; 18];
        data[2] = 2;
        data[12..14].copy_from_slice(&1_u16.to_le_bytes());
        data[14..16].copy_from_slice(&2_u16.to_le_bytes());
        data[16] = 24;
        data.extend_from_slice(&[30, 20, 10, 60, 50, 40]);

        let image = load_from_memory(&data).unwrap();
        assert_eq!(image.width, 1);
        assert_eq!(image.height, 2);
        assert_eq!(
            image.pixels[0],
            Color32 {
                red: 40,
                green: 50,
                blue: 60,
                alpha: 255
            }
        );
        assert_eq!(
            image.pixels[1],
            Color32 {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255
            }
        );
    }

    #[test]
    fn decodes_rle_rgba_without_swapping_red_and_blue() {
        let mut data = vec![0; 18];
        data[2] = 10;
        data[12..14].copy_from_slice(&2_u16.to_le_bytes());
        data[14..16].copy_from_slice(&1_u16.to_le_bytes());
        data[16] = 32;
        data[17] = 0x20;
        data.extend_from_slice(&[0x81, 30, 20, 10, 40]);

        let image = load_from_memory(&data).unwrap();
        assert_eq!(
            image.pixels,
            vec![
                Color32 {
                    red: 10,
                    green: 20,
                    blue: 30,
                    alpha: 40
                };
                2
            ]
        );
    }
}
