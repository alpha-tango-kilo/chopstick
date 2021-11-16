use std::error::Error;
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, ChopError>;

#[derive(Debug)]
pub enum ChopError {
    Io(io::Error),
    ByteSize(String),
    PartSizeTooLarge,
    NumPartsTooLarge,
    InvalidNumParts,
}

impl ChopError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ChopError::Io(_) => 2,
            ChopError::ByteSize(_) => 1,
            ChopError::PartSizeTooLarge => 1,
            ChopError::NumPartsTooLarge => 1,
            ChopError::InvalidNumParts => 1,
        }
    }
}

impl fmt::Display for ChopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChopError::*;
        match self {
            Io(why) => write!(f, "IO error: {}", why),
            ByteSize(why) => write!(f, "Error parsing file size: {}", why),
            PartSizeTooLarge => write!(f, "Part size too large. File wouldn't be split"),
            NumPartsTooLarge => write!(
                f,
                "Number of parts too large. Each part would be less than 1 byte"
            ),
            InvalidNumParts => write!(f, "Failed to parse number of parts"),
        }
    }
}

impl Error for ChopError {}

impl From<io::Error> for ChopError {
    fn from(err: io::Error) -> Self {
        ChopError::Io(err)
    }
}

// For bytesize errors
impl From<String> for ChopError {
    fn from(string: String) -> Self {
        ChopError::ByteSize(string)
    }
}
