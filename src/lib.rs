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
    ($table:expr, $file_name:expr, $line_one:expr, $line_two:expr) => {
        $table.add_row(Row::new(vec![
            Cell::new($file_name, 1),
            Cell::new($line_one, 1),
            Cell::new($line_two, 1),
        ]));
    };
}

pub fn see_difference<A: AsRef<Path>, B: AsRef<Path>>(a_base: A, b_base: B) -> Result<(), Error> {
    let mut table = Table::new();
    table.max_column_width = 40;

    table.style = TableStyle::extended();

    let filename_a = &a_base.as_ref().to_str().unwrap();
    let filename_b = &b_base.as_ref().to_str().unwrap();

    table.add_row(Row::new(vec![Cell::new_with_alignment(
        "DIFFERENCES",
        3,
        Alignment::Center,
    )]));

    table.add_row(Row::new(vec![
        Cell::new("Filename", 1),
        Cell::new(filename_a, 1),
        Cell::new(filename_b, 1),
    ]));

    let zipped_file_names = zip_dir_files_to_same_name(
        &walk_dir_and_get_only_files(&a_base),
        &mut walk_dir_and_get_only_files(&b_base),
    );

    for (a, b) in zipped_file_names.into_iter() {
        match (a, b) {
            (Some(i), None) => {
                add_row!(table, i, "FILE EXISTS", "DOESN'T EXIST");
            }

            (None, Some(i)) => {
                add_row!(table, i, "DOESN'T EXIST", "FILE EXISTS");
            }

            (Some(file_1), Some(file_2)) => {
                let mut buffreader_a =
                    BufReader::new(File::open(format!("{}/{}", filename_a, &file_1))?).lines();

                let mut buffreader_b =
                    BufReader::new(File::open(format!("{}/{}", filename_b, &file_2))?).lines();

                let mut line_number = 1;

                loop {
                    match (&buffreader_a.next(), &buffreader_b.next()) {
                        (None, None) => break,

                        (Some(line_a), Some(line_b)) => {
                            match (line_a, line_b) {
                                (Ok(content_a), Ok(content_b)) => if content_a != content_b {
                                    add_row!(
                                        table,
                                        format!("\"{}\":{}", &file_1, line_number),
                                        &content_a,
                                        &content_b
                                    );
                                },
                                (Ok(content_a), Err(_)) => {
                                    add_row!(
                                        table,
                                        format!("\"{}\":{}", &file_1, line_number),
                                        &content_a,
                                        ""
                                    );
                                }
                                (Err(_), Ok(content_b)) => {
                                    add_row!(
                                        table,
                                        format!("\"{}\":{}", &file_1, line_number),
                                        "",
                                        &content_b
                                    );
                                }
                                _ => {}
                            };
                        }

                        (Some(line_a), None) => match line_a {
                            Ok(line_content) => add_row!(
                                table,
                                format!("\"{}\":{}", &file_1, line_number),
                                &line_content,
                                ""
                            ),

                            Err(_) => {
                                add_row!(table, format!("\"{}\":{}", &file_1, line_number), "", "")
                            }
                        },

                        (None, Some(line_b)) => match line_b {
                            Ok(line_content) => add_row!(
                                table,
                                format!("\"{}\":{}", &file_2, line_number),
                                "",
                                &line_content
                            ),
                            Err(_) => {
                                add_row!(table, format!("\"{}\":{}", &file_2, line_number), "", "")
                            }
                        },
                    };

                    line_number += 1;
                }
            }
            _ => {}
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

fn walk_dir_and_get_only_files<P: AsRef<Path>>(path: P) -> Vec<String> {
    WalkDir::new(&path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|a| a.file_type().is_file())
        .into_iter()
        .map(|e| {
            String::from(e.path().to_str().unwrap()).replace(path.as_ref().to_str().unwrap(), "")
        })
        .collect()
}

fn compare_by_file_name(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(b.file_name())
}

fn zip_dir_files_to_same_name<'a>(
    el1: &[String],
    el2: &mut Vec<String>,
) -> Vec<(Option<String>, Option<String>)> {
    let matched_data = el1.iter().fold(
        Vec::<(Option<String>, Option<String>)>::new(),
        |mut previous, current| {
            match el2.into_iter().position(|x| x == current) {
                Some(i) => previous.push((Some(current.to_string()), Some(el2.remove(i)))),
                None => previous.push((Some(current.to_string()), None)),
            };

            return previous;
        },
    );

    el2.into_iter().fold(matched_data, |mut previous, current| {
        previous.push((None, Some(current.to_string())));
        previous
    })
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
