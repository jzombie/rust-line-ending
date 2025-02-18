# Rust Line Endings

A Rust crate to detect, normalize, and convert line endings across platforms. Ensures consistent handling of `LF`, `CRLF`, and `CR` line endings in text processing.

## Install

```sh
cargo add line-ending
```

## Usage

### Instantiate from string slice

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

### Apply to lines

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

### Convert to another type

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
