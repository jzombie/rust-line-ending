# Rust Line Endings

[![made-with-rust][rust-logo]][rust-src-page]
[![crates.io][crates-badge]][crates-page]
[![Documentation][docs-badge]][docs-page]
[![MIT licensed][license-badge]][license-page]


| OS            | Status                                                                               |
|---------------|--------------------------------------------------------------------------------------|
| Ubuntu-latest | [![Ubuntu Tests][ubuntu-latest-badge]][ubuntu-latest-workflow]                       |
| macOS-latest  | [![macOS Tests][macos-latest-badge]][macos-latest-workflow]                          |
| Windows-latest| [![Windows Tests][windows-latest-badge]][windows-latest-workflow]                    |


A Rust crate to detect, normalize, and convert line endings across platforms. Ensures consistent handling of `LF`, `CRLF`, and `CR` line endings in text processing.

## Install

```sh
cargo add line-ending
```

## Usage

### Split into lines

Split a string into lines using the detected line ending.

```rust
use line_ending::LineEnding;

  let crlf_lines = LineEnding::split_into_lines("first\r\nsecond\r\nthird");
  let cr_lines = LineEnding::split_into_lines("first\rsecond\rthird");
  let lf_lines = LineEnding::split_into_lines("first\nsecond\nthird");

  let expected = vec!["first", "second", "third"];

  assert_eq!(crlf_lines, expected);
  assert_eq!(cr_lines, expected);
  assert_eq!(lf_lines, expected);
```

### Apply to lines

Join a vector of strings with the specified line ending.

```rust
use line_ending::LineEnding;

let lines = vec![
  "first".to_string(),
  "second".to_string(),
  "third".to_string(),
];

assert_eq!(
    LineEnding::CRLF.apply_to_lines(lines.clone()),
    "first\r\nsecond\r\nthird"
);
assert_eq!(
    LineEnding::CR.apply_to_lines(lines.clone()),
    "first\rsecond\rthird"
);
assert_eq!(
    LineEnding::LF.apply_to_lines(lines.clone()),
    "first\nsecond\nthird"
);
```

### Convert to type

Convert a string from any line ending type to a specified one.

```rust
use line_ending::LineEnding;

let mixed_text = "first line\r\nsecond line\rthird line\nfourth line\n";

assert_eq!(
    LineEnding::CRLF.convert_to(mixed_text),
    "first line\r\nsecond line\r\nthird line\r\nfourth line\r\n"
);
assert_eq!(
    LineEnding::CR.convert_to(mixed_text),
    "first line\rsecond line\rthird line\rfourth line\r"
);
assert_eq!(
    LineEnding::LF.convert_to(mixed_text),
    "first line\nsecond line\nthird line\nfourth line\n"
);
```

### From string slice

Detect the predominant line ending style used in the input string.

```rust
use line_ending::LineEnding;

let sample = "first line\nsecond line\nthird line";
assert_eq!(LineEnding::from(sample), LineEnding::LF);
```

```rust
use line_ending::LineEnding;

let sample = "first line\r\nsecond line\r\nthird line";
assert_eq!(LineEnding::from(sample), LineEnding::CRLF);
```

```rust
use line_ending::LineEnding;

let sample = "first line\rsecond line\rthird line";
assert_eq!(LineEnding::from(sample), LineEnding::CR);
```

### Normalize

Convert all line endings in a string to LF (`\n`) for consistent processing.

```rust
use line_ending::LineEnding;

let crlf = "first\r\nsecond\r\nthird";
let cr = "first\rsecond\rthird";
let lf = "first\nsecond\nthird";

assert_eq!(LineEnding::normalize(crlf), lf);
assert_eq!(LineEnding::normalize(cr), lf);
assert_eq!(LineEnding::normalize(lf), lf);
```

### Denormalize

Restore line endings in a string to the specified type.

```rust
use line_ending::LineEnding;

let text = "first\nsecond\nthird";
let crlf_restored = LineEnding::CRLF.denormalize(text);
let cr_restored = LineEnding::CR.denormalize(text);
let lf_restored = LineEnding::LF.denormalize(text);

assert_eq!(crlf_restored, "first\r\nsecond\r\nthird");
assert_eq!(cr_restored, "first\rsecond\rthird");
assert_eq!(lf_restored, "first\nsecond\nthird");
```

## License

Licensed under **MIT**. See [`LICENSE`][license-page] for details.




[rust-src-page]: https://www.rust-lang.org/
[rust-logo]: https://img.shields.io/badge/Made%20with-Rust-black?&logo=Rust

[crates-page]: https://crates.io/crates/line-ending
[crates-badge]: https://img.shields.io/crates/v/line-ending.svg

[docs-page]: https://docs.rs/line-ending
[docs-badge]: https://docs.rs/line-ending/badge.svg

[license-page]: https://github.com/jzombie/rust-line-ending/blob/main/LICENSE
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[ubuntu-latest-badge]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20ubuntu-latest)
[ubuntu-latest-workflow]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml?query=branch%3Amain

[macos-latest-badge]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20macos-latest)
[macos-latest-workflow]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml?query=branch%3Amain

[windows-latest-badge]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20windows-latest)
[windows-latest-workflow]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml?query=branch%3Amain
