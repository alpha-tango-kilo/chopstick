use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fmt, io};
use StickError::*;

pub type Result<T> = std::result::Result<T, StickError>;

#[derive(Debug)]
pub enum StickError {
    BadParent(io::Error),
    NoParts,
    IncompleteParts(Vec<OsString>),
    InsufficientDiskSpace,
    CreateOriginal(PathBuf, io::Error),
    ReadPart(PathBuf, io::Error),
    WriteOriginal(io::Error),
    DeletePart(PathBuf, io::Error),
}

impl StickError {
    pub fn exit_code(&self) -> i32 {
        match self {
            BadParent(_) => 1,
            NoParts => 1,
            IncompleteParts(_) => 1,
            InsufficientDiskSpace => 1,
            CreateOriginal(_, _) => 2,
            ReadPart(_, _) => 2,
            WriteOriginal(_) => 2,
            DeletePart(_, _) => 2,
        }
    }
}

impl fmt::Display for StickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadParent(why) => write!(f, "Unable to determine or access parent folder: {}", why),
            NoParts => write!(f, "No parts were found to stick"),
            IncompleteParts(found) => write!(f, "Couldn't find all the parts to stick, only found the following: {:?}", found),
            InsufficientDiskSpace => write!(f, "Insufficient disk space to perform operation"),
            CreateOriginal(path, why) => write!(f, "Failed to create file {}: {}", path.to_string_lossy(), why),
            ReadPart(path, why) => write!(f, "Couldn't read part {}: {}", path.to_string_lossy(), why),
            WriteOriginal(why) => write!(f, "Couldn't write to original file: {}", why),
            DeletePart(path, why) => write!(f, "Couldn't delete part {}: {}", path.to_string_lossy(), why),
        }
    }
}

impl Error for StickError {}
