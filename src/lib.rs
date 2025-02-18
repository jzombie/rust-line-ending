/// Enum representing the detected line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LineEnding {
    /// Line Feed (LF) - Common on Unix, Linux, and macOS (`\n`).
    LF,
    /// Carriage Return + Line Feed (CRLF) - Used on Windows (`\r\n`).
    CRLF,
    /// Carriage Return (CR) - Used in older Mac OS (pre-OS X) (`\r`).
    CR,
}

impl From<&str> for LineEnding {
    /// Detects the predominant line ending style used in the input string.
    ///
    /// Note: This assumes that the input string is not of varying types, in
    /// which case there is really
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let sample = "first line\r\nsecond line\r\nthird line";
    /// assert_eq!(LineEnding::from(sample), LineEnding::CRLF);
    /// ```
    fn from(s: &str) -> Self {
        let crlf_score = Self::CRLF.split_as(s).len();
        let cr_score = Self::CR.split_as(s).len();
        let lf_score = Self::LF.split_as(s).len();

        // Return the most frequent line ending
        if crlf_score >= cr_score && crlf_score >= lf_score {
            Self::CRLF
        } else if cr_score >= lf_score {
            Self::CR
        } else {
            Self::LF
        }
    }
}

impl LineEnding {
    /// Returns the string representation of the line ending (`\n`, `\r\n`, or `\r`).
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// assert_eq!(LineEnding::LF.as_str(), "\n");
    /// assert_eq!(LineEnding::CRLF.as_str(), "\r\n");
    /// assert_eq!(LineEnding::CR.as_str(), "\r");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LF => "\n",
            Self::CRLF => "\r\n",
            Self::CR => "\r",
        }
    }

    /// Converts all line endings in a string to LF (`\n`) for consistent processing.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let mixed = "first\r\nsecond\rthird\n";
    /// assert_eq!(LineEnding::normalize(mixed), "first\nsecond\nthird\n");
    /// ```
    pub fn normalize(s: &str) -> String {
        s.replace("\r\n", "\n").replace("\r", "\n")
    }

    /// Restores line endings in a string to the specified type.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let normalized = "first\nsecond\nthird";
    /// assert_eq!(LineEnding::CRLF.denormalize(normalized), "first\r\nsecond\r\nthird");
    /// assert_eq!(LineEnding::CR.denormalize(normalized), "first\rsecond\rthird");
    /// ```
    pub fn denormalize(&self, s: &str) -> String {
        s.replace("\n", self.as_str())
    }

    /// Splits a string into lines using the detected line ending.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let text = "line1\r\nline2\r\nline3";
    /// let lines = LineEnding::split(text);
    /// assert_eq!(lines, vec!["line1", "line2", "line3"]);
    /// ```
    pub fn split(s: &str) -> Vec<String> {
        let line_ending = Self::from(s).as_str();
        s.split(line_ending).map(String::from).collect()
    }

    // TODO: Document
    pub fn split_as(&self, s: &str) -> Vec<String> {
        s.split(self.as_str()).map(String::from).collect()
    }

    /// Joins a vector of strings with the specified line ending.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let lines = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
    /// assert_eq!(LineEnding::CRLF.join(lines.clone()), "line1\r\nline2\r\nline3");
    /// assert_eq!(LineEnding::LF.join(lines.clone()), "line1\nline2\nline3");
    /// ```
    pub fn join(&self, lines: Vec<String>) -> String {
        lines.join(self.as_str())
    }

    /// Converts a string from any line ending type to a specified one.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let mixed_text = "first line\r\nsecond line\rthird line\n";
    /// assert_eq!(LineEnding::CRLF.apply(mixed_text), "first line\r\nsecond line\r\nthird line\r\n");
    /// assert_eq!(LineEnding::LF.apply(mixed_text), "first line\nsecond line\nthird line\n");
    /// ```
    pub fn apply(&self, s: &str) -> String {
        let normalized = Self::normalize(s);
        normalized.replace("\n", self.as_str())
    }
}

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

    // TODO: Update to handle multi-line scoring
    // #[test]
    // fn ignores_escaped_line_endings_in_detection() {
    //     // Escaped line endings should NOT be detected as actual newlines
    //     // assert_eq!(LineEnding::from("This is a test\\nstring"), LineEnding::LF);
    //     // assert_eq!(
    //     //     LineEnding::from("This is a test\\r\\nstring"),
    //     //     LineEnding::LF
    //     // );
    //     // assert_eq!(LineEnding::from("This is a test\\rstring"), LineEnding::LF);

    //     // // Actual line endings should be detected correctly
    //     // assert_eq!(LineEnding::from("This is a test\nstring"), LineEnding::LF);
    //     // assert_eq!(
    //     //     LineEnding::from("This is a test\r\nstring"),
    //     //     LineEnding::CRLF
    //     // );
    //     // assert_eq!(LineEnding::from("This is a test\rstring"), LineEnding::CR);
    // }

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
}
