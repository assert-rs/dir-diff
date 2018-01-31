use std::io::prelude::*;
use std::ffi;
use std::fs;
use std::path;

use walkdir;

use error::IoError;
use error::{AssertionKind, AssertionError};

type WalkIter = walkdir::IntoIter;

/// A builder to create an iterator for recusively diffing two directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirDiff {
    left: path::PathBuf,
    right: path::PathBuf,
}

impl DirDiff {
    /// Create a builder for recursively diffing two directories, starting at `left_root` and
    /// `right_root`.
    pub fn new<L, R>(left_root: L, right_root: R) -> Self
        where L: Into<path::PathBuf>,
              R: Into<path::PathBuf>
    {
        Self {
            left: left_root.into(),
            right: right_root.into(),
        }
    }

    fn walk(path: &path::Path) -> WalkIter {
        walkdir::WalkDir::new(path).min_depth(1).into_iter()
    }
}

impl IntoIterator for DirDiff {
    type Item = Result<DiffEntry, IoError>;

    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let left_walk = Self::walk(&self.left);
        let right_walk = Self::walk(&self.right);
        IntoIter {
            left_root: self.left,
            left_walk,
            right_root: self.right,
            right_walk,
        }
    }
}

/// A potential directory entry.
///
/// # Differences with `std::fs::DirEntry`
///
/// This mostly mirrors `DirEntry` in `std::fs` and `walkdir`
///
/// * The path might not actually exist.  In this case, `.file_type()` returns `None`.
/// * Borroed information is returned
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DirEntry {
    path: path::PathBuf,
    file_type: Option<fs::FileType>,
}

impl DirEntry {
    /// The full path that this entry represents.
    pub fn path(&self) -> &path::Path {
        self.path.as_path()
    }

    /// Returns the metadata for the file that this entry points to.
    pub fn metadata(&self) -> Result<fs::Metadata, IoError> {
        let m = fs::metadata(&self.path)?;
        Ok(m)
    }

    /// Returns the file type for the file that this entry points to.
    ///
    /// The `Option` is `None` if the file does not exist.
    pub fn file_type(&self) -> Option<fs::FileType> {
        self.file_type
    }

    /// Returns the file name of this entry.
    ///
    /// If this entry has no file name (e.g. `/`), then the full path is returned.
    pub fn file_name(&self) -> &ffi::OsStr {
        self.path
            .file_name()
            .unwrap_or_else(|| self.path.as_os_str())
    }

    pub(self) fn exists(path: path::PathBuf) -> Result<Self, IoError> {
        let metadata = fs::symlink_metadata(&path)?;
        let file_type = Some(metadata.file_type());
        let s = Self { path, file_type };
        Ok(s)
    }

    pub(self) fn missing(path: path::PathBuf) -> Result<Self, IoError> {
        let file_type = None;
        let s = Self { path, file_type };
        Ok(s)
    }
}

/// To paths to compare.
///
/// This is the type of value that is yielded from `IntoIter`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiffEntry {
    left: DirEntry,
    right: DirEntry,
}

impl DiffEntry {
    /// The entry for the left tree.
    ///
    /// This will always be returned, even if the entry does not exist.  See `DirEntry::file_type`
    /// to see how to check if the path exists.
    pub fn left(&self) -> &DirEntry {
        &self.left
    }

    /// The entry for the right tree.
    ///
    /// This will always be returned, even if the entry does not exist.  See `DirEntry::file_type`
    /// to see how to check if the path exists.
    pub fn right(&self) -> &DirEntry {
        &self.right
    }

    /// Embed the `DiffEntry` into an `AssertionError` for convinience when writing assertions.
    pub fn into_error(self, kind: AssertionKind) -> AssertionError {
        AssertionError::new(kind, self)
    }

    /// Returns an error if the two paths are different.
    ///
    /// If this default policy does not work for you, you can use the constinuent assertions
    /// (e.g. `assert_exists).
    pub fn assert(self) -> Result<Self, AssertionError> {
        match self.file_types() {
            (Some(left), Some(right)) => {
                if left != right {
                    Err(self.into_error(AssertionKind::FileType))
                } else if left.is_file() {
                    // Because of the `left != right` test, we can assume `right` is also a file.
                    match self.content_matches() {
                        Ok(true) => Ok(self),
                        Ok(false) => Err(self.into_error(AssertionKind::Content)),
                        Err(e) => Err(self.into_error(AssertionKind::Content).with_cause(e)),
                    }
                } else {
                    Ok(self)
                }
            }
            _ => Err(self.into_error(AssertionKind::Missing)),
        }
    }

    /// Returns an error iff one of the two paths does not exist.
    pub fn assert_exists(self) -> Result<Self, AssertionError> {
        match self.file_types() {
            (Some(_), Some(_)) => Ok(self),
            _ => Err(self.into_error(AssertionKind::Missing)),
        }
    }

    /// Returns an error iff two paths are of different types.
    pub fn assert_file_type(self) -> Result<Self, AssertionError> {
        match self.file_types() {
            (Some(left), Some(right)) => {
                if left != right {
                    Err(self.into_error(AssertionKind::FileType))
                } else {
                    Ok(self)
                }
            }
            _ => Ok(self),
        }
    }

    /// Returns an error iff the file content of the two paths is different.
    ///
    /// This is assuming they are both files.
    pub fn assert_content(self) -> Result<Self, AssertionError> {
        if !self.are_files() {
            return Ok(self);
        }

        match self.content_matches() {
            Ok(true) => Ok(self),
            Ok(false) => Err(self.into_error(AssertionKind::Content)),
            Err(e) => Err(self.into_error(AssertionKind::Content).with_cause(e)),
        }
    }

    fn file_types(&self) -> (Option<fs::FileType>, Option<fs::FileType>) {
        let left = self.left.file_type();
        let right = self.right.file_type();
        (left, right)
    }

    fn are_files(&self) -> bool {
        let (left, right) = self.file_types();
        let left = left.as_ref().map(fs::FileType::is_file).unwrap_or(false);
        let right = right.as_ref().map(fs::FileType::is_file).unwrap_or(false);
        left && right
    }

    fn content_matches(&self) -> Result<bool, IoError> {
        let left = Self::read_to_vec(self.left.path())?;
        let right = Self::read_to_vec(self.right.path())?;
        Ok(left == right)
    }

    fn read_to_vec(file: &path::Path) -> Result<Vec<u8>, IoError> {
        let mut data = Vec::new();
        let mut file = fs::File::open(file)?;

        file.read_to_end(&mut data)?;

        Ok(data)
    }
}

/// An iterator for recursively diffing two directories.
///
/// To create an `IntoIter`, first create the builder `DirDiff` and call `.into_iter()`.
#[derive(Debug)]
pub struct IntoIter {
    pub(self) left_root: path::PathBuf,
    pub(self) left_walk: WalkIter,
    pub(self) right_root: path::PathBuf,
    pub(self) right_walk: WalkIter,
}

impl IntoIter {
    fn transposed_next(&mut self) -> Result<Option<DiffEntry>, IoError> {
        if let Some(entry) = self.left_walk.next() {
            let entry = entry?;
            let entry_path = entry.path();

            let relative = entry_path
                .strip_prefix(&self.left_root)
                .expect("WalkDir returns items rooted under left_root");
            let right = self.right_root.join(relative);
            let right = if right.exists() {
                DirEntry::exists(right)
            } else {
                DirEntry::missing(right)
            }?;

            // Don't use `walkdir::DirEntry` because its `file_type` came from `fs::read_dir`
            // which we can't reproduce for `right`
            let left = DirEntry::exists(entry_path.to_owned())?;

            let entry = DiffEntry { left, right };
            return Ok(Some(entry));
        }

        while let Some(entry) = self.right_walk.next() {
            let entry = entry?;
            let entry_path = entry.path();

            let relative = entry_path
                .strip_prefix(&self.right_root)
                .expect("WalkDir returns items rooted under right_root");
            let left = self.left_root.join(relative);
            // `left.exists()` was covered above
            if !left.exists() {
                let left = DirEntry::missing(left)?;

                // Don't use `walkdir::DirEntry` because its `file_type` came from `fs::read_dir`
                // which we can't reproduce for `left`
                let right = DirEntry::exists(entry_path.to_owned())?;

                let entry = DiffEntry { left, right };
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }
}

impl Iterator for IntoIter {
    type Item = Result<DiffEntry, IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.transposed_next();
        match item {
            Ok(Some(i)) => Some(Ok(i)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
