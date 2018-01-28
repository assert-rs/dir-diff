use std::io;
use std::path;

use walkdir;

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    StripPrefix(path::StripPrefixError),
    WalkDir(walkdir::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<path::StripPrefixError> for Error {
    fn from(e: path::StripPrefixError) -> Error {
        Error::StripPrefix(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}
