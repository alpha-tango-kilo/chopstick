use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fmt, io};
use StickError::*;

pub type Result<T> = std::result::Result<T, StickError>;

#[derive(Debug)]
pub enum StickError {
    NotRecognised(PathBuf),
    Canonicalise(PathBuf, io::Error),
    IncompleteParts(Vec<OsString>),
}

impl StickError {
    pub fn exit_code(&self) -> i32 {
        match self {
            NotRecognised(_) => 1,
            Canonicalise(_, _) => 2,
            IncompleteParts(_) => 1,
        }
    }
}

impl fmt::Display for StickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotRecognised(path) => write!(f, "Not recognised as a chopstick part: {}", path.to_string_lossy()),
            Canonicalise(path, why) => write!(f, "Failed to canonicalise {}: {}\nMaybe try giving the full path instead", path.to_string_lossy(), why),
            IncompleteParts(found) => write!(f, "Couldn't find all the parts to stick, only found the following: {:?}", found),
        }
    }
}

impl Error for StickError {}
