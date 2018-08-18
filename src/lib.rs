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
//!

extern crate walkdir;

use std::cmp::Ordering;
use std::io::prelude::Read;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, fs::File};
use walkdir::{DirEntry, WalkDir};

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
    /// One directory has more or less files than the other.
    MissingFiles,
    /// File name doesn't match.
    FileNameMismatch(PathBuf, PathBuf),
    /// Binary contetn doesn't match.
    BinaryContentMismatch(PathBuf, PathBuf),
    /// One file has more or less lines than the other.
    FileLengthMismatch(PathBuf, PathBuf),
    /// The content of a file doesn't match.
    ContentMismatch {
        line_number: usize,
        a_path: PathBuf,
        b_path: PathBuf,
        a_content: String,
        b_content: String,
    },
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
    let mut a_walker = walk_dir(a_base);
    let mut b_walker = walk_dir(b_base);

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

    Ok(!a_walker.next().is_none() || !b_walker.next().is_none())
}

/// Identify the differences between two directories.
///
/// # Examples
///
/// ```no_run
/// extern crate dir_diff;
///
/// assert_eq!(dir_diff::see_difference("main/dir1", "main/dir1").unwrap(), ());
/// ```
pub fn see_difference<A: AsRef<Path>, B: AsRef<Path>>(a_base: A, b_base: B) -> Result<(), Error> {
    let mut files_a = walk_dir_and_strip_prefix(&a_base)
        .into_iter()
        .collect::<Vec<_>>();
    let mut files_b = walk_dir_and_strip_prefix(&b_base)
        .into_iter()
        .collect::<Vec<_>>();

    if files_a.len() != files_b.len() {
        return Err(Error::MissingFiles);
    }

    files_a.sort();
    files_b.sort();

    for (a, b) in files_a.into_iter().zip(files_b.into_iter()).into_iter() {
        if a != b {
            return Err(Error::FileNameMismatch(a, b));
        }

        let full_path_a = &a_base.as_ref().join(&a);
        let full_path_b = &b_base.as_ref().join(&b);

        if full_path_a.is_dir() || full_path_b.is_dir() {
            continue;
        }

        let content_of_a = fs::read(full_path_a)?;
        let content_of_b = fs::read(full_path_b)?;

        match (
            String::from_utf8(content_of_a),
            String::from_utf8(content_of_b),
        ) {
            (Err(content_of_a), Err(content_of_b)) => {
                if content_of_a.as_bytes() != content_of_b.as_bytes() {
                    return Err(Error::BinaryContentMismatch(a, b));
                }
            }
            (Ok(content_of_a), Ok(content_of_b)) => {
                let mut a_lines = content_of_a.lines().collect::<Vec<&str>>();
                let mut b_lines = content_of_b.lines().collect::<Vec<&str>>();

                if a_lines.len() != b_lines.len() {
                    return Err(Error::FileLengthMismatch(a, b));
                }

                for (line_number, (line_a, line_b)) in
                    a_lines.into_iter().zip(b_lines.into_iter()).enumerate()
                {
                    if line_a != line_b {
                        return Err(Error::ContentMismatch {
                            a_path: a,
                            b_path: b,
                            a_content: line_a.to_string(),
                            b_content: line_b.to_string(),
                            line_number,
                        });
                    }
                }
            }
            _ => return Err(Error::BinaryContentMismatch(a, b)),
        }
    }

    Ok(())
}

fn walk_dir<P: AsRef<Path>>(path: P) -> std::iter::Skip<walkdir::IntoIter> {
    WalkDir::new(path)
        .sort_by(compare_by_file_name)
        .into_iter()
        .skip(1)
}

/// Iterated through a directory, and strip t
fn walk_dir_and_strip_prefix<'a, P>(path: P) -> impl Iterator<Item = PathBuf>
where
    P: AsRef<Path> + Copy,
{
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        // .filter(|a| a.to_owned().file_type().is_file())
        .filter_map(move |e| {
            let new_path = e.path();
            new_path.strip_prefix(&path).map(|e| e.to_owned()).ok()
        })
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
