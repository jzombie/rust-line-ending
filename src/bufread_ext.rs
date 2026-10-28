//! Byte-oriented, chunk-safe line-ending handling over [`BufRead`].
//!
//! The [`PeekableLineEndingExt`][crate::PeekableLineEndingExt] API works on
//! in-memory `char` iterators. When bytes arrive over I/O (files, sockets,
//! pipes), a `\r\n` pair can be split across two reads: a standard `Iterator`
//! reports the end of each chunk as `None` (exhausted), which is
//! indistinguishable from true EOF, so stitching across `None` boundaries is
//! unsound.
//!
//! This module solves that with an explicit cross-buffer state machine built
//! on [`BufRead::fill_buf`] / [`BufRead::consume`]:
//!
//! - When `b'\r'` is the last byte of the buffered slice, the implementation
//!   records the pending-CR state, consumes the slice, and issues a second
//!   `fill_buf()` call to inspect the first byte of the next chunk for
//!   `b'\n'` (CRLF) versus anything else / EOF (lone CR).
//! - Every `fill_buf()` call — including the lookahead call — retries on
//!   [`std::io::ErrorKind::Interrupted`] without touching pending state.
//!   Other I/O errors are surfaced to the caller.
//! - Only the bytes `b'\r'` and `b'\n'` are inspected; line content is
//!   decoded as UTF-8, with decoding failures reported as
//!   [`std::io::ErrorKind::InvalidData`].
//!
//! Wire mode vs. literal mode: unlike the `char`-based API, this module
//! treats a backslash (`b'\\'`) as an ordinary byte. Rust escape sequences
//! such as `\\r` / `\\n` are a source-level concern and are not interpreted
//! here.

use crate::LineEnding;
use std::io::{self, BufRead};

/// Byte-oriented extension for [`BufRead`] streams (wire mode).
///
/// Detects and consumes `LF`, `CRLF`, and `CR` line breaks across I/O chunk
/// boundaries. A backslash byte is treated as ordinary content (no escape
/// handling); see the [module-level docs](self) for the wire-vs-literal
/// distinction.
pub trait BufReadLineEndingExt: BufRead {
    /// Consumes a single line ending at the current position, if present.
    ///
    /// Returns `Ok(None)` at EOF or when the next byte is not `\r` or `\n`.
    /// `Interrupted` I/O errors are retried internally; all other I/O
    /// errors are returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use line_ending::{LineEnding, BufReadLineEndingExt};
    /// use std::io::BufReader;
    ///
    /// let mut reader = BufReader::new(b"\r\nhello" as &[u8]);
    /// assert_eq!(
    ///     reader.consume_line_ending().expect("read should succeed"),
    ///     Some(LineEnding::CRLF)
    /// );
    /// ```
    fn consume_line_ending(&mut self) -> io::Result<Option<LineEnding>>;

    /// Iterates over `(line, terminator)` pairs until EOF.
    ///
    /// The final line yields `None` as its terminator when the stream ends
    /// without a trailing break. I/O and UTF-8 errors are yielded as `Err`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use line_ending::{LineEnding, BufReadLineEndingExt};
    /// use std::io::BufReader;
    ///
    /// let reader = BufReader::new(b"a\r\nb\nc\rd" as &[u8]);
    /// let lines: Vec<(String, Option<LineEnding>)> = reader
    ///     .lines_streaming()
    ///     .collect::<std::io::Result<_>>()
    ///     .expect("streaming should succeed");
    /// assert_eq!(
    ///     lines,
    ///     vec![
    ///         ("a".to_string(), Some(LineEnding::CRLF)),
    ///         ("b".to_string(), Some(LineEnding::LF)),
    ///         ("c".to_string(), Some(LineEnding::CR)),
    ///         ("d".to_string(), None),
    ///     ]
    /// );
    /// ```
    fn lines_streaming(self) -> LinesStreaming<Self>
    where
        Self: Sized,
    {
        LinesStreaming::new(self)
    }
}

impl<R: BufRead> BufReadLineEndingExt for R {
    fn consume_line_ending(&mut self) -> io::Result<Option<LineEnding>> {
        enum Decision {
            None,
            Lf,
            Cr,
            Crlf,
            CrTerminal,
        }

        // Single-view inspection: `fill_buf` is idempotent until `consume`,
        // so buf[0]/buf[1] observed here remain valid for the consume below.
        let decision = loop {
            match self.fill_buf() {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
                Ok(buf) => {
                    break match buf.first() {
                        None => Decision::None,
                        Some(b'\n') => Decision::Lf,
                        Some(b'\r') if buf.len() == 1 => Decision::CrTerminal,
                        Some(b'\r') if buf[1] == b'\n' => Decision::Crlf,
                        Some(b'\r') => Decision::Cr,
                        Some(_) => Decision::None,
                    };
                }
            }
        };

        match decision {
            Decision::None => Ok(None),
            Decision::Lf => {
                self.consume(1);
                Ok(Some(LineEnding::LF))
            }
            Decision::Cr => {
                self.consume(1);
                Ok(Some(LineEnding::CR))
            }
            Decision::Crlf => {
                self.consume(2);
                Ok(Some(LineEnding::CRLF))
            }
            Decision::CrTerminal => {
                // `b'\r'` ended the buffered slice: consume it, then look
                // ahead at the next chunk. Pending-CR lives in control flow;
                // `Interrupted` retries must not disturb it.
                self.consume(1);
                let is_lf = loop {
                    match self.fill_buf() {
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                        Ok(buf) => break buf.first().copied() == Some(b'\n'),
                    }
                };
                if is_lf {
                    self.consume(1);
                    Ok(Some(LineEnding::CRLF))
                } else {
                    Ok(Some(LineEnding::CR))
                }
            }
        }
    }
}

/// Iterator over `(line_content, terminator)` pairs from a [`BufRead`].
///
/// Created via [`BufReadLineEndingExt::lines_streaming`]. Like
/// [`std::io::Lines`], errors are yielded as `Err` (UTF-8 failures as
/// [`std::io::ErrorKind::InvalidData`]) and iteration continues afterwards;
/// `None` is returned only after a clean EOF with no buffered content.
pub struct LinesStreaming<R> {
    inner: R,
    done: bool,
}

impl<R> LinesStreaming<R> {
    /// Wraps a [`BufRead`] reader for line-by-line streaming.
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            done: false,
        }
    }
}

impl<R: BufRead> Iterator for LinesStreaming<R> {
    type Item = io::Result<(String, Option<LineEnding>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        enum Outcome {
            /// Plain content: `n` buffered bytes were appended to the line.
            Consume(usize),
            /// Line complete: `n` buffered bytes consumed, emit with ending.
            Emit(usize, Option<LineEnding>),
            /// Buffer ended with `b'\r'`: prefix appended, `n` bytes (through
            /// the `\r`) still to consume before the lookahead.
            CrTerminal(usize),
            /// Clean EOF.
            Eof,
        }

        let mut line: Vec<u8> = Vec::new();

        loop {
            // Inspect + copy phase. The immutable borrow of `self.inner`
            // ends at the block boundary (NLL); `line` is disjoint from
            // `self.inner`, so appending here is sound.
            let outcome = {
                let buf = match self.inner.fill_buf() {
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Some(Err(e)),
                    Ok(b) => b,
                };
                if buf.is_empty() {
                    Outcome::Eof
                } else {
                    match buf.iter().position(|&b| b == b'\n' || b == b'\r') {
                        None => {
                            line.extend_from_slice(buf);
                            Outcome::Consume(buf.len())
                        }
                        Some(pos) if buf[pos] == b'\n' => {
                            line.extend_from_slice(&buf[..pos]);
                            Outcome::Emit(pos + 1, Some(LineEnding::LF))
                        }
                        Some(pos) if pos + 1 < buf.len() => {
                            line.extend_from_slice(&buf[..pos]);
                            if buf[pos + 1] == b'\n' {
                                Outcome::Emit(pos + 2, Some(LineEnding::CRLF))
                            } else {
                                Outcome::Emit(pos + 1, Some(LineEnding::CR))
                            }
                        }
                        Some(pos) => {
                            line.extend_from_slice(&buf[..pos]);
                            Outcome::CrTerminal(pos + 1)
                        }
                    }
                }
            };

            match outcome {
                Outcome::Consume(n) => {
                    self.inner.consume(n);
                }
                Outcome::Emit(n, ending) => {
                    self.inner.consume(n);
                    return Some(decode_line(line, ending));
                }
                Outcome::CrTerminal(n) => {
                    // Consume through the terminal `\r`, then look ahead at
                    // the next chunk. Pending-CR lives in control flow;
                    // `Interrupted` retries must not disturb it.
                    self.inner.consume(n);
                    let is_lf = loop {
                        match self.inner.fill_buf() {
                            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                            Err(e) => return Some(Err(e)),
                            Ok(buf) => break buf.first().copied() == Some(b'\n'),
                        }
                    };
                    if is_lf {
                        self.inner.consume(1);
                        return Some(decode_line(line, Some(LineEnding::CRLF)));
                    }
                    return Some(decode_line(line, Some(LineEnding::CR)));
                }
                Outcome::Eof => {
                    self.done = true;
                    if line.is_empty() {
                        return None;
                    }
                    return Some(decode_line(line, None));
                }
            }
        }
    }
}

/// Decodes an accumulated line as UTF-8, mapping failures to
/// [`io::ErrorKind::InvalidData`] (mirroring [`std::io::Lines`).
fn decode_line(
    line: Vec<u8>,
    ending: Option<LineEnding>,
) -> io::Result<(String, Option<LineEnding>)> {
    match String::from_utf8(line) {
        Ok(text) => Ok((text, ending)),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}
