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

extern crate walkdir;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use walkdir::WalkDir;

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
}

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
    let a_base = a_base.as_ref();
    let b_base = b_base.as_ref();

    // We start by checking the count in the top-level directories. There's two
    // reasons for this: first, if a is empty, but b is not, our for loop will not
    // catch it; it will execute zero times and return true, and that's wrong!
    // Second, if they're differnet, there's no reason to bother comparing the
    // files; we already know they're different.

    let a_count = std::fs::read_dir(&a_base)?.count();
    let b_count = std::fs::read_dir(&b_base)?.count();

    if a_count != b_count {
        return Ok(true);
    }

    Ok(compare(&a_base, &b_base)? || compare(&b_base, &a_base)?)
}

fn compare(a_base: &Path, b_base: &Path) -> Result<bool, Error> {
    // next, we walk all of the entries in a and compare them to b
    for entry in WalkDir::new(a_base) {
        let entry = entry?;
        let a = entry.path();

        // calculate just the part of the path relative to a
        let no_prefix = a.strip_prefix(a_base)?;

        // and then join that with b to get the path in b
        let b = b_base.join(no_prefix);

        if a.is_dir() {
            if b.is_dir() {
                // can't compare the contents of directories, so just continue
                continue;
            } else {
                // if one is a file and one is a directory, we have a difference!
                return Ok(true)
            }
        }

        // file a is guaranteed to exist...
        let a_text = read_to_vec(a)?;

        // but file b is not. If we have any kind of error when loading
        // it up, that's a positive result, not an actual error.
        let b_text = match read_to_vec(b) {
            Ok(contents) => contents,
            Err(_) => return Ok(true),
        };

        if a_text != b_text {
            return Ok(true);
        }
    }

    // if we made it here, we're good!
    Ok(false)
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