use fileformat::image::Image;
use web_sys::WebGl2RenderingContext;

use crate::{Error, texture};

const CANDIDATES: [&str; 2] = ["war3mapMap.blp", "war3mapMap.tga"];

pub struct Mini {
    pub texture: texture::Texture,
}

impl Mini {
    pub fn read(
        gl: &WebGl2RenderingContext,
        w3m: &mut dyn webrpg_mpq::Archive,
    ) -> Result<Self, Error> {
        let (path, data) = first_available(&CANDIDATES, |path| w3m.read_file(path))?;
        let texture = texture::Texture::new(gl, Image::load(path, &data)?)?;

        Ok(Self { texture })
    }
}

fn first_available<'a>(
    candidates: &'a [&str],
    mut read: impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<(&'a str, Vec<u8>), Error> {
    for &path in candidates {
        if let Ok(data) = read(path) {
            return Ok((path, data));
        }
    }
    Err(Error::MissingMiniMap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_from_blp_to_tga() {
        let mut attempted = Vec::new();
        let (path, data) = first_available(&CANDIDATES, |path| {
            attempted.push(path.to_owned());
            match path {
                "war3mapMap.tga" => Ok(vec![1, 2, 3]),
                _ => Err("missing".into()),
            }
        })
        .unwrap();

        assert_eq!(attempted, CANDIDATES);
        assert_eq!(path, "war3mapMap.tga");
        assert_eq!(data, [1, 2, 3]);
    }

    #[test]
    fn reports_missing_minimap() {
        assert_eq!(
            first_available(&CANDIDATES, |_| Err("missing".into())),
            Err(Error::MissingMiniMap)
        );
    }
}
