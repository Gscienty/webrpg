#[allow(dead_code)]
mod blp;
#[allow(dead_code)]
mod blp1;
#[allow(dead_code)]
mod blp2;
mod error;

pub use error::Error;
use shared::color::Color32;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

pub struct Texture {
    gl: WebGl2RenderingContext,
    texture: WebGlTexture,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn bind(&self) {
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    fn allocate(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Result<Self, Error> {
        let texture = gl.create_texture().ok_or(Error::CreateTextureFailed)?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        Ok(Self {
            gl: gl.clone(),
            texture,
            width,
            height,
        })
    }

    fn load_texture_mip_level(
        &self,
        gl: &WebGl2RenderingContext,
        pixels: &[Color32],
        width: u32,
        height: u32,
        level: usize,
    ) -> Result<(), Error> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(pixels.as_ptr().cast::<u8>(), size_of_val(pixels))
        };

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            level as i32,
            WebGl2RenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(bytes),
        )
        .map_err(|_| Error::UploadTextureFailed)?;

        if level > 0 {
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
                level as i32,
            );
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR as i32,
            );
        } else {
            for parameter in [
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
            ] {
                gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    parameter,
                    WebGl2RenderingContext::REPEAT as i32,
                );
            }
            for parameter in [
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            ] {
                gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    parameter,
                    WebGl2RenderingContext::LINEAR as i32,
                );
            }
        }

        Ok(())
    }

    pub fn load(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        match blp_version(data)? {
            1 => Self::load_blp1(gl, data),
            2 => Self::load_blp2(gl, data),
            _ => unreachable!(),
        }
    }

    fn load_blp1(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        let info = blp1::InternalBLPInfos::load(data)?;

        let texture = Self::allocate(gl, info.width(0), info.height(0))?;

        for level in 0..info.mip_level() {
            let width = info.width(level);
            let height = info.height(level);

            let pixels = info.convert(level, data)?;
            if !pixels.is_empty() {
                texture.load_texture_mip_level(gl, &pixels, width, height, level)?;
            }
        }

        Ok(texture)
    }

    fn load_blp2(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        let info = blp2::BLP2Header::load(data)?;

        let texture = Self::allocate(gl, info.width(0), info.height(0))?;

        for level in 0..info.mip_level() {
            let width = info.width(level);
            let height = info.height(level);

            let pixels = info.convert(level, data);
            if !pixels.is_empty() {
                texture.load_texture_mip_level(gl, &pixels, width, height, level)?;
            }
        }

        Ok(texture)
    }
}

fn blp_version(data: &[u8]) -> Result<u8, Error> {
    let magic = data.get(..4).ok_or(Error::TruncatedHeader)?;
    match magic {
        b"BLP1" => Ok(1),
        b"BLP2" => Ok(2),
        _ => Err(Error::InvalidMagic),
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.gl.delete_texture(Some(&self.texture));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_blp_version() {
        assert_eq!(blp_version(b"BLP1"), Ok(1));
        assert_eq!(blp_version(b"BLP2"), Ok(2));
        assert_eq!(blp_version(b"BLP"), Err(Error::TruncatedHeader));
        assert_eq!(blp_version(b"BLP3"), Err(Error::InvalidMagic));
    }
}
