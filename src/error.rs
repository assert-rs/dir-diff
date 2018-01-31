use std::io;

use walkdir;

/// IO errors preventing diffing from happening.
#[derive(Debug)]
pub struct IoError(InnerIoError);

#[derive(Debug)]
enum InnerIoError {
    Io(io::Error),
    WalkDir(walkdir::Error),
}

impl From<io::Error> for IoError {
    fn from(e: io::Error) -> IoError {
        IoError(InnerIoError::Io(e))
    }
}

impl From<walkdir::Error> for IoError {
    fn from(e: walkdir::Error) -> IoError {
        IoError(InnerIoError::WalkDir(e))
    }
}
