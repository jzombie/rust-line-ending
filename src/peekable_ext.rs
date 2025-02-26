use crate::LineEnding;
use std::iter::Peekable;

/// A trait to extend `Peekable<char>` with a method for consuming line endings.
pub trait PeekableLineEndingExt {
    /// Consumes the line ending from the iterator if the upcoming characters
    /// form a line break (CR, LF, or CRLF), and returns its type.
    /// Otherwise, returns `None`.
    ///
    /// **Note:** Escaped sequences (`\\r`, `\\n`, `\\r\\n`) are treated as part of the content
    /// and are **not consumed** as actual line endings.
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
    ///   - If the next character is `'\\'`, it checks if the following character is `'r'` or `'n'`.
    ///     - If so, it is **an escaped sequence** (`\\r`, `\\n`, or `\\r\\n`) and is treated as **part of the content**.
    ///     - It is **not** consumed as an actual line ending.
    ///   - If the next character is `'\r'`, it clones the iterator and checks the following character.
    ///     - If that character is `'\n'`, it consumes both characters (interpreting them
    ///       as a CRLF sequence) and returns `Some(LineEnding::CRLF)`.
    ///     - If the next character is **not** `'\n'`, it consumes the lone `'\r'` and returns `Some(LineEnding::CR)`.
    ///   - If the next character is `'\n'`, it consumes it and returns `Some(LineEnding::LF)`.
    ///
    /// If no line ending is detected at the front of the iterator, the method returns `None`.
    ///
    /// # Behavior Summary:
    /// - **`\r\n`** → Consumed and returns `Some(LineEnding::CRLF)`
    /// - **`\r`** → Consumed and returns `Some(LineEnding::CR)`
    /// - **`\n`** → Consumed and returns `Some(LineEnding::LF)`
    /// - **`\\r`, `\\n`, `\\r\\n`** → **Not consumed**, treated as normal characters in the content
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
    ///
    /// // Example with escaped line endings:
    /// let mut it: Peekable<Chars> = "\\r\\nline1".chars().peekable();
    /// assert_eq!(it.consume_line_ending(), None); // `\\r\\n` is part of content
    /// assert_eq!(it.peek(), Some(&'\\')); // The iterator remains at '\\'
    /// ```
    fn consume_line_ending(&mut self) -> Option<LineEnding> {
        if let Some(&first) = self.peek() {
            if first == '\\' {
                // Check if the next character is 'r' or 'n' (escaped line endings)
                let mut clone = self.clone();
                clone.next(); // Consume first '\\'

                if let Some(&second) = clone.peek() {
                    if second == 'r' || second == 'n' {
                        // If escaped, do nothing and return None
                        return None;
                    }
                }
            } else if first == '\r' {
                let mut clone = self.clone();
                clone.next(); // Consume '\r'
                if let Some(&second) = clone.peek() {
                    if second == '\n' {
                        self.next(); // Consume '\r'
                        self.next(); // Consume '\n'
                        return Some(LineEnding::CRLF);
                    }
                }
                self.next(); // Consume '\r'
                return Some(LineEnding::CR);
            } else if first == '\n' {
                self.next(); // Consume '\n'
                return Some(LineEnding::LF);
            }
        }
        None
    }
}
