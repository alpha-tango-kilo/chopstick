use std::error::Error;
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, ChopError>;

pub const fn round_up_div(a: u64, b: u64) -> u64 {
    a / b + (a % b != 0) as u64
}

#[derive(Debug)]
pub enum ChopError {
    Io(io::Error),
    ByteSize(String),
    PartSizeTooLarge,
    NumPartsTooLarge,
    InvalidNumParts,
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
