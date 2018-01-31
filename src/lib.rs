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

mod error;
mod iter;

use std::path::PathBuf;

pub use error::IoError;
pub use error::{AssertionKind, AssertionError};
pub use iter::{DirDiff, DirEntry, DiffEntry, IntoIter};

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
        if entry?.assert().is_err() {
            return Ok(true);
        }
    }

    Ok(false)
}
