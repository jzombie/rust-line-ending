#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod line_ending;
pub use line_ending::LineEnding;
#[cfg(feature = "std")]
pub use line_ending::LineEndingScores;

#[cfg(feature = "std")]
pub mod bufread_ext;
#[cfg(feature = "std")]
pub use bufread_ext::{BufReadLineEndingExt, LinesStreaming};

pub mod peekable_ext;
pub use peekable_ext::PeekableLineEndingExt;
