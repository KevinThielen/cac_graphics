use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ///The Context doesn't fit the requirements
    InvalidContext(String),
    ResourceNotFound,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidContext(error) => write!(f, "invalid context, caused by {error}"),
            Self::ResourceNotFound => write!(f, "resource not found"),
        }
    }
}
