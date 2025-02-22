use crate::LineEnding;
use std::iter::Peekable;

/// A trait to extend `Peekable<char>` with a method for consuming line endings.
pub trait PeekableLineEndingExt {
    /// Consumes the line ending from the iterator if the upcoming characters
    /// form a line break (CR, LF, or CRLF), and returns its type.
    /// Otherwise, returns None.
    fn consume_line_ending(&mut self) -> Option<LineEnding>;
}

impl<I> PeekableLineEndingExt for Peekable<I>
where
    I: Iterator<Item = char> + Clone,
{
    /// Consumes the line ending from the iterator if the upcoming characters form a line break,
    /// and returns its corresponding `LineEnding` variant.
    ///
    /// This method works as follows:
    /// - It peeks at the next character:
    ///   - If the next character is `'\r'`, it clones the iterator and checks the following
    ///     character. If that character is `'\n'`, it consumes both characters (interpreting them
    ///     as a CRLF sequence) and returns `Some(LineEnding::CRLF)`.
    ///   - If the next character is `'\r'` but is not followed by `'\n'`, it consumes the lone
    ///     `'\r'` and returns `Some(LineEnding::CR)`.
    /// - If the next character is `'\n'`, it consumes it and returns `Some(LineEnding::LF)`.
    ///
    /// If no line ending is detected at the front of the iterator, the method returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::iter::Peekable;
    /// use std::str::Chars;
    /// use line_ending::{LineEnding, PeekableLineEndingExt};
    ///
    /// // Example with CRLF:
    /// let mut it: Peekable<Chars> = "\r\n".chars().peekable();
    /// assert_eq!(it.consume_line_ending(), Some(LineEnding::CRLF));
    ///
    /// // Example with LF:
    /// let mut it: Peekable<Chars> = "\n".chars().peekable();
    /// assert_eq!(it.consume_line_ending(), Some(LineEnding::LF));
    ///
    /// // Example with CR:
    /// let mut it: Peekable<Chars> = "\r".chars().peekable();
    /// assert_eq!(it.consume_line_ending(), Some(LineEnding::CR));
    ///
    /// // Example with no line ending:
    /// let mut it: Peekable<Chars> = "hello".chars().peekable();
    /// assert_eq!(it.consume_line_ending(), None);
    /// ```
    fn consume_line_ending(&mut self) -> Option<LineEnding> {
        if let Some(&first) = self.peek() {
            if first == '\r' {
                let mut clone = self.clone();
                clone.next(); // skip the '\r' in the clone
                if let Some(&second) = clone.peek() {
                    if second == '\n' {
                        // Consume both for CRLF.
                        self.next();
                        self.next();
                        return Some(LineEnding::CRLF);
                    }
                }
                // Otherwise, consume lone CR.
                self.next();
                return Some(LineEnding::CR);
            } else if first == '\n' {
                self.next();
                return Some(LineEnding::LF);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_line_endings() {
        // This string has:
        // - "line1" ending with CRLF,
        // - "line2" ending with LF,
        // - "line3" ending with CR,
        // followed by "line4" (with no trailing line break).
        let s = "line1\r\nline2\nline3\rline4";
        let mut it = s.chars().peekable();
        let mut consumed = Vec::new();
        let mut current_line = String::new();
        let mut lines = Vec::new();

        // Iterate over the stream. If a line ending is consumed,
        // push the current line into `lines` and reset it.
        while it.peek().is_some() {
            if let Some(le) = it.consume_line_ending() {
                consumed.push(le);
                lines.push(current_line);
                current_line = String::new();
            } else {
                // Not a line break; append the character to the current line.
                current_line.push(it.next().unwrap());
            }
        }
        // Push the final line (which may be non-empty).
        lines.push(current_line);

        // Expect to detect, in order: CRLF, LF, and CR.
        let expected_line_endings = vec![LineEnding::CRLF, LineEnding::LF, LineEnding::CR];
        assert_eq!(consumed, expected_line_endings);

        // Expect to collect the lines: "line1", "line2", "line3", and "line4".
        let expected_lines = vec!["line1", "line2", "line3", "line4"];
        assert_eq!(lines, expected_lines);
    }
}
