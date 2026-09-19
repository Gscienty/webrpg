mod blp;
mod blp1;
mod blp2;
mod dds;
mod error;
mod pcx;
mod stb;

pub use error::Error;
use shared::color::Color32;
use web_sys::{WebGl2RenderingContext, WebGlTexture, WebglCompressedTextureS3tc};

pub struct Texture {
    texture: WebGlTexture,
    width: u32,
    height: u32,
}

impl Texture {
    pub(crate) fn white(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        let texture = Self::allocate(gl, 1, 1)?;
        texture.load_texture_mip_level(
            gl,
            &[Color32 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            }],
            1,
            1,
            0,
        )?;
        Ok(texture)
    }

    pub fn init(gl: &WebGl2RenderingContext) -> Result<(), Error> {
        gl.get_extension("WEBGL_compressed_texture_s3tc")
            .map_err(|_| Error::UnsupportedTextureFormat)?
            .ok_or(Error::UnsupportedTextureFormat)?;
        Ok(())
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext, unit: u32) {
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + unit);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    fn allocate(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Result<Self, Error> {
        let texture = gl.create_texture().ok_or(Error::CreateTextureFailed)?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
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

        Ok(Self {
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
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
        }

        Ok(())
    }

    pub fn load(gl: &WebGl2RenderingContext, filename: &str, data: &[u8]) -> Result<Self, Error> {
        let magic = data.get(..4).ok_or(Error::TruncatedHeader)?;
        match magic {
            b"BLP1" => Self::load_blp1(gl, data),
            b"BLP2" => Self::load_blp2(gl, data),
            b"DDS\x20" => Self::load_dds(gl, data),
            _ if filename.to_lowercase().ends_with(".pcx") || pcx::PCXInfo::is_pcx(data) => {
                Self::load_pcx(gl, data)
            }
            _ if filename.to_lowercase().ends_with(".tga") => Self::load_stb(gl, data),
            _ => Err(Error::InvalidMagic),
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

    fn load_dds(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        let info = dds::DDSHeader::load(data)?;

        if info.is_four_cc() && info.get_four_cc() != 0 {
            Self::load_dds_four_cc(gl, &info, data)
        } else if info.is_rgb() || (info.get_flags() & 0x20000) != 0 {
            Self::load_dds_rgb(gl, &info, data)
        } else {
            Err(Error::UnsupportedTextureFormat)
        }
    }

    fn load_dds_four_cc(
        gl: &WebGl2RenderingContext,
        info: &dds::DDSHeader,
        data: &[u8],
    ) -> Result<Self, Error> {
        const FOURCC_DXT1: u32 = u32::from_le_bytes(*b"DXT1");
        const FOURCC_DXT3: u32 = u32::from_le_bytes(*b"DXT3");
        const FOURCC_DXT5: u32 = u32::from_le_bytes(*b"DXT5");

        let (format, block_size) = match info.get_four_cc() {
            FOURCC_DXT1 => (WebglCompressedTextureS3tc::COMPRESSED_RGB_S3TC_DXT1_EXT, 8),
            FOURCC_DXT3 => (
                WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT3_EXT,
                16,
            ),
            FOURCC_DXT5 => (
                WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT5_EXT,
                16,
            ),
            _ => return Err(Error::UnsupportedTextureFormat),
        };

        let texture = Self::allocate(gl, info.width(), info.height())?;

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            0,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR as i32,
        );

        let mut offset = info.pixel_offset();
        let mut width = info.width();
        let mut height = info.height();

        let mut levels = info.mipmap_count();
        for level in 0..levels {
            if width == 0 || height == 0 {
                levels -= 1;
                break;
            }
            let blocks_wide = (width as usize).div_ceil(4);
            let blocks_high = (height as usize).div_ceil(4);
            let size = blocks_wide
                .checked_mul(blocks_high)
                .and_then(|blocks| blocks.checked_mul(block_size))
                .ok_or(Error::InvalidDimensions)?;
            let end = offset.checked_add(size).ok_or(Error::MipOutOfBounds)?;
            let pixels = data.get(offset..end).ok_or(Error::MipOutOfBounds)?;

            gl.compressed_tex_image_2d_with_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                level as i32,
                format,
                width as i32,
                height as i32,
                0,
                pixels,
            );
            if gl.get_error() != WebGl2RenderingContext::NO_ERROR {
                return Err(Error::UploadTextureFailed);
            }

            offset = end;
            width /= 2;
            height /= 2;
        }

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            (levels - 1) as i32,
        );

        Ok(texture)
    }

    fn load_dds_rgb(
        gl: &WebGl2RenderingContext,
        info: &dds::DDSHeader,
        data: &[u8],
    ) -> Result<Self, Error> {
        let layout = info.rgb_layout()?;
        let format = if layout.bytes_per_pixel == 4 {
            WebGl2RenderingContext::RGBA
        } else {
            WebGl2RenderingContext::RGB
        };
        let texture = Self::allocate(gl, info.width(), info.height())?;
        let mut levels = info.mipmap_count().max(1);
        let mut offset = info.pixel_offset();
        let mut width = info.width();
        let mut height = info.height();

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            0,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            if levels > 1 {
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR as i32
            } else {
                WebGl2RenderingContext::LINEAR as i32
            },
        );

        for level in 0..levels {
            if width == 0 || height == 0 {
                levels -= 1;
                break;
            }

            let size = ((width * height) as usize) * layout.bytes_per_pixel;

            let mut swapped = Vec::new();
            let pixels = if layout.swap_red_blue {
                swapped.extend_from_slice(&data[offset..offset + size]);
                for pixel in swapped.chunks_exact_mut(layout.bytes_per_pixel) {
                    pixel.swap(0, 2);
                }
                swapped.as_slice()
            } else {
                &data[offset..offset + size]
            };

            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                level as i32,
                format as i32,
                width as i32,
                height as i32,
                0,
                format,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(pixels),
            )
            .map_err(|_| Error::UploadTextureFailed)?;

            offset += size;
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            (levels - 1) as i32,
        );

        Ok(texture)
    }

    pub fn load_pcx(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        let info = pcx::PCXInfo::load(data)?;
        let pixels = info.convert(data)?;
        let texture = Self::allocate(gl, info.width(), info.height())?;
        texture.load_texture_mip_level(gl, &pixels, info.width(), info.height(), 0)?;
        Ok(texture)
    }

    pub fn load_stb(gl: &WebGl2RenderingContext, data: &[u8]) -> Result<Self, Error> {
        let image = stb::load_from_memory(data)?;

        let texture = Self::allocate(gl, image.width, image.height)?;
        texture.load_texture_mip_level(gl, &image.pixels, image.width, image.height, 0)?;
        Ok(texture)
    }

    pub fn drop(&self, gl: &WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.texture));
    }
}
