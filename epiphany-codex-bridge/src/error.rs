use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpiphanyBridgeError {
    InvalidRequest(String),
    Fatal(String),
}

pub type Result<T> = std::result::Result<T, EpiphanyBridgeError>;

impl fmt::Display for EpiphanyBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) | Self::Fatal(message) => f.write_str(message),
        }
    }
}

impl Error for EpiphanyBridgeError {}
