# dir-diff

Are the contents of two directories different?

[![Travis Status](https://travis-ci.org/assert-rs/dir-diff.svg?branch=master)](https://travis-ci.org/assert-rs/dir-diff)
[![Appveyor Status](https://ci.appveyor.com/api/projects/status/xsayr0kcerir694j/branch/master?svg=true)](https://ci.appveyor.com/project/epage/dir-diff/branch/master)
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/dir-diff.svg)
[![Crates Status](https://img.shields.io/crates/v/dir-diff.svg)](https://crates.io/crates/dir-diff)

Mostly useful for integration-style tests when you want to check some generated
output.

## Example

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

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/dir-diff
[Documentation]: https://docs.rs/dir-diff
[`tempdir`]: https://crates.io/crates/tempdir
