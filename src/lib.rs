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

extern crate term_table;
extern crate walkdir;

use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
use term_table::{
    cell::{Alignment, Cell}, row::Row, Table, TableStyle,
};
use walkdir::{DirEntry, WalkDir};

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

macro_rules! add_row {
    (&mut $table:expr, $file_name:expr, $line_one:expr, $line_two:expr) => {
        $table.add_row(Row::new(vec![
            Cell::new($file_name, 1),
            Cell::new($line_one, 1),
            Cell::new($line_two, 1),
        ]));
    };
}

pub fn see_difference<A: AsRef<Path>, B: AsRef<Path>>(a_base: A, b_base: B) -> Result<(), Error> {
    let mut a_walker = walk_dir(a_base);
    let mut b_walker = walk_dir(b_base);

    let mut table = Table::new();
    table.max_column_width = 40;

    table.style = TableStyle::extended();

    table.add_row(Row::new(vec![Cell::new_with_alignment(
        "Differences",
        3,
        Alignment::Center,
    )]));

    for (a, b) in (&mut a_walker).zip(&mut b_walker) {
        let a = a?;
        let b = b?;

        let lines = BufReader::new(File::open(b.path())?)
            .lines()
            .zip(BufReader::new(File::open(a.path())?).lines());

        for (line_for_a, line_for_b) in lines {
            match (line_for_a, line_for_b) {
                (Ok(content_a), Ok(content_b)) => if content_a != content_b {
                    add_row!(
                        &mut table,
                        a.path().to_string_lossy(),
                        &content_a,
                        &content_b
                    );
                },
                (Ok(content_a), Err(_)) => {
                    add_row!(&mut table, a.path().to_string_lossy(), &content_a, "");
                }
                (Err(_), Ok(content_b)) => {
                    add_row!(&mut table, a.path().to_string_lossy(), "", &content_b);
                }
                _ => {}
            };
        }
    }

    println!("{}", table.as_string());
    Ok(())
}

fn walk_dir<P: AsRef<Path>>(path: P) -> std::iter::Skip<walkdir::IntoIter> {
    WalkDir::new(path)
        .sort_by(compare_by_file_name)
        .into_iter()
        .skip(1)
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
