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
use std::path::{Path, PathBuf};

mod error;
mod iter;

pub use error::IoError;

/// Are the contents of two directories different?
///
/// # Examples
///
/// ```no_run
/// extern crate dir_diff;
///
/// assert!(dir_diff::is_different("dir/a", "dir/b").unwrap());
/// ```
pub fn is_different<L, R>(left_root: L, right_root: R) -> Result<bool, IoError>
    where L: Into<PathBuf>,
          R: Into<PathBuf>
{
    for entry in iter::DirDiff::new(left_root, right_root) {
        let entry = entry?;
        let left = entry.left();
        let right = entry.right();

        // Covers missing files because We know that entry can never be missing on both sides
        if left.file_type() != right.file_type() {
            return Ok(true);
        }

        let are_files = left.file_type()
            .expect("exists because of above `file_type` check")
            .is_file();
        if are_files && read_to_vec(left.path())? != read_to_vec(right.path())? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn read_to_vec<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, IoError> {
    let mut data = Vec::new();
    let mut file = File::open(file.as_ref())?;

    file.read_to_end(&mut data)?;

    Ok(data)
}
