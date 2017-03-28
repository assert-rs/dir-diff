extern crate dir_diff;

#[test]
fn easy_good() {
    assert!(dir_diff::is_different("tests/easy/good/dir1", "tests/easy/good/dir2").unwrap());
}

#[test]
fn easy_bad() {
    assert!(!dir_diff::is_different("tests/easy/bad/dir1", "tests/easy/bad/dir2").unwrap());
}