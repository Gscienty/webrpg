use fileformat::image::{Image, RawImage};
use shared::color::Color32;
use web_sys::WebGl2RenderingContext;

use crate::{Error, texture};

const SHADOW_MAP_PATH: &str = "war3map.shd";

pub struct Shadow {
    pub texture: texture::Texture,
}

impl Shadow {
    pub fn read(
        gl: &WebGl2RenderingContext,
        w3m: &mut dyn webrpg_mpq::Archive,
        world: &fileformat::w3e::Map,
    ) -> Result<Self, Error> {
        let width = (world.width - 1) * 4;
        let height = (world.height - 1) * 4;
        let shadows = w3m.read_file(SHADOW_MAP_PATH).map_err(Error::Mpq)?;
        let pixels = decode_pixels(&shadows, width, height)?;
        let texture = texture::Texture::new(
            gl,
            Image::Raw(vec![RawImage {
                pixels,
                width,
                height,
            }]),
        )?;

        Ok(Self { texture })
    }
}

fn decode_pixels(shadows: &[u8], width: u32, height: u32) -> Result<Vec<Color32>, Error> {
    let length = width
        .checked_mul(height)
        .and_then(|length| usize::try_from(length).ok())
        .ok_or(Error::InvalidShadowMapSize)?;
    if shadows.len() < length {
        return Err(Error::InvalidShadowMapSize);
    }

    Ok(shadows[..length]
        .iter()
        .map(|shadow| Color32 {
            red: 0,
            green: 0,
            blue: 0,
            alpha: shadow / 2,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_shadow_intensity_as_half_alpha() {
        assert_eq!(
            decode_pixels(&[0, 64, 128, 255], 2, 2).unwrap(),
            [
                Color32::default(),
                Color32 {
                    alpha: 32,
                    ..Default::default()
                },
                Color32 {
                    alpha: 64,
                    ..Default::default()
                },
                Color32 {
                    alpha: 127,
                    ..Default::default()
                },
            ]
        );
    }

    #[test]
    fn rejects_short_shadow_data() {
        assert_eq!(
            decode_pixels(&[0; 3], 2, 2),
            Err(Error::InvalidShadowMapSize)
        );
    }
}
