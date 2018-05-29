use std::fmt;
use std::fs;
use std::io;

use walkdir;

use super::iter;

/// The type of assertion that occurred.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AssertionKind {
    /// One of the two sides is missing.
    Missing,
    /// The two sides have different types.
    FileType,
    /// The content of the two sides is different.
    Content,
}

impl AssertionKind {
    /// Test if the assertion is from one of the two sides being missing.
    pub fn is_missing(self) -> bool {
        self == AssertionKind::Missing
    }

    /// Test if the assertion is from the two sides having different file types.
    pub fn is_file_type(self) -> bool {
        self == AssertionKind::FileType
    }

    /// Test if the assertion is from the two sides having different content.
    pub fn is_content(self) -> bool {
        self == AssertionKind::Content
    }
}

/// Error to capture the difference between paths.
#[derive(Debug, Clone)]
pub struct AssertionError {
    kind: AssertionKind,
    entry: iter::DiffEntry,
    msg: Option<String>,
    cause: Option<IoError>,
}

impl AssertionError {
    /// The type of difference detected.
    pub fn kind(self) -> AssertionKind {
        self.kind
    }

    /// Access to the `DiffEntry` for which a difference was detected.
    pub fn entry(&self) -> &iter::DiffEntry {
        &self.entry
    }

    /// Underlying error found when trying to find a difference
    pub fn cause(&self) -> Option<&IoError> {
        self.cause.as_ref()
    }

    /// Add an optional message to display with the error.
    pub fn with_msg<S: Into<String>>(mut self, msg: S) -> Self {
        self.msg = Some(msg.into());
        self
    }

    /// Add an underlying error found when trying to find a difference.
    pub fn with_cause<E: Into<IoError>>(mut self, err: E) -> Self {
        self.cause = Some(err.into());
        self
    }

    pub(crate) fn new(kind: AssertionKind, entry: iter::DiffEntry) -> Self {
        Self {
            kind,
            entry,
            msg: None,
            cause: None,
        }
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            AssertionKind::Missing => {
                write!(f,
                       "One side is missing: {}\n  left: {:?}\n  right: {:?}",
                       self.msg.as_ref().map(String::as_str).unwrap_or(""),
                       self.entry.left().path(),
                       self.entry.right().path())
            }
            AssertionKind::FileType => {
                write!(f,
                       "File types differ: {}\n  left: {:?} is {}\n  right: {:?} is {}",
                       self.msg.as_ref().map(String::as_str).unwrap_or(""),
                       self.entry.left().path(),
                       display_file_type(self.entry.left().file_type()),
                       self.entry.right().path(),
                       display_file_type(self.entry.right().file_type()))
            }
            AssertionKind::Content => {
                write!(f,
                       "Content differs: {}\n  left: {:?}\n  right: {:?}",
                       self.msg.as_ref().map(String::as_str).unwrap_or(""),
                       self.entry.left().path(),
                       self.entry.right().path())
            }
        }?;

        if let Some(cause) = self.cause() {
            write!(f, "\ncause: {}", cause)?;
        }

        Ok(())
    }
}

fn display_file_type(file_type: Option<fs::FileType>) -> String {
    if let Some(file_type) = file_type {
        if file_type.is_file() {
            "file".to_owned()
        } else if file_type.is_dir() {
            "dir".to_owned()
        } else {
            format!("{:?}", file_type)
        }
    } else {
        "missing".to_owned()
    }
}

/// IO errors preventing diffing from happening.
#[derive(Debug, Clone)]
pub struct IoError(InnerIoError);

#[derive(Debug)]
enum InnerIoError {
    Io(io::Error),
    WalkDir(walkdir::Error),
    WalkDirEmpty,
}

impl Clone for InnerIoError {
    fn clone(&self) -> Self {
        match *self {
            InnerIoError::Io(_) |
            InnerIoError::WalkDirEmpty => self.clone(),
            InnerIoError::WalkDir(_) => InnerIoError::WalkDirEmpty,
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for InnerIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InnerIoError::Io(ref e) => e.fmt(f),
            InnerIoError::WalkDir(ref e) => e.fmt(f),
            InnerIoError::WalkDirEmpty => write!(f, "Unknown error when walking"),
        }
    }
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
