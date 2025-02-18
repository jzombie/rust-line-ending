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

### Split

Split a string into lines using the detected line ending.

```rust
use line_ending::LineEnding;

let crlf_lines = LineEnding::split("first\r\nsecond\r\nthird");
let cr_lines = LineEnding::split("first\rsecond\rthird");
let lf_lines = LineEnding::split("first\nsecond\nthird");

let expected = vec!["first", "second", "third"];

assert_eq!(crlf_lines, expected);
assert_eq!(cr_lines, expected);
assert_eq!(lf_lines, expected);
```

### Join
Join a vector of strings with the specified line ending.

```rust
use line_ending::LineEnding;

let lines = vec![
  "first".to_string(),
  "second".to_string(),
  "third".to_string(),
];

assert_eq!(
    LineEnding::CRLF.join(lines.clone()),
    "first\r\nsecond\r\nthird"
);
assert_eq!(
    LineEnding::CR.join(lines.clone()),
    "first\rsecond\rthird"
);
assert_eq!(
    LineEnding::LF.join(lines.clone()),
    "first\nsecond\nthird"
);
```

### Apply

Converts a string from any line ending type to a specified one.

```rust
use line_ending::LineEnding;

let mixed_text = "first line\r\nsecond line\rthird line\nfourth line\n";

assert_eq!(
    LineEnding::CRLF.apply(mixed_text),
    "first line\r\nsecond line\r\nthird line\r\nfourth line\r\n"
);
assert_eq!(
    LineEnding::CR.apply(mixed_text),
    "first line\rsecond line\rthird line\rfourth line\r"
);
assert_eq!(
    LineEnding::LF.apply(mixed_text),
    "first line\nsecond line\nthird line\nfourth line\n"
);
```

### From String Slice

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

### Handling Mixed Line Endings

When a string contains multiple types of line endings (`LF`, `CRLF`, and `CR`), the `LineEnding::from` method will detect the most frequent line ending type and return it as the dominant one. This ensures a consistent approach to mixed-line-ending detection.

```rust
use line_ending::LineEnding;

let text = "line1\nline2\r\nline3\rline4\nline5\r\n";

assert_eq!(LineEnding::from(text), LineEnding::LF); // LF is the most common
```

The detection algorithm works as follows:

1. Counts occurrences of each line ending type (`LF`, `CRLF`, `CR`).
2. Selects the most frequent one as the detected line ending.
3. Defaults to `LF` if all are equally present or if the input is empty.

#### Edge Cases & Examples

##### Case 1: One Line Ending Type is Clearly Dominant

```rust
use line_ending::LineEnding;

let mostly_crlf = "line1\r\nline2\r\nline3\nline4\r\nline5\r\n"; 
assert_eq!(LineEnding::from(mostly_crlf), LineEnding::CRLF); // CRLF is the most common

let mostly_cr = "line1\rline2\rline3\nline4\rline5\r"; 
assert_eq!(LineEnding::from(mostly_cr), LineEnding::CR); // CR is the most common
```

##### Case 2: All Line Endings Appear Equally

If `LF`, `CRLF`, and `CR` all appear the same number of times, the function will return CRLF since it has the highest precedence.

```rust
use line_ending::LineEnding;

let equal_mixed = "line1\r\nline2\nline3\rline4\r\nline5\nline6\r"; 
assert_eq!(LineEnding::from(equal_mixed), LineEnding::CRLF); // CRLF > CR > LF
```

##### Case 3: Single Line Containing Multiple Line Endings

If a single line contains different line endings, the function still chooses the most frequent across the entire string.

```rust
use line_ending::LineEnding;

let mixed_on_one_line = "line1\r\nline2\rline3\r\nline4\r\nline5\r"; 
assert_eq!(LineEnding::from(mixed_on_one_line), LineEnding::CRLF); // CRLF appears the most overall
```

##### Case 4: Empty Input Defaults to LF

```rust
use line_ending::LineEnding;

let empty_text = "";
assert_eq!(LineEnding::from(empty_text), LineEnding::LF); // Defaults to LF
```

### Auto-Handling Escaped Line Endings

Rust automatically treats escaped line endings (e.g., `\\n`, `\\r\\n`, `\\r`) as 
literal text and does not interpret them as actual line breaks. This means that 
functions like `split()` and `replace()` operate **only on actual newlines**, 
ensuring efficient and predictable behavior **without additional processing overhead**.

For example:

```rust
use line_ending::LineEnding;

let text = "First\\nSecond\nThird";
let result = LineEnding::split(text);

assert_eq!(result, vec!["First\\nSecond", "Third"]); // Escaped \\n remains intact
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
