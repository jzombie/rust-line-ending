#[cfg(test)]
mod tests {
    use super::*;

    fn get_readme_contents() -> String {
        use std::fs::File;
        use std::io::Read;

        let readme_file = "README.md";

        // Read file contents
        let mut read_content = String::new();
        File::open(readme_file)
            .unwrap_or_else(|_| panic!("Failed to open {}", readme_file))
            .read_to_string(&mut read_content)
            .unwrap_or_else(|_| panic!("Failed to read {}", readme_file));

        read_content
    }

    #[test]
    fn detects_platform_line_ending_correctly() {
        // Determine line ending from file contents
        let detected = LineEnding::from(get_readme_contents().as_str());

        // Assert expected line ending based on platform
        #[cfg(target_os = "windows")]
        assert_eq!(detected, LineEnding::CRLF, "Windows should detect CRLF");

        #[cfg(target_family = "unix")]
        assert_eq!(detected, LineEnding::LF, "Unix/macOS should detect LF");
    }

    #[test]
    fn detects_lf_correctly() {
        let sample = "first line\nsecond line\nthird line";
        assert_eq!(LineEnding::from(sample), LineEnding::LF);
    }

    #[test]
    fn detects_crlf_correctly() {
        let sample = "first line\r\nsecond line\r\nthird line";
        assert_eq!(LineEnding::from(sample), LineEnding::CRLF);
    }

    #[test]
    fn detects_cr_correctly() {
        let sample = "first line\rsecond line\rthird line";
        assert_eq!(LineEnding::from(sample), LineEnding::CR);
    }

    #[test]
    fn normalize_converts_all_to_lf() {
        let crlf = "first\r\nsecond\r\nthird";
        let cr = "first\rsecond\rthird";
        let lf = "first\nsecond\nthird";

        assert_eq!(LineEnding::normalize(crlf), lf);
        assert_eq!(LineEnding::normalize(cr), lf);
        assert_eq!(LineEnding::normalize(lf), lf);
    }

    #[test]
    fn splits_into_lines() {
        let readme_contents = get_readme_contents();
        let readme_lines = LineEnding::split(&readme_contents);

        assert_eq!(readme_lines.first().unwrap(), "# Rust Line Endings");

        let crlf_lines = LineEnding::split("first\r\nsecond\r\nthird");
        let cr_lines = LineEnding::split("first\rsecond\rthird");
        let lf_lines = LineEnding::split("first\nsecond\nthird");

        let expected = vec!["first", "second", "third"];

        assert_eq!(crlf_lines, expected);
        assert_eq!(cr_lines, expected);
        assert_eq!(lf_lines, expected);
    }

    #[test]
    fn restore_correctly_applies_line_endings() {
        let text = "first\nsecond\nthird";
        let crlf_restored = LineEnding::CRLF.denormalize(text);
        let cr_restored = LineEnding::CR.denormalize(text);
        let lf_restored = LineEnding::LF.denormalize(text);

        assert_eq!(crlf_restored, "first\r\nsecond\r\nthird");
        assert_eq!(cr_restored, "first\rsecond\rthird");
        assert_eq!(lf_restored, "first\nsecond\nthird");
    }

    #[test]
    fn applies_correct_line_endings() {
        let lines = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];

        assert_eq!(
            LineEnding::CRLF.join(lines.clone()),
            "first\r\nsecond\r\nthird"
        );
        assert_eq!(LineEnding::CR.join(lines.clone()), "first\rsecond\rthird");
        assert_eq!(LineEnding::LF.join(lines.clone()), "first\nsecond\nthird");
    }

    #[test]
    fn apply_correctly_applies_line_endings() {
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
    }

    #[test]
    fn handles_mixed_line_endings() {
        // Mixed with some CRLF and CR, but LF is dominant
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

        // Mixed with some LF and CR, but CRLF is dominant
        let mostly_crlf = "line1\r\nline2\r\nline3\nline4\rline5\r\nline6\r\n";
        assert_eq!(LineEnding::from(mostly_crlf), LineEnding::CRLF);
        assert_eq!(
            LineEnding::score_mixed_types(mostly_crlf,),
            [
                (LineEnding::CRLF, 4),
                (LineEnding::CR, 1),
                (LineEnding::LF, 1),
            ]
            .into_iter()
            .collect::<LineEndingScores>()
        );

        // Mixed with some LF and CRLF, but CR is dominant
        let mostly_cr = "line1\rline2\r\nline3\rline4\nline5\rline6\r";
        assert_eq!(LineEnding::from(mostly_cr), LineEnding::CR);
        assert_eq!(
            LineEnding::score_mixed_types(mostly_cr,),
            [
                (LineEnding::CRLF, 1),
                (LineEnding::CR, 4),
                (LineEnding::LF, 1),
            ]
            .into_iter()
            .collect::<LineEndingScores>()
        );
    }

    #[test]
    fn handles_mixed_line_edge_cases() {
        // Case 1: One Line Ending Type is Clearly Dominant
        let mostly_crlf = "line1\r\nline2\r\nline3\nline4\r\nline5\r\n";
        assert_eq!(LineEnding::from(mostly_crlf), LineEnding::CRLF); // CRLF is the most common

        // Case 2: All Line Endings Appear Equally
        let equal_mixed = "line1\r\nline2\nline3\rline4\r\nline5\nline6\r";
        assert_eq!(LineEnding::from(equal_mixed), LineEnding::CRLF); // CRLF > CR > LF

        // Case 3: Single Line Containing Multiple Line Endings
        let mixed_on_one_line = "line1\r\nline2\rline3\r\nline4\r\nline5\r";
        assert_eq!(LineEnding::from(mixed_on_one_line), LineEnding::CRLF); // CRLF appears the most overall

        // Case 4: Empty Input Defaults to CRLF
        let empty_text = "";
        assert_eq!(LineEnding::from(empty_text), LineEnding::CRLF); // Defaults to CRLF
    }

    #[test]
    fn ignores_escaped_line_endings_in_split() {
        let input_lf = "First\\nSecond\\nThird";
        let input_crlf = "First\\r\\nSecond\\r\\nThird";
        let input_cr = "First\\rSecond\\rThird";

        // Expected output: The input should NOT be split since these are escaped sequences
        assert_eq!(LineEnding::split(input_lf), vec!["First\\nSecond\\nThird"]);
        assert_eq!(
            LineEnding::split(input_crlf),
            vec!["First\\r\\nSecond\\r\\nThird"]
        );
        assert_eq!(LineEnding::split(input_cr), vec!["First\\rSecond\\rThird"]);
    }

    #[test]
    fn split_does_not_split_on_escaped_line_endings() {
        let input_lf = "First\\nSecond\\nThird";
        let input_crlf = "First\\r\\nSecond\\r\\nThird";
        let input_cr = "First\\rSecond\\rThird";

        // All inputs should remain as a single, unsplit string
        assert_eq!(LineEnding::split(input_lf), vec!["First\\nSecond\\nThird"]);
        assert_eq!(
            LineEnding::split(input_crlf),
            vec!["First\\r\\nSecond\\r\\nThird"]
        );
        assert_eq!(LineEnding::split(input_cr), vec!["First\\rSecond\\rThird"]);
    }

    #[test]
    fn split_correctly_splits_on_actual_line_endings() {
        let input_lf = "First\nSecond\nThird";
        let input_crlf = "First\r\nSecond\r\nThird";
        let input_cr = "First\rSecond\rThird";

        // Each input should split correctly based on its actual line endings
        assert_eq!(
            LineEnding::split(input_lf),
            vec!["First", "Second", "Third"]
        );
        assert_eq!(
            LineEnding::split(input_crlf),
            vec!["First", "Second", "Third"]
        );
        assert_eq!(
            LineEnding::split(input_cr),
            vec!["First", "Second", "Third"]
        );
    }

    #[test]
    fn split_detects_mixed_escaped_and_actual_line_endings() {
        // LF test case (escaped `\\n` should not trigger a split, actual `\n` should)
        let input_lf = "First\\nSecond\nThird";
        assert_eq!(LineEnding::split(input_lf), vec!["First\\nSecond", "Third"]);

        // CRLF test case (escaped `\\r\\n` should be ignored, actual `\r\n` should split)
        let input_crlf = "First\\r\\nSecond\r\nThird";
        assert_eq!(
            LineEnding::split(input_crlf),
            vec!["First\\r\\nSecond", "Third"]
        );

        // CR test case (escaped `\\r` should be ignored, actual `\r` should split)
        let input_cr = "First\\rSecond\rThird";
        assert_eq!(LineEnding::split(input_cr), vec!["First\\rSecond", "Third"]);
    }

    #[test]
    fn test_as_char_returns_single_char_for_lf_and_cr() {
        // LF should return '\n'
        assert_eq!(LineEnding::LF.as_char(), '\n');
        // CR should return '\r'
        assert_eq!(LineEnding::CR.as_char(), '\r');
    }

    #[test]
    #[should_panic(expected = "CRLF cannot be represented as a single character")]
    fn test_as_char_panics_for_crlf() {
        // CRLF is composed of two characters, so this should panic.
        let _ = LineEnding::CRLF.as_char();
    }
}
