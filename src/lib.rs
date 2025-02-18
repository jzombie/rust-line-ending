/// Enum representing the detected line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LineEnding {
    LF,   // "\n" (Unix, Linux, macOS)
    CRLF, // "\r\n" (Windows)
    CR,   // "\r" (old Mac OS [pre-OS X])
}

impl From<&str> for LineEnding {
    /// Detects the line ending style used in the input string.
    fn from(s: &str) -> Self {
        if s.contains("\r\n") {
            Self::CRLF
        } else if s.contains("\r") {
            Self::CR
        } else {
            Self::LF
        }
    }
}

impl LineEnding {
    /// Returns the string representation of the line ending.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LF => "\n",
            Self::CRLF => "\r\n",
            Self::CR => "\r",
        }
    }

    /// Normalize to `\n` for consistent processing.
    pub fn normalize(s: &str) -> String {
        s.replace("\r\n", "\n").replace("\r", "\n")
    }

    /// Restores line endings back to their original value.
    pub fn denormalize(&self, s: &str) -> String {
        s.replace("\n", self.as_str())
    }

    /// Applies the line endiing to the given lines.
    pub fn apply_to_lines(&self, lines: Vec<String>) -> String {
        lines.join(self.as_str())
    }

    /// Converts a string from any line ending type to a specified line ending.
    pub fn convert_to(&self, s: &str) -> String {
        let normalized = Self::normalize(s); // Convert all line endings to LF first
        normalized.replace("\n", self.as_str()) // Replace LF with the target line ending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_platform_line_ending_correctly() {
        use std::fs::File;
        use std::io::Read;

        let readme_file = "README.md";

        // Read file contents
        let mut read_content = String::new();
        File::open(readme_file)
            .expect(&format!("Failed to open {}", readme_file))
            .read_to_string(&mut read_content)
            .expect(&format!("Failed to read {}", readme_file));

        // Determine line ending from file contents
        let detected = LineEnding::from(read_content.as_str());

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
            LineEnding::CRLF.apply_to_lines(lines.clone()),
            "first\r\nsecond\r\nthird"
        );
        assert_eq!(
            LineEnding::CR.apply_to_lines(lines.clone()),
            "first\rsecond\rthird"
        );
        assert_eq!(
            LineEnding::LF.apply_to_lines(lines.clone()),
            "first\nsecond\nthird"
        );
    }

    #[test]
    fn convert_to_correctly_applies_line_endings() {
        let mixed_text = "first line\r\nsecond line\rthird line\nfourth line\n";

        assert_eq!(
            LineEnding::CRLF.convert_to(mixed_text),
            "first line\r\nsecond line\r\nthird line\r\nfourth line\r\n"
        );
        assert_eq!(
            LineEnding::CR.convert_to(mixed_text),
            "first line\rsecond line\rthird line\rfourth line\r"
        );
        assert_eq!(
            LineEnding::LF.convert_to(mixed_text),
            "first line\nsecond line\nthird line\nfourth line\n"
        );
    }
}
