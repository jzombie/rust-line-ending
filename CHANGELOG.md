# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.0] - 2026-10-28

### Added

- Byte-oriented, chunk-safe streaming API over `std::io::BufRead`: `BufReadLineEndingExt::consume_line_ending` (cross-buffer CRLF state machine with `ErrorKind::Interrupted` retries) and the `LinesStreaming` iterator yielding `io::Result<(String, Option<LineEnding>)>`.
- `no_std` partitioning via feature flags (`std` default, `alloc`, pure `core`): `BufRead` API and `LineEndingScores`/`score_mixed_types` are `std`-gated; string-allocating APIs are `alloc`-gated; detection enum and `PeekableLineEndingExt` are core-only.
- README section "Handling Byte Streams (Chunk-Safe)" with a runnable example.
- README: every example annotated with its feature tier (`no_std`/`alloc`/`std`), machine-checked by CI; `no_std` badge; platform-support note.
- `CHANGELOG.md` (this file; backfilled previous entries).

### Changed

- README header: removed the OS test-status table in favor of single `Tests` and `Lint` workflow badges.
- `crates.io` metadata: added `no-std`, `text-processing`, `parsing` categories.
- Dual-licensed under **MIT OR Apache-2.0** (renamed `LICENSE` → `LICENSE-MIT`, added `LICENSE-APACHE`; `Cargo.toml` SPDX expression updated).
- CI hardening: absolute zero-dependency gate (`cargo tree` assertion), README tier-annotation check, feature-tier test matrix (`std`/`alloc`/`core`), `actions/checkout@v7`, toolchain `components: rustfmt, clippy`, `cargo-udeps`/`cargo-sort`/`cargo-deny`/`cargo-audit` checks.

### Removed

- **Zero dependencies**: removed the `doc-comment` dev-dependency. README code blocks are now tested natively via `#![doc = include_str!("../README.md")]`.

## [1.5.1] - 2025-03-03

### Changed

- `doc-comment` demoted from a runtime dependency to a dev-dependency.

## [1.5.0] - 2025-03-02

### Added

- `LineEnding::from_current_platform()` — detect the default line ending for the current OS (Unix: `LF`, Windows: `CRLF`).

## [1.4.0] - 2025-02-26

### Fixed

- Escaped sequences (`\\r`, `\\n`) are no longer consumed as line breaks by the character-stream API; they are treated as content.

## [1.3.1] - 2025-02-22

### Changed

- README organization.

## [1.3.0] - 2025-02-22

### Added

- Character-stream support: `PeekableLineEndingExt::consume_line_ending()` for `Peekable` iterators over `char`, handling `LF`, `CRLF`, and `CR` plus mixed-type streams.
- `LineEnding::as_char()` for single-character line endings (`LF`, `CR`).

## [1.2.0] - 2025-02-18

### Added

- Mixed-type line-ending detection: `LineEnding::score_mixed_types()` and dominant-type selection in `LineEnding::from()`.

## [1.1.0] - 2025-02-18

### Changed

- Renamed `split_into_lines` → `split` and `apply_to_lines` → `join`.
- Note: this rename removed public API and was therefore technically breaking; it shipped ~2 hours after `1.0.0` during the initial release sprint rather than as `2.0.0`. Recorded here as published — history is immutable.

## [1.0.1] - 2025-02-18

### Changed

- README indentation; added lint workflow.

## [1.0.0] - 2025-02-18

### Added

- Initial implementation: `LineEnding` (`LF`, `CRLF`, `CR`) with `normalize`, `denormalize`, `split_into_lines`, `apply_to_lines`, `convert_to`, and `as_str`.
