use std::fs;
use std::path;

use walkdir;

use error::Error;

type WalkIter = walkdir::IntoIter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirDiff {
    left: path::PathBuf,
    right: path::PathBuf,
}

impl DirDiff {
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
    type Item = Result<DiffEntry, Error>;

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

#[derive(Debug, Clone)]
pub struct DirEntry {
    path: path::PathBuf,
    file_type: Option<fs::FileType>,
}

impl DirEntry {
    pub(self) fn exists(path: path::PathBuf) -> Result<Self, Error> {
        let metadata = fs::symlink_metadata(&path)?;
        let file_type = Some(metadata.file_type());
        let s = Self { path, file_type };
        Ok(s)
    }

    pub(self) fn missing(path: path::PathBuf) -> Result<Self, Error> {
        let file_type = None;
        let s = Self { path, file_type };
        Ok(s)
    }

    pub fn path(&self) -> &path::Path {
        self.path.as_path()
    }

    pub fn file_type(&self) -> Option<fs::FileType> {
        self.file_type
    }
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    left: DirEntry,
    right: DirEntry,
}

impl DiffEntry {
    pub fn left(&self) -> &DirEntry {
        &self.left
    }

    pub fn right(&self) -> &DirEntry {
        &self.right
    }
}

#[derive(Debug)]
pub struct IntoIter {
    pub(self) left_root: path::PathBuf,
    pub(self) left_walk: WalkIter,
    pub(self) right_root: path::PathBuf,
    pub(self) right_walk: WalkIter,
}

impl IntoIter {
    fn transposed_next(&mut self) -> Result<Option<DiffEntry>, Error> {
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
    type Item = Result<DiffEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.transposed_next();
        match item {
            Ok(Some(i)) => Some(Ok(i)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
