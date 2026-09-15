#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    CreateShaderFailed,
    CompileShaderFailed(String),
    CreateProgramFailed,
    LinkProgramFailed(String),
    InvalidUniformData(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateShaderFailed => formatter.write_str("failed to create shader"),
            Self::CompileShaderFailed(message) => {
                write!(formatter, "failed to compile shader: {message}")
            }
            Self::CreateProgramFailed => formatter.write_str("failed to create shader program"),
            Self::LinkProgramFailed(message) => {
                write!(formatter, "failed to link shader program: {message}")
            }
            Self::InvalidUniformData(name) => write!(formatter, "invalid uniform data: {name}"),
        }
    }
}

impl std::error::Error for Error {}
