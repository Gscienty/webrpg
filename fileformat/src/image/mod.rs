mod blp;
mod blp1;
mod blp2;
mod dds;
mod pcx;
mod stb;

mod error;

pub use error::Error;

pub struct RawImage {
    pub width: u32,
    pub height: u32,

    pub pixels: Vec<shared::color::Color32>,
}

pub enum CompressImageFormat {
    DXT1,
    DXT3,
    DXT5,

    RGB,
    RGBA,
}

pub struct CompressImageBlock<'s> {
    pub swap_red_blue: bool,

    pub width: u32,
    pub height: u32,

    pub raw: &'s [u8],
}

pub struct CompressImage<'s> {
    pub format: CompressImageFormat,

    pub width: u32,
    pub height: u32,

    pub blocks: Vec<CompressImageBlock<'s>>,
}

pub enum Image<'s> {
    Raw(Vec<RawImage>),
    Compress(CompressImage<'s>),
}

impl<'s> Image<'s> {
    pub fn load(filename: &str, raw: &'s [u8]) -> Result<Self, Error> {
        let magic = raw.get(..4).ok_or(Error::TruncatedHeader)?;

        match magic {
            b"BLP1" => RawImage::load_blp1(raw).map(Self::Raw),
            b"BLP2" => RawImage::load_blp2(raw).map(Self::Raw),
            b"DDS\x20" => CompressImage::load_dds(raw).map(Self::Compress),
            _ if filename.to_lowercase().ends_with(".pcx") || pcx::PCXInfo::is_pcx(raw) => {
                RawImage::load_pcx(raw).map(Self::Raw)
            }
            _ if filename.to_lowercase().ends_with(".tga") => {
                RawImage::load_stb(raw).map(Self::Raw)
            }
            _ => Err(Error::InvalidMagic),
        }
    }
}

impl RawImage {
    fn load_blp1(raw: &[u8]) -> Result<Vec<Self>, Error> {
        let info = blp1::InternalBLPInfos::load(raw)?;

        (0..info.mip_level())
            .map(|level| {
                info.convert(level, raw).map(|pixels| RawImage {
                    width: info.width(level),
                    height: info.height(level),
                    pixels,
                })
            })
            .collect()
    }

    fn load_blp2(raw: &[u8]) -> Result<Vec<Self>, Error> {
        let info = blp2::BLP2Header::load(raw)?;

        (0..info.mip_level())
            .map(|level| {
                info.convert(level, raw).map(|pixels| Self {
                    width: info.width(level),
                    height: info.height(level),
                    pixels,
                })
            })
            .collect()
    }

    fn load_pcx(raw: &[u8]) -> Result<Vec<Self>, Error> {
        let info = pcx::PCXInfo::load(raw)?;
        let pixels = info.convert(raw)?;

        Ok(vec![Self {
            width: info.width(),
            height: info.height(),
            pixels,
        }])
    }

    fn load_stb(raw: &[u8]) -> Result<Vec<Self>, Error> {
        let image = stb::load_from_memory(raw)?;

        Ok(vec![Self {
            width: image.width,
            height: image.height,
            pixels: image.pixels,
        }])
    }
}

impl<'s> CompressImage<'s> {
    fn load_dds(raw: &'s [u8]) -> Result<Self, Error> {
        let info = dds::DDSHeader::load(raw)?;

        if info.is_four_cc() && info.get_four_cc() != 0 {
            Self::load_dds_four_cc(&info, raw)
        } else if info.is_rgb() || (info.get_flags() & 0x20000) != 0 {
            Self::load_dds_rgb(&info, raw)
        } else {
            Err(Error::UnsupportedTextureFormat)
        }
    }

    fn load_dds_four_cc(info: &dds::DDSHeader, raw: &'s [u8]) -> Result<Self, Error> {
        const FOURCC_DXT1: u32 = u32::from_le_bytes(*b"DXT1");
        const FOURCC_DXT3: u32 = u32::from_le_bytes(*b"DXT3");
        const FOURCC_DXT5: u32 = u32::from_le_bytes(*b"DXT5");

        let (format, block_size) = match info.get_four_cc() {
            FOURCC_DXT1 => (CompressImageFormat::DXT1, 8),
            FOURCC_DXT3 => (CompressImageFormat::DXT3, 16),
            FOURCC_DXT5 => (CompressImageFormat::DXT5, 16),
            _ => return Err(Error::UnsupportedTextureFormat),
        };

        let mut offset = info.pixel_offset();
        let mut width = info.width();
        let mut height = info.height();

        let levels = info.mipmap_count();

        let mut blocks = Vec::with_capacity(levels);
        for _ in 0..info.mipmap_count() {
            let block_width = (width as usize).div_ceil(4);
            let block_height = (height as usize).div_ceil(4);
            let block_size = block_width * block_height * block_size;

            blocks.push(CompressImageBlock {
                swap_red_blue: false,

                width,
                height,

                raw: &raw[offset..offset + block_size],
            });

            offset += block_size;
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        Ok(Self {
            format,

            blocks,

            width: info.width(),
            height: info.height(),
        })
    }

    fn load_dds_rgb(info: &dds::DDSHeader, raw: &'s [u8]) -> Result<Self, Error> {
        let layout = info.rgb_layout()?;
        let format = if layout.bytes_per_pixel == 4 {
            CompressImageFormat::RGBA
        } else {
            CompressImageFormat::RGB
        };
        let levels = info.mipmap_count().max(1);
        let mut offset = info.pixel_offset();
        let mut width = info.width();
        let mut height = info.height();

        let mut blocks = Vec::with_capacity(levels);
        for _ in 0..levels {
            let block_size = ((width * height) as usize) * layout.bytes_per_pixel;

            blocks.push(CompressImageBlock {
                swap_red_blue: layout.swap_red_blue,

                width,
                height,

                raw: &raw[offset..offset + block_size],
            });

            offset += block_size;

            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        Ok(Self {
            format,
            width: info.width(),
            height: info.height(),
            blocks,
        })
    }
}
