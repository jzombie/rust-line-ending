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

        // Expect to detect, in order: CRLF, LF, and CR.
        let expected_line_endings = vec![LineEnding::CRLF, LineEnding::LF, LineEnding::CR];
        assert_eq!(consumed, expected_line_endings);

        // Expect to collect the lines: "line1", "line2", "line3", and "line4".
        let expected_lines = vec!["line1", "line2", "line3", "line4"];
        assert_eq!(lines, expected_lines);
    }
}
