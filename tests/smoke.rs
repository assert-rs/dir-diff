extern crate dir_diff;

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