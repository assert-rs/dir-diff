extern crate dir_diff;

use std::fs::create_dir;
use std::io::ErrorKind;
use std::path::Path;

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
#[should_panic]
fn firstmissing() {
    assert!(dir_diff::is_different("does_not_exist", "tests/easy/good/dir1").unwrap());
}

#[test]
#[should_panic]
fn secondmissing() {
    assert!(dir_diff::is_different("tests/easy/good/dir1", "does_not_exist").unwrap());
}

#[test]
#[should_panic]
fn bothmissing() {
    assert!(dir_diff::is_different("does_not_exist", "also_does_not_exist").unwrap());
}

#[test]
fn reflexive() {
    assert!(dir_diff::is_different("tests/reflexive/dir1", "tests/reflexive/dir2").unwrap());
}

#[test]
fn binary_permissions() {
    assert!(!dir_diff::is_different("tests/permissions/good/dir1", "tests/permissions/good/dir2").unwrap());
    assert!(dir_diff::is_different("tests/permissions/bad", "tests/permissions/good").unwrap());
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
