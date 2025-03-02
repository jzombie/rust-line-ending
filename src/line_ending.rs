use std::collections::HashMap;

/// Enum representing the detected line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum LineEnding {
    /// Line Feed (LF) - Common on Unix, Linux, and macOS (`\n`).
    LF,
    /// Carriage Return + Line Feed (CRLF) - Used on Windows (`\r\n`).
    CRLF,
    /// Carriage Return (CR) - Used in older Mac OS (pre-OS X) (`\r`).
    CR,
}

/// A mapping of line ending types to their respective occurrence counts.
///
/// This type alias represents a `HashMap<LineEnding, usize>`, where each
/// `LineEnding` key corresponds to the number of times that specific
/// line ending appears in a given string.
///
/// This is used in functions like [`LineEnding::score_mixed_types`] to track
/// the distribution of line endings in a text.
pub type LineEndingScores = HashMap<LineEnding, usize>;

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
        let scores = Self::score_mixed_types(s);

        let crlf_score = *scores.get(&Self::CRLF).unwrap_or(&0);
        let cr_score = *scores.get(&Self::CR).unwrap_or(&0);
        let lf_score = *scores.get(&Self::LF).unwrap_or(&0);

        // Select the highest count
        let max_score = crlf_score.max(cr_score).max(lf_score);

        if max_score == 0 || crlf_score == max_score {
            // `CRLF` is chosen as a tie-breaker because it represents both `CR`
            // and `LF`, making it the most inclusive option
            Self::CRLF
        } else if cr_score == max_score {
            Self::CR
        } else {
            Self::LF
        }
    }
}

impl LineEnding {
    /// Detects the default line ending based on the current operating system.
    ///
    /// - **Unix-based (Linux/macOS):** Uses LF (`\n`).
    /// - **Windows:** Uses CRLF (`\r\n`).
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let default_ending = LineEnding::from_current_platform();
    /// println!("Default line ending: {:?}", default_ending);
    /// ```
    pub fn from_current_platform() -> Self {
        if cfg!(windows) {
            Self::CRLF
        } else {
            Self::LF
        }
    }

    /// Counts occurrences of each line ending type in the given string.
    ///
    /// This function analyzes the input string and returns a `LineEndingScores`
    /// (a `HashMap<LineEnding, usize>`) containing the number of times each
    /// line ending appears.
    ///
    /// - `CRLF (\r\n)` is counted first to ensure `\r` inside `\r\n` is not
    ///   double-counted.
    /// - `CR (\r)` is counted separately, subtracting occurrences of `CRLF`.
    /// - `LF (\n)` is counted separately, also subtracting occurrences of `CRLF`.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::{LineEnding, LineEndingScores};
    ///
    /// let text = "line1\r\nline2\r\nline3\nline4\r";
    /// let scores = LineEnding::score_mixed_types(text);
    ///
    /// assert_eq!(scores[&LineEnding::CRLF], 2);
    /// assert_eq!(scores[&LineEnding::LF], 1);
    /// assert_eq!(scores[&LineEnding::CR], 1);
    /// ```
    pub fn score_mixed_types(s: &str) -> LineEndingScores {
        let crlf_score = Self::CRLF.split_with(s).len().saturating_sub(1);

        // Ensure CR is not double-counted when it's part of CRLF
        let cr_score = Self::CR.split_with(s).len().saturating_sub(1) - crlf_score;

        // Ensure LF is not double-counted when it's part of CRLF
        let lf_score = Self::LF.split_with(s).len().saturating_sub(1) - crlf_score;

        [
            (LineEnding::CRLF, crlf_score),
            (LineEnding::CR, cr_score),
            (LineEnding::LF, lf_score),
        ]
        .into_iter()
        .collect()
    }

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

    /// Returns the character representation of the line ending if it is a single character.
    ///
    /// # Panics
    ///
    /// Panics if the line ending is CRLF, because CRLF is composed of two characters
    /// and cannot be represented as a single character.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// assert_eq!(LineEnding::LF.as_char(), '\n');
    /// assert_eq!(LineEnding::CR.as_char(), '\r');
    /// // The following call will panic:
    /// // LineEnding::CRLF.as_char();
    /// ```
    pub fn as_char(&self) -> char {
        match self {
            Self::LF => '\n',
            Self::CR => '\r',
            Self::CRLF => panic!("CRLF cannot be represented as a single character"),
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

    /// Splits a string into a vector of strings using the auto-detected line ending
    /// parsed from the string.
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

    /// Splits a string into lines using the specified line ending.
    ///
    /// In most cases, `split` is the preferred method as it automatically detects the
    /// line ending to use.
    ///
    /// Unlike [`LineEnding::split`], which detects the line ending type from the input,
    /// this method explicitly uses the line ending type of `self` to split the string.
    ///
    /// # Example
    ///
    /// ```
    /// use line_ending::LineEnding;
    ///
    /// let text = "line1\r\nline2\r\nline3";
    /// let lines = LineEnding::CRLF.split_with(text);
    /// assert_eq!(lines, vec!["line1", "line2", "line3"]);
    ///
    /// let text = "line1\nline2\nline3";
    /// let lines = LineEnding::LF.split_with(text);
    /// assert_eq!(lines, vec!["line1", "line2", "line3"]);
    /// ```
    pub fn split_with(&self, s: &str) -> Vec<String> {
        s.split(self.as_str()).map(String::from).collect()
    }

    /// Joins a vector of strings using the specified line ending.
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

    /// Applies a specific line ending type to an existing string.
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
