#[cfg(test)]
mod tests {
    use line_ending::{LineEnding, PeekableLineEndingExt};

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

        // Expect to detect, in order: CRLF, LF, and CR
        let expected_line_endings = vec![LineEnding::CRLF, LineEnding::LF, LineEnding::CR];
        assert_eq!(consumed, expected_line_endings);

        // Expect to collect the lines: "line1", "line2", "line3", and "line4"
        let expected_lines = vec!["line1", "line2", "line3", "line4"];
        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn test_consume_line_endings_with_escape_sequences() {
        let s = "\\r\\nline1\\n\nline2\\r\rline3\\r\\n\r\nline4\\n";
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
        let expected_line_endings = vec![LineEnding::LF, LineEnding::CR, LineEnding::CRLF];
        assert_eq!(consumed, expected_line_endings);

        // Escaped line ending sequences are part of the content
        let expected_lines = vec!["\\r\\nline1\\n", "line2\\r", "line3\\r\\n", "line4\\n"];
        assert_eq!(lines, expected_lines);
    }

    #[test]
    fn test_consume_nested_line_endings_with_escape_sequences() {
        let s = "\\\\nline1\nline2\\\\r\\\\n\r\nline3\\\\r\rline4\r\nline5\rline6\\\\r";
        let mut it = s.chars().peekable();
        let mut consumed = Vec::new();
        let mut current_line = String::new();
        let mut lines = Vec::new();

        while it.peek().is_some() {
            if let Some(le) = it.consume_line_ending() {
                consumed.push(le);
                lines.push(current_line);
                current_line = String::new();
            } else {
                let next_char = it.next().unwrap();
                current_line.push(next_char);
            }
        }

        lines.push(current_line);

        assert_eq!(
            consumed,
            vec![
                LineEnding::LF,
                LineEnding::CRLF,
                LineEnding::CR,
                LineEnding::CRLF,
                LineEnding::CR
            ]
        );

        // Should preserve the escaped newlines
        assert_eq!(
            lines,
            vec![
                "\\\\nline1",
                "line2\\\\r\\\\n",
                "line3\\\\r",
                "line4",
                "line5",
                "line6\\\\r"
            ]
        );
    }
}
