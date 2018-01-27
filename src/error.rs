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
