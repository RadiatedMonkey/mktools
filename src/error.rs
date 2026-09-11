use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EncodingError {
    #[error("index out of range: {0}")]
    OutOfRange(String),
    #[error("invalid file: {0}")]
    InvalidFile(String),
    #[error("invalid string:")]
    InvalidString(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("unknown: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for EncodingError {
    fn from(value: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match value.kind() {
            ErrorKind::InvalidData => Self::InvalidFile(value.to_string()),
            ErrorKind::NotFound => Self::NotFound(value.to_string()),
            _ => Self::Unknown(value.to_string()),
        }
    }
}

impl From<std::ffi::FromBytesUntilNulError> for EncodingError {
    fn from(value: std::ffi::FromBytesUntilNulError) -> Self {
        EncodingError::InvalidString(value.to_string())
    }
}

impl From<std::str::Utf8Error> for EncodingError {
    fn from(value: std::str::Utf8Error) -> Self {
        EncodingError::InvalidString(value.to_string())
    }
}

pub type EncodingResult<T> = Result<T, EncodingError>;
