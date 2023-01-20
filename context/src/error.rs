use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ///The Context doesn't fit the requirements
    InvalidContext(String),
    ResourceNotFound,
    FailedToCompileShader(String),
    FailedToLinkShader(String),
    ConversionFailed(&'static str),
    ExternalError(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidContext(error) => write!(f, "invalid context, caused by {error}"),
            Self::ResourceNotFound => write!(f, "resource not found"),
            Self::FailedToLinkShader(error) => {
                write!(f, "failed to link shader, caused by {error}")
            }
            Self::FailedToCompileShader(error) => {
                write!(f, "failed to compile shader stage, caused by {error}")
            }
            Self::ConversionFailed(error) => write!(f, "conversion failed, caused by {error}"),
            Self::ExternalError(error) => write!(f, "external error, caused by {error}"),
        }
    }
}
