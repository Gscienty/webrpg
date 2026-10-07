use crate::{shader, texture};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Shader(shader::Error),
    Texture(texture::Error),
    Mpq(String),
}

impl From<shader::Error> for Error {
    fn from(error: shader::Error) -> Self {
        Self::Shader(error)
    }
}

impl From<texture::Error> for Error {
    fn from(error: texture::Error) -> Self {
        Self::Texture(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shader(error) => error.fmt(formatter),
            Self::Texture(error) => error.fmt(formatter),
            Self::Mpq(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Shader(error) => Some(error),
            Self::Texture(error) => Some(error),
            Self::Mpq(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_shader_errors() {
        let source = shader::Error::CreateShaderFailed;
        assert_eq!(Error::from(source.clone()), Error::Shader(source));
    }

    #[test]
    fn converts_texture_errors() {
        let source = texture::Error::CreateTextureFailed;
        assert_eq!(Error::from(source), Error::Texture(source));
    }
}
