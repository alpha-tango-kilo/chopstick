use std::error::Error;
use std::path::PathBuf;
use std::{fmt, io};

pub type Result<T, E = ChopError> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum ChopError {
    GenericIo(io::Error),
    ByteSize(String),
    PartSizeTooLarge,
    NumPartsTooLarge,
    InvalidNumParts,
    InsufficientDiskSpace,
    PartFileAlreadyExists(PathBuf),
    FailedToReadPart(io::Error),
    FailedToWritePart(PathBuf, io::Error),
    FailedToTruncate(io::Error),
    FailedToDeleteOriginal(io::Error),
}

impl ChopError {
    pub fn exit_code(&self) -> i32 {
        use ChopError::*;
        match self {
            GenericIo(_) => 2,
            ByteSize(_) => 1,
            PartSizeTooLarge => 1,
            NumPartsTooLarge => 1,
            InvalidNumParts => 1,
            InsufficientDiskSpace => 1,
            PartFileAlreadyExists(_) => 1,
            FailedToReadPart(_) => 2,
            FailedToWritePart(_, _) => 2,
            FailedToTruncate(_) => 2,
            FailedToDeleteOriginal(_) => 2,
        }
    }
}

impl fmt::Display for ChopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChopError::*;
        match self {
            GenericIo(why) => write!(f, "IO error: {}", why),
            ByteSize(why) => write!(f, "Error parsing file size: {}", why),
            PartSizeTooLarge => {
                write!(f, "Part size too large. File wouldn't be split")
            }
            NumPartsTooLarge => write!(
                f,
                "Number of parts too large. Each part would be less than 1 byte"
            ),
            InvalidNumParts => write!(f, "Failed to parse number of parts"),
            InsufficientDiskSpace => {
                write!(f, "Insufficient disk space to perform operation")
            }
            PartFileAlreadyExists(path) => write!(
                f,
                "Part file already exists at {}",
                path.to_string_lossy()
            ),
            FailedToReadPart(why) => {
                write!(f, "Failed to read a part of the original file: {}", why)
            }
            FailedToWritePart(path, why) => write!(
                f,
                "Failed to write to part file {}: {}",
                path.to_string_lossy(),
                why
            ),
            FailedToTruncate(why) => {
                write!(f, "Failed to truncate original file: {}", why)
            }
            FailedToDeleteOriginal(why) => {
                write!(f, "Failed to delete original file: {}", why)
            }
        }
    }
}

impl Error for ChopError {}

impl From<io::Error> for ChopError {
    fn from(err: io::Error) -> Self {
        ChopError::GenericIo(err)
    }
}

// For bytesize errors
impl From<String> for ChopError {
    fn from(string: String) -> Self {
        ChopError::ByteSize(string)
    }
}
