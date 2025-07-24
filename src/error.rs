use std::fmt;

#[derive(Debug)]
pub enum CalfError {
    Header,
    HeaderExtensionFeatures,
    HeaderExtensions,
    Level,
    SeekFile,
    ReadFile,
}

impl std::error::Error for CalfError {}

impl fmt::Display for CalfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalfError::Header => write!(f, "Could not parse header"),
            CalfError::HeaderExtensionFeatures => write!(f, "Could not parse features extension"),
            CalfError::HeaderExtensions => write!(f, "Could not get header extensions"),
            CalfError::Level => write!(f, "Failed to parse QCOW level"),
            CalfError::SeekFile => write!(f, "Failed to seek to provided offset"),
            CalfError::ReadFile => write!(f, "Failed to read bytes from QCOW file"),
        }
    }
}
