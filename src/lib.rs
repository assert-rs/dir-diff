//! Determine if two directories have different contents.
//!
//! For now, only one function exists: are they different, or not? In the future,
//! more functionality to actually determine the difference may be added.
//!
//! # Examples
//!
//! ```no_run
//! extern crate dir_diff;
//!
//! assert!(dir_diff::is_different("dir/a", "dir/b").unwrap());
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate walkdir;

use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(inner) => write!(f, "I/O error: {}", inner),
            Error::StripPrefix(inner) => write!(f, "Strip prefix error: {}", inner),
            Error::WalkDir(inner) => write!(f, "Walk dir error: {}", inner),
        }
    }
}

impl std::error::Error for Error {}

/// Are the contents of two directories different?
///
/// # Examples
///
/// ```no_run
/// extern crate dir_diff;
///
/// assert!(dir_diff::is_different("dir/a", "dir/b").unwrap());
/// ```
pub fn is_different<A: AsRef<Path>, B: AsRef<Path>>(a_base: A, b_base: B) -> Result<bool, Error> {
    let mut a_walker = walk_dir(a_base)?;
    let mut b_walker = walk_dir(b_base)?;

    for (a, b) in (&mut a_walker).zip(&mut b_walker) {
        let a = a?;
        let b = b?;

        if a.depth() != b.depth()
            || a.file_type() != b.file_type()
            || a.file_name() != b.file_name()
            || (a.file_type().is_file() && read_to_vec(a.path())? != read_to_vec(b.path())?)
        {
            return Ok(true);
        }
    }

    Ok(a_walker.next().is_some() || b_walker.next().is_some())
}

fn walk_dir<P: AsRef<Path>>(path: P) -> Result<walkdir::IntoIter, std::io::Error> {
    let mut walkdir = WalkDir::new(path).sort_by(compare_by_file_name).into_iter();
    if let Some(Err(e)) = walkdir.next() {
        Err(e.into())
    } else {
        Ok(walkdir)
    }
}

fn compare_by_file_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(b.file_name())
}

fn read_to_vec<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, std::io::Error> {
    let mut data = Vec::new();
    let mut file = File::open(file.as_ref())?;

    file.read_to_end(&mut data)?;

    Ok(data)
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(e: std::path::StripPrefixError) -> Error {
        Error::StripPrefix(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        use std::io::ErrorKind;

        assert_eq!(
            format!(
                "{}",
                Error::Io(std::io::Error::new(ErrorKind::Other, "oh no!"))
            ),
            "I/O error: oh no!"
        );
    }
}
