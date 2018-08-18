extern crate dir_diff;
#[macro_use]
extern crate matches;

use dir_diff::Error::*;
use std::fs::create_dir;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[test]
fn easy_good() {
    assert!(!dir_diff::is_different("tests/easy/good/dir1", "tests/easy/good/dir2").unwrap());
}

#[test]
fn easy_bad() {
    assert!(dir_diff::is_different("tests/easy/bad/dir1", "tests/easy/bad/dir2").unwrap());
}

#[test]
fn binary_good() {
    assert!(!dir_diff::is_different("tests/binary/good/dir1", "tests/binary/good/dir2").unwrap());
}

#[test]
fn binary_bad() {
    assert!(dir_diff::is_different("tests/binary/bad/dir1", "tests/binary/bad/dir2").unwrap());
}

#[test]
fn fileanddir() {
    assert!(dir_diff::is_different("tests/fileanddir/dir1", "tests/fileanddir/dir2").unwrap());
}

#[test]
fn oneempty() {
    assert!(dir_diff::is_different("tests/oneempty/dir1", "tests/oneempty/dir2").unwrap());
}

#[test]
fn reflexive() {
    assert!(dir_diff::is_different("tests/reflexive/dir1", "tests/reflexive/dir2").unwrap());
}

#[test]
fn dirs_differ() {
    assert!(dir_diff::is_different("tests/dirs_differ/dir1", "tests/dirs_differ/dir2").unwrap());
}

fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    match create_dir(path) {
        Err(ref err) if err.kind() == ErrorKind::AlreadyExists => Ok(()),
        other => other,
    }
}

#[test]
fn filedepth() {
    ensure_dir("tests/filedepth/asc/dir2/a").unwrap();
    ensure_dir("tests/filedepth/desc/dir1/b").unwrap();

    assert!(
        dir_diff::is_different("tests/filedepth/asc/dir1", "tests/filedepth/asc/dir2").unwrap()
    );
    assert!(
        dir_diff::is_different("tests/filedepth/desc/dir1", "tests/filedepth/desc/dir2").unwrap()
    );
}

#[test]
fn missing_file() {
    assert_matches!(
        dir_diff::see_difference("tests/missing_file/dir1", "tests/missing_file/dir2"),
        Err(MissingFiles)
    );

    assert_matches!(
        dir_diff::see_difference("tests/missing_dir/dir1", "tests/missing_dir/dir2"),
        Err(MissingFiles)
    );
}

#[test]
fn file_length_mismatch() {
    assert_matches!(
        dir_diff::see_difference(
            "tests/file_length_mismatch/dir1",
            "tests/file_length_mismatch/dir2",
        ),
        Err(FileLengthMismatch(_, _))
    );
}

#[test]
fn binary_content_mismatch() {
    let expected_binary_filename_a = PathBuf::from("rust-logo.png");
    let expected_binary_filename_b = PathBuf::from("rust-logo.png");

    let result = dir_diff::see_difference("tests/binary/bad/dir1", "tests/binary/bad/dir2");
    assert_matches!(result, Err(BinaryContentMismatch(_, _)));

    let result = result.unwrap_err();
    if let BinaryContentMismatch(a, b) = &result {
        if *a != expected_binary_filename_a || *b != expected_binary_filename_b {
            let expected = FileNameMismatch(expected_binary_filename_a, expected_binary_filename_b);
            panic!("{:?} doesn't match {:?}", &result, expected);
        }
    };
}

#[test]
fn dir_name_mismatch() {
    let expected_dir_a = PathBuf::from("dirA");
    let expected_dir_b = PathBuf::from("dirB");

    let result = dir_diff::see_difference(
        "tests/dir_name_mismatch/dir1",
        "tests/dir_name_mismatch/dir2",
    );
    assert_matches!(result, Err(FileNameMismatch(_, _)));

    let result = result.unwrap_err();
    if let FileNameMismatch(a, b) = &result {
        if *a != expected_dir_a || *b != expected_dir_b {
            let expected = FileNameMismatch(expected_dir_a, expected_dir_b);
            panic!("{:?} doesn't match {:?}", &result, expected);
        }
    };
}

#[test]
fn file_name_mismatch() {
    let expected_file_a = PathBuf::from("b.txt");
    let expected_file_b = PathBuf::from("a.txt");

    let result = dir_diff::see_difference(
        "tests/file_name_mismatch/dir1",
        "tests/file_name_mismatch/dir2",
    );
    assert_matches!(result, Err(FileNameMismatch(_, _)));

    let result = result.unwrap_err();
    if let FileNameMismatch(a, b) = &result {
        if *a != expected_file_a || *b != expected_file_b {
            let expected = FileNameMismatch(expected_file_a, expected_file_b);
            panic!("{:?} doesn't match {:?}", &result, expected);
        }
    };
}

#[test]
fn content_misatch() {
    let expected_a_path = PathBuf::from("test.txt");
    let expected_b_path = PathBuf::from("test.txt");
    let expected_a_content = String::from("testing testing");
    let expected_b_content = String::from("oh no!");

    let result =
        dir_diff::see_difference("tests/content_mismatch/dir1", "tests/content_mismatch/dir2");

    assert_matches!(result, Err(ContentMismatch { .. }));
    let result = result.unwrap_err();

    // Match the ContentMismatch result with th expected values.
    if let ContentMismatch {
        line_number,
        a_path,
        b_path,
        a_content,
        b_content,
    } = &result
    {
        if *line_number != 0
            || *a_path != expected_a_path
            || *b_path != expected_b_path
            || *a_content != expected_a_content
            || *b_content != expected_b_content
        {
            let expected = ContentMismatch {
                line_number: 0,
                a_path: expected_a_path,
                b_path: expected_b_path,
                a_content: expected_a_content,
                b_content: expected_b_content,
            };

            panic!("{:?} doesn't match {:?}", &result, expected);
        }
    }
}
