#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TruncatedHeader,
    InvalidMagic,
    InvalidType,
    InvalidAlphaDepth,
    InvalidDimensions,
    NoImageData,
    MipOutOfBounds,
    MissingJpegHeaderSize,
    TruncatedJpegHeader,
    TruncatedPalette,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TruncatedHeader => "truncated texture header",
            Self::InvalidMagic => "invalid texture magic",
            Self::InvalidType => "invalid texture type",
            Self::InvalidAlphaDepth => "invalid texture alpha depth",
            Self::InvalidDimensions => "invalid texture dimensions",
            Self::NoImageData => "texture has no image data",
            Self::MipOutOfBounds => "texture mip data is out of bounds",
            Self::MissingJpegHeaderSize => "missing JPEG header size",
            Self::TruncatedJpegHeader => "truncated JPEG header",
            Self::TruncatedPalette => "truncated texture palette",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Error {}
