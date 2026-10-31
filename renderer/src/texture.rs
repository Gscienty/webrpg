use fileformat::image::{CompressImage, CompressImageFormat, Image, RawImage};
use shared::color;
use web_sys::{WebGl2RenderingContext, WebGlTexture, WebglCompressedTextureS3tc};

use crate::error::Error;

#[derive(Clone)]
pub struct Texture {
    gltex: WebGlTexture,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub(crate) fn is_same(&self, other: &Self) -> bool {
        self.gltex == other.gltex
    }

    pub fn init(gl: &WebGl2RenderingContext) -> Result<(), Error> {
        gl.get_extension("WEBGL_compressed_texture_s3tc")
            .map_err(|_| Error::UnsupportedTextureFormat)?
            .ok_or(Error::UnsupportedTextureFormat)?;
        Ok(())
    }

    pub fn new(gl: &WebGl2RenderingContext, image: Image<'_>) -> Result<Self, Error> {
        match image {
            Image::Raw(images) => Self::from_raw(gl, &images),
            Image::Compress(image) => Self::from_compressed(gl, &image),
        }
    }

    pub(crate) fn color(gl: &WebGl2RenderingContext, pixel: color::Color32) -> Result<Self, Error> {
        Self::from_raw(
            gl,
            &[RawImage {
                pixels: vec![pixel],
                width: 1,
                height: 1,
            }],
        )
    }

    pub(crate) fn particle(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        const SIZE: usize = 8;
        const DOT_TEXTURE: [[u8; SIZE]; SIZE] = [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 1, 1, 0, 0],
            [0, 1, 2, 2, 2, 2, 1, 0],
            [0, 1, 2, 2, 2, 2, 1, 0],
            [0, 1, 2, 2, 2, 2, 1, 0],
            [0, 1, 2, 2, 2, 2, 1, 0],
            [0, 0, 1, 1, 1, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];
        let pixels: [color::Color32; SIZE * SIZE] = std::array::from_fn(|i| color::Color32 {
            red: 255,
            green: 255,
            blue: 255,
            alpha: DOT_TEXTURE[i / SIZE][i % SIZE] * 127,
        });

        Self::from_raw(
            gl,
            &[RawImage {
                pixels: pixels.into(),
                width: SIZE as u32,
                height: SIZE as u32,
            }],
        )
    }

    pub(crate) fn placeholder(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        const SIZE: usize = 16;
        let pixels: [color::Color32; SIZE * SIZE] = std::array::from_fn(|i| {
            let (x, y) = (i % SIZE, i / SIZE);
            if (x ^ y) & 1 != 0 {
                color::Color32 {
                    red: 255,
                    green: 0,
                    blue: 255,
                    alpha: 255,
                }
            } else {
                color::Color32 {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                }
            }
        });

        Self::from_raw(
            gl,
            &[RawImage {
                pixels: pixels.into(),
                width: SIZE as u32,
                height: SIZE as u32,
            }],
        )
    }

    pub(crate) fn shadow(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        const SIZE: usize = 64;
        let pixels: [color::Color32; SIZE * SIZE] = std::array::from_fn(|i| {
            let x = i % SIZE;
            let y = i / SIZE;
            let fx = (x as f32 + 0.5) / SIZE as f32 * 2. - 1.;
            let fy = (y as f32 + 0.5) / SIZE as f32 * 2. - 1.;
            let distance = (fx * fx + fy * fy).sqrt();
            let alpha = 1. - smooth_step(0.25, 1., distance);

            color::Color32 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: (alpha * 255.) as u8,
            }
        });

        Self::from_raw(
            gl,
            &[RawImage {
                pixels: pixels.into(),
                width: SIZE as u32,
                height: SIZE as u32,
            }],
        )
    }

    pub(crate) fn loading(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        const SIZE: usize = 128;
        let pixels: [color::Color32; SIZE * SIZE] = std::array::from_fn(|i| {
            let x = i % SIZE;
            let y = i / SIZE;
            let fx = 1. - (x as f32 + 0.5) / SIZE as f32 * 2.;
            let fy = (y as f32 + 0.5) / SIZE as f32 * 2. - 1.;
            let distance = (fx * fx + fy * fy).sqrt();
            let angle = (fy.atan2(fx) / std::f32::consts::TAU).rem_euclid(1.);
            let outer = 0.92;
            let inner = outer * 0.68;
            let edge = 0.035;
            let ring = smooth_step(inner - edge, inner, distance)
                * (1. - smooth_step(outer, outer + edge, distance));

            color::Color32 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: (255. * ring * angle) as u8,
            }
        });

        Self::from_raw(
            gl,
            &[RawImage {
                pixels: pixels.into(),
                width: SIZE as u32,
                height: SIZE as u32,
            }],
        )
    }

    fn from_raw(gl: &WebGl2RenderingContext, images: &[RawImage]) -> Result<Self, Error> {
        let first = images.first().ok_or(Error::NoImageData)?;
        let texture = Self::allocate(gl, first.width, first.height)?;

        for (level, image) in images.iter().enumerate() {
            texture.upload_raw(gl, &image.pixels, image.width, image.height, level)?;
        }
        texture.set_mipmap_filter(gl, images.len());

        Ok(texture)
    }

    fn from_compressed(
        gl: &WebGl2RenderingContext,
        image: &CompressImage<'_>,
    ) -> Result<Self, Error> {
        if image.blocks.is_empty() {
            return Err(Error::NoImageData);
        }

        let texture = Self::allocate(gl, image.width, image.height)?;
        for (level, block) in image.blocks.iter().enumerate() {
            match image.format {
                CompressImageFormat::DXT1
                | CompressImageFormat::DXT3
                | CompressImageFormat::DXT5 => {
                    let format = match image.format {
                        CompressImageFormat::DXT1 => {
                            WebglCompressedTextureS3tc::COMPRESSED_RGB_S3TC_DXT1_EXT
                        }
                        CompressImageFormat::DXT3 => {
                            WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT3_EXT
                        }
                        CompressImageFormat::DXT5 => {
                            WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT5_EXT
                        }
                        _ => unreachable!(),
                    };
                    texture.upload_compressed(
                        gl,
                        block.raw,
                        block.width,
                        block.height,
                        level,
                        format,
                    )?;
                }
                CompressImageFormat::RGB | CompressImageFormat::RGBA => {
                    let mut swapped = Vec::new();
                    let pixels = if block.swap_red_blue {
                        swapped.extend_from_slice(block.raw);
                        let channels = if matches!(image.format, CompressImageFormat::RGBA) {
                            4
                        } else {
                            3
                        };
                        for pixel in swapped.chunks_exact_mut(channels) {
                            pixel.swap(0, 2);
                        }
                        swapped.as_slice()
                    } else {
                        block.raw
                    };
                    texture.upload_bytes(
                        gl,
                        pixels,
                        block.width,
                        block.height,
                        level,
                        if matches!(image.format, CompressImageFormat::RGBA) {
                            WebGl2RenderingContext::RGBA
                        } else {
                            WebGl2RenderingContext::RGB
                        },
                    )?;
                }
            }
        }
        texture.set_mipmap_filter(gl, image.blocks.len());

        Ok(texture)
    }

    fn allocate(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Result<Self, Error> {
        let gltex = gl.create_texture().ok_or(Error::CreateTextureFailed)?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gltex));
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
            gltex,
            width,
            height,
        })
    }

    fn upload_raw(
        &self,
        gl: &WebGl2RenderingContext,
        pixels: &[color::Color32],
        width: u32,
        height: u32,
        level: usize,
    ) -> Result<(), Error> {
        let bytes = unsafe {
            std::slice::from_raw_parts(pixels.as_ptr().cast::<u8>(), size_of_val(pixels))
        };
        self.upload_bytes(
            gl,
            bytes,
            width,
            height,
            level,
            WebGl2RenderingContext::RGBA,
        )
    }

    fn upload_bytes(
        &self,
        gl: &WebGl2RenderingContext,
        pixels: &[u8],
        width: u32,
        height: u32,
        level: usize,
        format: u32,
    ) -> Result<(), Error> {
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.gltex));
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
        .map_err(|_| Error::UploadTextureFailed)
    }

    fn upload_compressed(
        &self,
        gl: &WebGl2RenderingContext,
        pixels: &[u8],
        width: u32,
        height: u32,
        level: usize,
        format: u32,
    ) -> Result<(), Error> {
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.gltex));
        gl.compressed_tex_image_2d_with_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            level as i32,
            format,
            width as i32,
            height as i32,
            0,
            pixels,
        );
        if gl.get_error() == WebGl2RenderingContext::NO_ERROR {
            Ok(())
        } else {
            Err(Error::UploadTextureFailed)
        }
    }

    fn set_mipmap_filter(&self, gl: &WebGl2RenderingContext, levels: usize) {
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.gltex));
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            levels.saturating_sub(1) as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            if levels > 1 {
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR
            } else {
                WebGl2RenderingContext::LINEAR
            } as i32,
        );
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext, unit: u32) {
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + unit);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.gltex));
    }

    pub fn set_texture_wrap(&self, gl: &WebGl2RenderingContext, wrap_s: bool, wrap_t: bool) {
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.gltex));
        for (parameter, repeat) in [
            (WebGl2RenderingContext::TEXTURE_WRAP_S, wrap_s),
            (WebGl2RenderingContext::TEXTURE_WRAP_T, wrap_t),
        ] {
            gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                parameter,
                if repeat {
                    WebGl2RenderingContext::REPEAT
                } else {
                    WebGl2RenderingContext::CLAMP_TO_EDGE
                } as i32,
            );
        }
    }

    pub fn drop(&self, gl: &WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.gltex));
    }
}

fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
