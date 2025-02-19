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

### Split into Multiple Strings

Split a string into a vector of strings using the auto-detected line ending parsed from the string.

```rust
use line_ending::LineEnding;

let crlf = LineEnding::split("first\r\nsecond\r\nthird");
let cr = LineEnding::split("first\rsecond\rthird");
let lf = LineEnding::split("first\nsecond\nthird");

let expected = vec!["first", "second", "third"];

assert_eq!(crlf, expected);
assert_eq!(cr, expected);
assert_eq!(lf, expected);
```

### Join Multiple Strings into a Single String

Join a vector of strings using the specified line ending.

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

### Change Line Ending Type

Apply a specific line ending type to an existing string.

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

### Auto-identify Line Ending Type

Detect the predominant line ending style used in the input string.

```rust
use line_ending::LineEnding;

let crlf = "first line\r\nsecond line\r\nthird line";
let cr = "first line\rsecond line\rthird line";
let lf = "first line\nsecond line\nthird line";

assert_eq!(LineEnding::from(crlf), LineEnding::CRLF);
assert_eq!(LineEnding::from(cr), LineEnding::CR);
assert_eq!(LineEnding::from(lf), LineEnding::LF);
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

let lf = "first\nsecond\nthird";

let crlf_restored = LineEnding::CRLF.denormalize(lf);
let cr_restored = LineEnding::CR.denormalize(lf);
let lf_restored = LineEnding::LF.denormalize(lf);

assert_eq!(crlf_restored, "first\r\nsecond\r\nthird");
assert_eq!(cr_restored, "first\rsecond\rthird");
assert_eq!(lf_restored, "first\nsecond\nthird");
```

### Handling Mixed-Type Line Endings

When a string contains multiple types of line endings (`LF`, `CRLF`, and `CR`), the `LineEnding::from` method will detect the most frequent line ending type and return it as the dominant one. This ensures a consistent approach to mixed-line-ending detection.

```rust
use line_ending::LineEnding;

let mixed_type = "line1\nline2\r\nline3\nline4\nline5\r\n";
assert_eq!(LineEnding::from(mixed_type), LineEnding::LF); // `LF` is the most common
```

The detection algorithm works as follows:

1. Counts occurrences of each line ending type (`LF`, `CRLF`, `CR`).
2. Selects the most frequent one as the detected line ending.
3. Defaults to `CRLF` if all are equally present or if the input is empty.

#### Edge Cases & Examples

##### Case 1: One Line Ending Type is Clearly Dominant

```rust
use line_ending::LineEnding;

let mostly_crlf = "line1\r\nline2\r\nline3\nline4\r\nline5\r\n"; 
assert_eq!(LineEnding::from(mostly_crlf), LineEnding::CRLF); // `CRLF` is the most common

let mostly_cr = "line1\rline2\rline3\nline4\rline5\r"; 
assert_eq!(LineEnding::from(mostly_cr), LineEnding::CR); // `CR` is the most common
```

##### Case 2: All Line Endings Appear Equally

If `LF`, `CRLF`, and `CR` all appear the same number of times, the function will return `CRLF` as a tie-breaker.

```rust
use line_ending::LineEnding;

let equal_mixed = "line1\r\nline2\nline3\rline4\r\nline5\nline6\r"; 
assert_eq!(LineEnding::from(equal_mixed), LineEnding::CRLF); // `CRLF` > `CR` > `LF`
```

*`CRLF` is chosen as a tie-breaker because it represents both `CR` and `LF`, making it the most inclusive option.*

##### Case 3: Single Line Containing Multiple Line Endings

If a single line contains different line endings, the function still chooses the most frequent across the entire string.

```rust
use line_ending::LineEnding;

let mixed_on_one_line = "line1\r\nline2\rline3\r\nline4\r\nline5\r"; 
assert_eq!(LineEnding::from(mixed_on_one_line), LineEnding::CRLF); // `CRLF` appears the most overall
```

##### Case 4: Empty Input Defaults to `CRLF`

```rust
use line_ending::LineEnding;

let empty_text = "";
assert_eq!(LineEnding::from(empty_text), LineEnding::CRLF); // Defaults to `CRLF`
```

#### Additional Mixed-Type Code Examples

##### Counting Mixed Types

Count occurrences of each line ending type in the given string.

```rust
use line_ending::{LineEnding, LineEndingScores};

// `LineEndingScores` is a hash map that associates each line ending type with its occurrence count.









let mostly_lf = "line1\nline2\r\nline3\rline4\nline5\nline6\n";
assert_eq!(LineEnding::from(mostly_lf), LineEnding::LF);
assert_eq!(
    LineEnding::score_mixed_types(mostly_lf,),
    [
        (LineEnding::CRLF, 1),
        (LineEnding::CR, 1),
        (LineEnding::LF, 4),
    ]
    .into_iter()
    .collect::<LineEndingScores>()
);
```

##### Split as a Specific Type

If you want to forcefully split by a certain type.

```rust
use line_ending::{LineEnding};

let mostly_lf = "line1\nline2\r\nline3\rline4\nline5\nline6\n";
let split_crlf = LineEnding::CRLF.split_with(mostly_lf);

assert_eq!(split_crlf, vec!["line1\nline2", "line3\rline4\nline5\nline6\n"]);
```

### Escaped vs. Actual Line Endings

Rust treats `\\n` as a literal sequence rather than an actual newline. This behavior ensures that escaped sequences are not mistakenly interpreted as real line breaks.

For example:

```rust
use line_ending::LineEnding;

let lf_with_escaped = "First\\nSecond\nThird";
let result = LineEnding::split(lf_with_escaped);

assert_eq!(result, vec!["First\\nSecond", "Third"]); // Escaped `\\n` remains intact

let lf = "First\nSecond\nThird";
let result_actual = LineEnding::split(lf);

assert_eq!(result_actual, vec!["First", "Second", "Third"]); // Actual `\n` splits
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
