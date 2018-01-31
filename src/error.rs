use std::io;

use walkdir;

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    WalkDir(walkdir::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}
