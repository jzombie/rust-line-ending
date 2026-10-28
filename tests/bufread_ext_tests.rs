#[cfg(all(test, feature = "std"))]
mod tests {
    use line_ending::{BufReadLineEndingExt, LineEnding, LinesStreaming};
    use std::collections::VecDeque;
    use std::io::{self, BufRead, BufReader, Read};

    fn collect_all<R: BufRead>(reader: R) -> Vec<io::Result<(String, Option<LineEnding>)>> {
        LinesStreaming::new(reader).collect()
    }

    fn unwrap_all(
        items: Vec<io::Result<(String, Option<LineEnding>)>>,
    ) -> Vec<(String, Option<LineEnding>)> {
        items
            .into_iter()
            .map(|r| r.expect("streaming item should succeed"))
            .collect()
    }

    #[test]
    fn streams_mixed_endings() {
        let reader = BufReader::new(b"line1\r\nline2\nline3\rline4" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("line1".to_string(), Some(LineEnding::CRLF)),
                ("line2".to_string(), Some(LineEnding::LF)),
                ("line3".to_string(), Some(LineEnding::CR)),
                ("line4".to_string(), None),
            ]
        );
    }

    #[test]
    fn consume_line_ending_steps_through_mixed_stream() {
        let mut reader = BufReader::new(b"a\r\nb\nc\rd" as &[u8]);
        let mut text = String::new();
        let mut endings = Vec::new();
        let mut buf = [0u8; 1];
        while !reader.fill_buf().expect("fill").is_empty() {
            if let Some(le) = reader.consume_line_ending().expect("consume") {
                endings.push(le);
            } else {
                reader.read_exact(&mut buf).expect("read byte");
                text.push(buf[0] as char);
            }
        }
        assert_eq!(text, "abcd");
        assert_eq!(
            endings,
            vec![LineEnding::CRLF, LineEnding::LF, LineEnding::CR]
        );
    }

    #[test]
    fn capacity_boundary_splits_crlf() {
        // `ab\r` is exactly 3 bytes: with capacity 3 the first `fill_buf`
        // returns `[a, b, \r]`, placing `\r` at the physical buffer end and
        // forcing the pending-CR -> re-`fill_buf` -> peek-`\n` path.
        // (Capacity 5 would ingest `[a, b, \r, \n, c]` and never test this.)
        let reader = BufReader::with_capacity(3, b"ab\r\ncd" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("ab".to_string(), Some(LineEnding::CRLF)),
                ("cd".to_string(), None),
            ]
        );
    }

    #[test]
    fn capacity_boundary_lone_cr_at_eof() {
        // Same boundary shape, but EOF follows the `\r`: must emit CR, not
        // hang waiting for a lookahead byte that never arrives.
        let reader = BufReader::with_capacity(3, b"ab\rcd" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("ab".to_string(), Some(LineEnding::CR)),
                ("cd".to_string(), None),
            ]
        );
    }

    #[test]
    fn one_byte_chunks_still_split_correctly() {
        // Extreme chunking: every byte arrives alone. `BufReader` coalesces
        // only up to its capacity, so use capacity 1 to force per-byte views.
        let reader = BufReader::with_capacity(1, b"a\r\nb\rc\nd" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("a".to_string(), Some(LineEnding::CRLF)),
                ("b".to_string(), Some(LineEnding::CR)),
                ("c".to_string(), Some(LineEnding::LF)),
                ("d".to_string(), None),
            ]
        );
    }

    #[test]
    fn empty_stream_yields_nothing() {
        let reader = BufReader::new(b"" as &[u8]);
        assert!(LinesStreaming::new(reader).next().is_none());
    }

    #[test]
    fn only_breaks_yield_empty_lines() {
        let reader = BufReader::new(b"\n\r\n\r" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("".to_string(), Some(LineEnding::LF)),
                ("".to_string(), Some(LineEnding::CRLF)),
                ("".to_string(), Some(LineEnding::CR)),
            ]
        );
    }

    #[test]
    fn backslash_is_ordinary_content_in_wire_mode() {
        // Wire mode: no escape handling. A literal backslash never suppresses
        // a real break, and `\\r\\n` literal bytes split as CR + LF.
        let reader = BufReader::new(b"a\\b\r\nc" as &[u8]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("a\\b".to_string(), Some(LineEnding::CRLF)),
                ("c".to_string(), None),
            ]
        );
    }

    #[test]
    fn non_interrupted_io_errors_surface_as_err() {
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "injected failure",
                ))
            }
        }
        impl BufRead for Failing {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "injected failure",
                ))
            }
            fn consume(&mut self, _amt: usize) {}
        }

        let mut it = LinesStreaming::new(Failing);
        match it.next() {
            Some(Err(e)) => assert_eq!(e.kind(), io::ErrorKind::ConnectionReset),
            other => panic!("expected an I/O error, got {:?}", other.map(|r| r.is_ok())),
        }
    }

    #[test]
    fn invalid_utf8_surfaces_as_invalid_data() {
        let reader = BufReader::new(b"\xff\xfe\nok\n" as &[u8]);
        let mut it = LinesStreaming::new(reader);
        match it.next() {
            Some(Err(e)) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
            other => panic!("expected InvalidData, got {:?}", other.map(|r| r.is_ok())),
        }
        // Stream continues after the error (mirrors `std::io::Lines`).
        assert_eq!(
            it.next()
                .expect("stream should continue")
                .expect("line should decode"),
            ("ok".to_string(), Some(LineEnding::LF))
        );
    }

    /// Scripted `BufRead` where each `fill_buf` view is programmed: data
    /// chunks, injected `Interrupted` faults, then EOF.
    struct ScriptedBufRead {
        script: VecDeque<ScriptAction>,
        current: Vec<u8>,
        pos: usize,
    }

    enum ScriptAction {
        Data(Vec<u8>),
        Interrupted,
    }

    impl ScriptedBufRead {
        fn new(script: Vec<ScriptAction>) -> Self {
            Self {
                script: script.into(),
                current: Vec::new(),
                pos: 0,
            }
        }
    }

    impl Read for ScriptedBufRead {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let buf = self.fill_buf()?;
            let n = buf.len().min(out.len());
            out[..n].copy_from_slice(&buf[..n]);
            self.consume(n);
            Ok(n)
        }
    }

    impl BufRead for ScriptedBufRead {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if self.pos >= self.current.len() {
                match self.script.pop_front() {
                    None => return Ok(&[]),
                    Some(ScriptAction::Data(chunk)) => {
                        self.current = chunk;
                        self.pos = 0;
                    }
                    Some(ScriptAction::Interrupted) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "injected interrupt",
                        ));
                    }
                }
            }
            Ok(&self.current[self.pos..])
        }

        fn consume(&mut self, amt: usize) {
            self.pos = (self.pos + amt).min(self.current.len());
        }
    }

    #[test]
    fn interrupted_during_lookahead_still_yields_single_crlf() {
        // The `Interrupted` fires exactly on the lookahead `fill_buf` after
        // a `\r`-terminal slice; the retry must preserve pending-CR and
        // emit one CRLF (not CR + LF, not an error).
        let reader = ScriptedBufRead::new(vec![
            ScriptAction::Data(b"ab\r".to_vec()),
            ScriptAction::Interrupted,
            ScriptAction::Data(b"\ncd".to_vec()),
        ]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![
                ("ab".to_string(), Some(LineEnding::CRLF)),
                ("cd".to_string(), None),
            ]
        );
    }

    #[test]
    fn interrupted_mid_line_is_transparent() {
        let reader = ScriptedBufRead::new(vec![
            ScriptAction::Data(b"he".to_vec()),
            ScriptAction::Interrupted,
            ScriptAction::Data(b"llo\n".to_vec()),
        ]);
        assert_eq!(
            unwrap_all(collect_all(reader)),
            vec![("hello".to_string(), Some(LineEnding::LF)),]
        );
    }

    #[test]
    fn consume_line_ending_retries_interrupted_lookahead() {
        let mut reader = ScriptedBufRead::new(vec![
            ScriptAction::Data(b"\r".to_vec()),
            ScriptAction::Interrupted,
            ScriptAction::Interrupted,
            ScriptAction::Data(b"\n".to_vec()),
        ]);
        assert_eq!(
            reader.consume_line_ending().expect("consume"),
            Some(LineEnding::CRLF)
        );
    }

    #[test]
    fn large_mixed_input_counts_match() {
        use line_ending::LineEndingScores;
        let mut data = Vec::new();
        for i in 0..25_000 {
            data.extend_from_slice(format!("line{i}").as_bytes());
            data.extend_from_slice(match i % 3 {
                0 => b"\r\n",
                1 => b"\n",
                _ => b"\r",
            });
        }
        let text = String::from_utf8(data.clone()).expect("ascii");
        let expected: LineEndingScores = LineEnding::score_mixed_types(&text);

        let reader = BufReader::with_capacity(7, &data[..]);
        let mut counts = LineEndingScores::new();
        for item in LinesStreaming::new(reader) {
            let (_, ending) = item.expect("line should stream");
            if let Some(le) = ending {
                *counts.entry(le).or_insert(0) += 1;
            }
        }
        // Final line has a trailing break in this fixture, so every break is
        // a terminator; counts must match the scorer exactly.
        assert_eq!(counts, expected);
    }

    /// Differential test: `LinesStreaming` must agree with a trivial
    /// reference splitter on randomized inputs, at every buffer capacity
    /// from 1 to 8 bytes. Exercises split CRLF, chunk-edge `\r`, and
    /// multi-byte UTF-8 content straddling buffer boundaries (bytes
    /// `b'\r'`/`b'\n'` never appear inside multi-byte sequences, so
    /// content must pass through byte-identical).
    ///
    /// Deterministic: fixed-seed LCG, no dependencies.
    #[test]
    fn streaming_matches_reference_on_random_inputs() {
        fn reference(text: &str) -> Vec<(String, Option<LineEnding>)> {
            let mut out = Vec::new();
            let mut cur = String::new();
            let mut chars = text.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '\r' => {
                        if chars.peek() == Some(&'\n') {
                            chars.next();
                            out.push((std::mem::take(&mut cur), Some(LineEnding::CRLF)));
                        } else {
                            out.push((std::mem::take(&mut cur), Some(LineEnding::CR)));
                        }
                    }
                    '\n' => out.push((std::mem::take(&mut cur), Some(LineEnding::LF))),
                    _ => cur.push(c),
                }
            }
            out.push((cur, None));
            // Reference emits a trailing ("", None) for inputs ending in a
            // break; drop it to match iterator convention (done at EOF).
            if out.len() >= 2 {
                let n = out.len();
                if out[n - 2].1.is_some() && out[n - 1].0.is_empty() && out[n - 1].1.is_none() {
                    out.pop();
                }
            }
            out
        }

        // Alphabet includes lone/trailing breaks and 2-byte UTF-8.
        let pieces = ["a", "\r", "\n", "\r\n", "é", "\r\r", "\n\n", "x"];
        let mut state: u64 = 0x1234_5678_9ABC_DEF0;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };

        for round in 0..500 {
            let mut text = String::new();
            let n = 1 + next() % 12;
            for _ in 0..n {
                text.push_str(pieces[next() % pieces.len()]);
            }
            let expected = reference(&text);
            for capacity in 1..=8usize {
                let reader = BufReader::with_capacity(capacity, text.as_bytes());
                let got: Vec<(String, Option<LineEnding>)> = LinesStreaming::new(reader)
                    .collect::<io::Result<_>>()
                    .unwrap_or_else(|e| panic!("round {round} cap {capacity} err: {e}"));
                assert_eq!(
                    got, expected,
                    "round {round} capacity {capacity} input {text:?}"
                );
            }
        }
    }
}
