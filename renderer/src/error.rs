#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    CreateShaderFailed,
    CreateBufferFailed,
    CreateVertexArrayFailed,
    CompileShaderFailed(String),
    CreateProgramFailed,
    LinkProgramFailed(String),
    InvalidUniformData(&'static str),
    UnsupportedTextureFormat,
    NoImageData,
    CreateTextureFailed,
    UploadTextureFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateShaderFailed => formatter.write_str("failed to create shader"),
            Self::CreateBufferFailed => formatter.write_str("failed to create_buffer"),
            Self::CreateVertexArrayFailed => formatter.write_str("failed to create_vertex_array"),
            Self::CompileShaderFailed(message) => {
                write!(formatter, "failed to compile shader: {message}")
            }
            Self::CreateProgramFailed => formatter.write_str("failed to create shader program"),
            Self::LinkProgramFailed(message) => {
                write!(formatter, "failed to link shader program: {message}")
            }
            Self::InvalidUniformData(name) => write!(formatter, "invalid uniform data: {name}"),
            Self::UnsupportedTextureFormat => formatter.write_str("unsupported texture format"),
            Self::NoImageData => formatter.write_str("texture has no image data"),
            Self::CreateTextureFailed => formatter.write_str("failed to create WebGL texture"),
            Self::UploadTextureFailed => formatter.write_str("failed to upload WebGL texture"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_shader_error_details() {
        assert_eq!(
            Error::CompileShaderFailed("bad source".to_owned()).to_string(),
            "failed to compile shader: bad source"
        );
    }
}
