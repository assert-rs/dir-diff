# dir-diff

Are the contents of two directories different?

[![Build Status](https://travis-ci.org/steveklabnik/dir-diff.svg?branch=master)](https://travis-ci.org/steveklabnik/dir-diff)

Mostly useful for integration-style tests when you want to check some generated
output.

## Usage

Using `dir-diff` in an integration test with [`tempdir`]:

```rust,ignore
extern crate dir_diff;
extern crate tempdir;

#[test]
fn smoke_test() {
    let tmp_dir = TempDir::new("foo").expect("create temp dir failed");

    generate_some_stuff(&tmp_dir);

    assert!(!dir_diff::is_different(&tmp_dir.path(), "path/to/fixture").unwrap());
}
```

[`tempdir`]: https://crates.io/crates/tempdir
