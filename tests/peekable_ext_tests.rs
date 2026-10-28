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

    fn drain(s: &str) -> (Vec<String>, Vec<LineEnding>) {
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
                current_line.push(it.next().unwrap());
            }
        }
        lines.push(current_line);
        (lines, consumed)
    }

    #[test]
    fn trailing_cr_at_true_eof_is_single_cr() {
        let (lines, consumed) = drain("abc\r");
        assert_eq!(consumed, vec![LineEnding::CR]);
        assert_eq!(lines, vec!["abc", ""]);
    }

    #[test]
    fn empty_input_yields_no_break() {
        let mut it = "".chars().peekable();
        assert_eq!(it.consume_line_ending(), None);
        let (lines, consumed) = drain("");
        assert!(consumed.is_empty());
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn letters_r_and_n_never_split() {
        // Literal 'r'/'n' letters (and backslash + letter, which the
        // char API treats as escaped content) never form breaks;
        // only 0x0D/0x0A do.
        let (lines, consumed) = drain("learn\niron\r\nurn\rern\nrun\\ner");
        assert_eq!(
            consumed,
            vec![
                LineEnding::LF,
                LineEnding::CRLF,
                LineEnding::CR,
                LineEnding::LF
            ]
        );
        assert_eq!(lines, vec!["learn", "iron", "urn", "ern", "run\\ner"]);
    }

    #[test]
    fn lone_breaks_are_detected() {
        assert_eq!(drain("\n").1, vec![LineEnding::LF]);
        assert_eq!(drain("\r").1, vec![LineEnding::CR]);
        assert_eq!(drain("\r\n").1, vec![LineEnding::CRLF]);
    }

    #[test]
    fn consecutive_breaks_yield_empty_lines() {
        let (lines, consumed) = drain("a\n\nb\r\n\r\nc");
        assert_eq!(
            consumed,
            vec![
                LineEnding::LF,
                LineEnding::LF,
                LineEnding::CRLF,
                LineEnding::CRLF
            ]
        );
        assert_eq!(lines, vec!["a", "", "b", "", "c"]);
    }

    #[test]
    fn split_iterators_do_not_stitch_crlf() {
        // Negative control (SEV-1): `None` is EOF, not end-of-chunk. Two
        // separate iterators MUST yield CR then LF — cross-`None` stitching
        // is out of scope by `Iterator` contract.
        let mut first = "abc\r".chars().peekable();
        let mut second = "\nxyz".chars().peekable();
        // A leading non-break yields `None` without consuming.
        assert_eq!(first.consume_line_ending(), None);
        // Drain content, then observe the trailing break.
        assert_eq!(first.next(), Some('a'));
        assert_eq!(first.next(), Some('b'));
        assert_eq!(first.next(), Some('c'));
        assert_eq!(first.consume_line_ending(), Some(LineEnding::CR));
        assert_eq!(second.consume_line_ending(), Some(LineEnding::LF));
    }
}
