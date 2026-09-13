#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TruncatedHeader,
    InvalidMagic,
    InvalidHeaderSize,
    InvalidPixelFormatSize,
    UnsupportedTextureFormat,
    InvalidType,
    InvalidEncoding,
    InvalidAlphaDepth,
    InvalidAlphaEncoding,
    InvalidDimensions,
    NoImageData,
    MipOutOfBounds,
    MissingJpegHeaderSize,
    TruncatedJpegHeader,
    InvalidJpegData,
    InvalidPcxData,
    InvalidImageData,
    TruncatedPalette,
    CreateTextureFailed,
    UploadTextureFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TruncatedHeader => "truncated texture header",
            Self::InvalidMagic => "invalid texture magic",
            Self::InvalidHeaderSize => "invalid texture header size",
            Self::InvalidPixelFormatSize => "invalid texture pixel format size",
            Self::UnsupportedTextureFormat => "unsupported texture format",
            Self::InvalidType => "invalid texture type",
            Self::InvalidEncoding => "invalid texture encoding",
            Self::InvalidAlphaDepth => "invalid texture alpha depth",
            Self::InvalidAlphaEncoding => "invalid texture alpha encoding",
            Self::InvalidDimensions => "invalid texture dimensions",
            Self::NoImageData => "texture has no image data",
            Self::MipOutOfBounds => "texture mip data is out of bounds",
            Self::MissingJpegHeaderSize => "missing JPEG header size",
            Self::TruncatedJpegHeader => "truncated JPEG header",
            Self::InvalidJpegData => "invalid JPEG data",
            Self::InvalidPcxData => "invalid PCX data",
            Self::InvalidImageData => "invalid image data",
            Self::TruncatedPalette => "truncated texture palette",
            Self::CreateTextureFailed => "failed to create WebGL texture",
            Self::UploadTextureFailed => "failed to upload WebGL texture",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Error {}
