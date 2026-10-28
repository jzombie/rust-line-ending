# Rust Line Endings

[![made-with-rust][rust-logo]][rust-src-page]
[![no_std][nostd-badge]](#no_std-support)
[![crates.io][crates-badge]][crates-page]
[![Documentation][docs-badge]][docs-page]
[![MIT/Apache-2.0 licensed][license-badge]][license-page]
[![Tests][tests-badge]][tests-workflow]
[![Lint][lint-badge]][lint-workflow]


A Rust crate to detect, normalize, and convert line endings across platforms, including support for character streams. Ensures consistent handling of `LF`, `CRLF`, and `CR` line endings in text processing.

Supported on any platform Rust targets, including [`no_std` environments](#no_std-support). CI runs the full test suite on Ubuntu, macOS, and Windows.

## Install

```sh
cargo add line-ending
```

## Usage

### Detect for Current Platform

Line endings can be auto-detected for the current platform.

```rust
use line_ending::LineEnding;

let detected = LineEnding::from_current_platform();

#[cfg(target_os = "windows")]
assert_eq!(detected, LineEnding::CRLF, "Windows should detect CRLF");

#[cfg(target_family = "unix")]
assert_eq!(detected, LineEnding::LF, "Unix/macOS should detect LF");

#[cfg(target_family = "wasm")]
assert_eq!(detected, LineEnding::LF, "WASM should default to LF");
```

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

Convert all line endings in a string to `LF` (`\n`) for consistent processing.

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
// `LineEndingScores` is a hash map that associates each line ending type with
// its occurrence count.

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

### Handling Character Streams

When processing text from a stream (for example, when reading from a file), you often work with a `Peekable` iterator over characters. Manually checking for a newline (such as '\n') isn’t enough to handle all platforms, because Windows uses a two‑character sequence (`\r\n`) and some older systems use just `\r`.

This crate provides a trait extension (via the `PeekableLineEndingExt` trait) that adds a consume_line_ending() method to a `Peekable<Chars>` iterator. This method automatically detects and consumes the full line break sequence (whether it’s LF, CR, or CRLF) from the stream.

The following example demonstrates how to split a character stream into lines without having to manually handle each line-ending case:

```rust
use line_ending::{LineEnding, PeekableLineEndingExt};

let text = "line1\r\nline2\nline3\rline4";
let mut it = text.chars().peekable();
let mut lines = Vec::new();
let mut current_line = String::new();

while it.peek().is_some() {
    // consume_line_ending() will automatically consume the full line break (CR, LF, or CRLF)
    if it.consume_line_ending().is_some() {
        lines.push(current_line);
        current_line = String::new();
    } else {
        current_line.push(it.next().unwrap());
    }
}
lines.push(current_line);

assert_eq!(lines, vec!["line1", "line2", "line3", "line4"]);
```

_Note: Mixed-type line-ending character streams are automatically handled._

### `no_std` Support

The crate is partitioned into three tiers via feature flags (default is `std`, i.e. everything):

| Tier | Feature flags | API surface |
|------|---------------|-------------|
| `core` (no alloc) | `--no-default-features` | `LineEnding` detection (`from`, `as_str`, `as_char`, `from_current_platform`), `PeekableLineEndingExt` |
| `alloc` | `--no-default-features --features alloc` | Above plus `normalize`, `denormalize`, `split`, `split_with`, `join`, `apply` |
| `std` (default) | (default) | Above plus `BufReadLineEndingExt`, `LinesStreaming`, `LineEndingScores` / `score_mixed_types` |

### Handling Byte Streams (Chunk-Safe)

When bytes arrive over I/O (files, sockets, pipes), a `\r\n` pair can be split across two reads. A plain `char` iterator cannot stitch across that boundary (each chunk's end looks like EOF), so this crate provides a byte-oriented API built on `BufRead` with an explicit cross-buffer state machine.

`LinesStreaming` yields `(line, terminator)` pairs, where the final unterminated line yields `None`. I/O and UTF-8 errors are yielded as `Err` (like `std::io::Lines`). Unlike the `char`-based API above, a backslash is treated as an ordinary byte — escape sequences are not interpreted on the wire.

```rust
use line_ending::{LineEnding, LinesStreaming};
use std::io::BufReader;

let data = b"line1\r\nline2\nline3\rline4";
let reader = BufReader::new(&data[..]);
let lines: Vec<(String, Option<LineEnding>)> = LinesStreaming::new(reader)
    .collect::<std::io::Result<_>>()
    .expect("streaming should succeed");

assert_eq!(
    lines,
    vec![
        ("line1".to_string(), Some(LineEnding::CRLF)),
        ("line2".to_string(), Some(LineEnding::LF)),
        ("line3".to_string(), Some(LineEnding::CR)),
        ("line4".to_string(), None),
    ]
);
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

Licensed under **MIT OR Apache-2.0**. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE) for details.

See [`CHANGELOG.md`](CHANGELOG.md) for release history.




[rust-src-page]: https://www.rust-lang.org/
[rust-logo]: https://img.shields.io/badge/Made%20with-Rust-black?&logo=Rust

[nostd-badge]: https://img.shields.io/badge/no__std-supported-blue

[crates-page]: https://crates.io/crates/line-ending
[crates-badge]: https://img.shields.io/crates/v/line-ending.svg

[docs-page]: https://docs.rs/line-ending
[docs-badge]: https://docs.rs/line-ending/badge.svg

[license-page]: https://github.com/jzombie/rust-line-ending/blob/main/LICENSE-MIT
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg

[tests-badge]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml/badge.svg?branch=main
[tests-workflow]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-tests.yml?query=branch%3Amain

[lint-badge]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-lint.yml/badge.svg?branch=main
[lint-workflow]: https://github.com/jzombie/rust-line-ending/actions/workflows/rust-lint.yml?query=branch%3Amain
