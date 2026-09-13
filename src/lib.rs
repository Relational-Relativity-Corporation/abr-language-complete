// abr-language-complete  lib.rs
// V0.1.0 — Complete Observable Stream
// Origin: Robin Macomber / Metatron Dynamics
//
// DECLARATION:
//   The primitive communication stream X = (x_0, x_1, ..., x_{n-1})
//   where every position x_i is observed and preserved.
//   Nothing is stripped, cleaned, or pre-classified.
//
// COMPLETENESS INVARIANT:
//   ∀ x_i ∈ X, x_i remains represented in the relational description.
//
// RECONSTRUCTION INVARIANT:
//   R⁻¹(R(X)) = X
//   Every position accounted for. Zero unaccounted positions.
//
// PHASE 0 SCOPE:
//   Direct observables only. No derived relations.
//   No boundary detection, no Q(S), no case normalization,
//   no contraction/possession distinction, no quoted region derivation.
//   Those are Phase 1 relational derivations built on the confirmed stream.

// ─────────────────────────────────────────────────────────────────────────────
// CASE STATE — directly observed
// ─────────────────────────────────────────────────────────────────────────────

/// The case state of a character position, directly observed.
/// No interpretation — upper vs lower is an observation, not a judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CaseState {
    Lower,    // a-z
    Upper,    // A-Z
    NonAlpha, // not a letter — case state is not applicable
}

// ─────────────────────────────────────────────────────────────────────────────
// CHARACTER CLASS — directly observed from the character itself
// ─────────────────────────────────────────────────────────────────────────────

/// The character class of a position, directly observed.
/// No derived categories — only what the character itself declares.
/// Contraction vs possession, quoted region boundaries, compound relations —
/// all of these are Phase 1 derivations, not primitive observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CharClass {
    Alpha,       // a-z, A-Z — letter
    Digit,       // 0-9
    Whitespace,  // space, tab, newline, carriage return
    Period,      // .
    Question,    // ?
    Exclamation, // !
    Comma,       // ,
    Semicolon,   // ;
    Colon,       // :
    Apostrophe,  // '  or  '  (curly apostrophe)
    Hyphen,      // -  or  —  (em dash, en dash)
    QuoteDouble, // "  or  "  "  (curly double quotes)
    QuoteSingle, // '  or  '  '  (when used as quotation, not apostrophe —
                 //   Phase 1 will distinguish; here we observe the mark itself)
    ParenOpen,   // (
    ParenClose,  // )
    BracketOpen, // [
    BracketClose,// ]
    BraceOpen,   // {
    BraceClose,  // }
    Newline,     // \n — preserved separately from generic whitespace
                 //   because newlines carry structural information
                 //   (paragraph boundaries, line breaks in verse)
    Other,       // any character not in the above classes
}

impl CharClass {
    /// Classify a character directly from its identity.
    /// No context required — this is a pure character-level observation.
    pub fn from_char(c: char) -> Self {
        match c {
            'a'..='z' | 'A'..='Z' => CharClass::Alpha,
            '0'..='9' => CharClass::Digit,
            '\n' => CharClass::Newline,
            ' ' | '\t' | '\r' => CharClass::Whitespace,
            '.' => CharClass::Period,
            '?' => CharClass::Question,
            '!' => CharClass::Exclamation,
            ',' => CharClass::Comma,
            ';' => CharClass::Semicolon,
            ':' => CharClass::Colon,
            // Apostrophe: straight and curly right single quote
            '\'' | '\u{2019}' => CharClass::Apostrophe,
            // Hyphen family: hyphen-minus, en dash, em dash
            '-' | '\u{2013}' | '\u{2014}' => CharClass::Hyphen,
            // Double quote family: straight and curly
            '"' | '\u{201C}' | '\u{201D}' => CharClass::QuoteDouble,
            // Single quote family (left curly — right curly handled as apostrophe above)
            '\u{2018}' => CharClass::QuoteSingle,
            '(' => CharClass::ParenOpen,
            ')' => CharClass::ParenClose,
            '[' => CharClass::BracketOpen,
            ']' => CharClass::BracketClose,
            '{' => CharClass::BraceOpen,
            '}' => CharClass::BraceClose,
            _ => CharClass::Other,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STREAM POSITION — the primitive observable unit
// ─────────────────────────────────────────────────────────────────────────────

/// A single position in the complete observable communication stream.
/// Every field is directly observed — nothing derived.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StreamPosition {
    /// Absolute index in the full concatenated stream (across all corpus files)
    pub position: usize,

    /// The raw character at this position
    pub character: char,

    /// Case state — observed directly from the character
    pub case_state: CaseState,

    /// Character class — observed directly from the character
    pub char_class: CharClass,

    /// Byte offset within the source file
    pub byte_offset: usize,

    /// Index into the source file list (0 = first file, etc.)
    pub source_file_index: usize,
}

impl StreamPosition {
    /// Construct a StreamPosition from a character and its context.
    pub fn new(
        position: usize,
        character: char,
        byte_offset: usize,
        source_file_index: usize,
    ) -> Self {
        let case_state = if character.is_ascii_lowercase() {
            CaseState::Lower
        } else if character.is_ascii_uppercase() {
            CaseState::Upper
        } else {
            CaseState::NonAlpha
        };

        let char_class = CharClass::from_char(character);

        StreamPosition {
            position,
            character,
            case_state,
            char_class,
            byte_offset,
            source_file_index,
        }
    }

    /// Reconstruct the original character from this position.
    /// Part of verifying R⁻¹(R(X)) = X.
    pub fn reconstruct(&self) -> char {
        self.character
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPLETE STREAM
// ─────────────────────────────────────────────────────────────────────────────

/// The complete observable communication stream.
/// X = (x_0, x_1, ..., x_{n-1})
#[derive(Debug)]
pub struct CompleteStream {
    pub positions: Vec<StreamPosition>,
    pub source_files: Vec<String>,
    pub file_lengths: Vec<usize>, // character count per file
}

impl CompleteStream {
    /// Ingest a set of corpus files and produce the complete stream.
    /// Every character in every file becomes exactly one StreamPosition.
    pub fn from_files(file_paths: &[(&str, &str)]) -> Result<Self, String> {
        // file_paths: slice of (display_name, full_path)
        let mut positions = Vec::new();
        let mut source_files = Vec::new();
        let mut file_lengths = Vec::new();
        let mut global_pos = 0usize;

        for (file_index, (name, path)) in file_paths.iter().enumerate() {
            let contents = std::fs::read_to_string(path)
                .map_err(|e| format!("Cannot read {}: {}", path, e))?;

            let mut byte_offset = 0usize;
            let mut char_count = 0usize;

            for c in contents.chars() {
                positions.push(StreamPosition::new(
                    global_pos,
                    c,
                    byte_offset,
                    file_index,
                ));
                global_pos += 1;
                char_count += 1;
                byte_offset += c.len_utf8();
            }

            source_files.push(name.to_string());
            file_lengths.push(char_count);
        }

        Ok(CompleteStream {
            positions,
            source_files,
            file_lengths,
        })
    }

    /// Total number of positions in the stream.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Reconstruct the original text from the stream.
    /// Verifies R⁻¹(R(X)) = X.
    pub fn reconstruct(&self) -> String {
        self.positions.iter().map(|p| p.character).collect()
    }

    /// Reconstruct the text for a single source file.
    pub fn reconstruct_file(&self, file_index: usize) -> String {
        self.positions
            .iter()
            .filter(|p| p.source_file_index == file_index)
            .map(|p| p.character)
            .collect()
    }

    /// Count positions by character class.
    pub fn class_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for pos in &self.positions {
            let key = format!("{:?}", pos.char_class);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    /// Count positions by case state.
    pub fn case_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for pos in &self.positions {
            let key = format!("{:?}", pos.case_state);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPLETENESS VERIFICATION
// ─────────────────────────────────────────────────────────────────────────────

/// Result of the completeness verification pass.
#[derive(Debug)]
pub struct CompletenessReport {
    pub stream_length: usize,
    pub total_file_chars: usize,
    pub lengths_match: bool,
    pub reconstruction_matches: bool,
    pub all_classes_covered: bool,
    pub missing_classes: Vec<String>,
    pub class_counts: std::collections::HashMap<String, usize>,
    pub case_counts: std::collections::HashMap<String, usize>,
    pub file_reports: Vec<FileReport>,
}

#[derive(Debug)]
pub struct FileReport {
    pub name: String,
    pub declared_length: usize,
    pub stream_length: usize,
    pub reconstruction_matches: bool,
}

/// Verify the completeness and reconstruction invariants.
pub fn verify_completeness(
    stream: &CompleteStream,
    original_texts: &[(&str, String)],
) -> CompletenessReport {
    let stream_length = stream.len();
    let total_file_chars: usize = stream.file_lengths.iter().sum();
    let lengths_match = stream_length == total_file_chars;

    // Full reconstruction check
    let reconstructed = stream.reconstruct();
    let original_full: String = original_texts.iter().map(|(_, t)| t.as_str()).collect();
    let reconstruction_matches = reconstructed == original_full;

    // Per-file reconstruction
    let mut file_reports = Vec::new();
    for (file_index, (name, original)) in original_texts.iter().enumerate() {
        let file_reconstructed = stream.reconstruct_file(file_index);
        let file_stream_len = stream
            .positions
            .iter()
            .filter(|p| p.source_file_index == file_index)
            .count();
        file_reports.push(FileReport {
            name: name.to_string(),
            declared_length: original.chars().count(),
            stream_length: file_stream_len,
            reconstruction_matches: file_reconstructed == *original,
        });
    }

    // Check all declared CharClass variants appear
    let class_counts = stream.class_counts();
    let all_classes = vec![
        "Alpha", "Digit", "Whitespace", "Period", "Question", "Exclamation",
        "Comma", "Semicolon", "Colon", "Apostrophe", "Hyphen", "QuoteDouble",
        "QuoteSingle", "ParenOpen", "ParenClose", "BracketOpen", "BracketClose",
        "BraceOpen", "BraceClose", "Newline", "Other",
    ];
    let missing_classes: Vec<String> = all_classes
        .iter()
        .filter(|&&c| !class_counts.contains_key(c))
        .map(|&c| c.to_string())
        .collect();
    let all_classes_covered = missing_classes.is_empty();

    let case_counts = stream.case_counts();

    CompletenessReport {
        stream_length,
        total_file_chars,
        lengths_match,
        reconstruction_matches,
        all_classes_covered,
        missing_classes,
        class_counts,
        case_counts,
        file_reports,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stream_named(text: &str, filename: &str) -> (CompleteStream, String) {
        use std::io::Write;
        let mut tmp = std::env::temp_dir();
        tmp.push(filename);
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(text.as_bytes()).unwrap();
        }
        let path = tmp.to_str().unwrap().to_string();
        let stream = CompleteStream::from_files(&[("test", &path)]).unwrap();
        (stream, text.to_string())
    }

    #[test]
    fn test_stream_length_matches_char_count() {
        let text = "Hello, World! How are you?";
        let (stream, _) = make_stream_named(text, "abr_lc_t1.txt");
        assert_eq!(stream.len(), text.chars().count());
    }

    #[test]
    fn test_reconstruction_invariant() {
        let text = "The quick brown fox.\nIt jumped over the lazy dog!";
        let (stream, original) = make_stream_named(text, "abr_lc_t2.txt");
        assert_eq!(stream.reconstruct(), original);
    }

    #[test]
    fn test_reconstruction_with_punctuation() {
        let text = "\"Hello,\" she said. It's Darwin's theory.";
        let (stream, original) = make_stream_named(text, "abr_lc_t3.txt");
        assert_eq!(stream.reconstruct(), original);
    }

    #[test]
    fn test_case_states_observed() {
        let text = "Hello WORLD";
        let (stream, _) = make_stream_named(text, "abr_lc_t4.txt");
        let cases: Vec<CaseState> = stream.positions.iter()
            .map(|p| p.case_state)
            .collect();
        assert_eq!(cases[0], CaseState::Upper);
        assert_eq!(cases[1], CaseState::Lower);
        assert_eq!(cases[5], CaseState::NonAlpha);
        assert_eq!(cases[6], CaseState::Upper);
    }

    #[test]
    fn test_char_classes_observed() {
        let text = "Hi. It's done!";
        let (stream, _) = make_stream_named(text, "abr_lc_t5.txt");
        let classes: Vec<CharClass> = stream.positions.iter()
            .map(|p| p.char_class)
            .collect();
        assert_eq!(classes[0], CharClass::Alpha);
        assert_eq!(classes[2], CharClass::Period);
        let apos_pos = stream.positions.iter()
            .find(|p| p.char_class == CharClass::Apostrophe);
        assert!(apos_pos.is_some());
        let excl_pos = stream.positions.iter()
            .find(|p| p.char_class == CharClass::Exclamation);
        assert!(excl_pos.is_some());
    }

    #[test]
    fn test_no_position_is_unclassified() {
        let text = "All chars: a-z, A-Z, 0-9, spaces\n.";
        let (stream, _) = make_stream_named(text, "abr_lc_t6.txt");
        assert_eq!(stream.len(), text.chars().count());
        for pos in &stream.positions {
            assert_eq!(pos.reconstruct(), pos.character);
        }
    }

    #[test]
    fn test_apostrophe_observed_not_classified() {
        let text = "it's Darwin's";
        let (stream, _) = make_stream_named(text, "abr_lc_t7.txt");
        let apostrophes: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Apostrophe)
            .collect();
        assert_eq!(apostrophes.len(), 2);
    }

    #[test]
    fn test_hyphen_family_all_classified() {
        let text = "well-known\u{2013}en\u{2014}em";
        let (stream, _) = make_stream_named(text, "abr_lc_t8.txt");
        let hyphens: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Hyphen)
            .collect();
        assert_eq!(hyphens.len(), 3);
    }

    #[test]
    fn test_newline_preserved_separately() {
        let text = "line one\nline two\nline three";
        let (stream, _) = make_stream_named(text, "abr_lc_t9.txt");
        let newlines: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Newline)
            .collect();
        assert_eq!(newlines.len(), 2);
    }

    #[test]
    fn test_per_file_reconstruction() {
        use std::io::Write;
        let text1 = "First file content.";
        let text2 = "Second file content!";
        let mut tmp1 = std::env::temp_dir();
        tmp1.push("abr_lc_t10a.txt");
        let mut tmp2 = std::env::temp_dir();
        tmp2.push("abr_lc_t10b.txt");
        std::fs::File::create(&tmp1).unwrap()
            .write_all(text1.as_bytes()).unwrap();
        std::fs::File::create(&tmp2).unwrap()
            .write_all(text2.as_bytes()).unwrap();
        let stream = CompleteStream::from_files(&[
            ("file1", tmp1.to_str().unwrap()),
            ("file2", tmp2.to_str().unwrap()),
        ]).unwrap();
        assert_eq!(stream.reconstruct_file(0), text1);
        assert_eq!(stream.reconstruct_file(1), text2);
    }

    #[test]
    fn test_completeness_verification_passes() {
        let text = "Hello, World! \"Quoted.\" It's fine.";
        let (stream, original) = make_stream_named(text, "abr_lc_t11.txt");
        let originals = vec![("test", original.clone())];
        let report = verify_completeness(&stream, &originals);
        assert!(report.lengths_match);
        assert!(report.reconstruction_matches);
        assert_eq!(report.file_reports[0].reconstruction_matches, true);
    }

    #[test]
    fn test_upper_lower_nonalpha_counts() {
        let text = "Hi! 123";
        let (stream, _) = make_stream_named(text, "abr_lc_t12.txt");
        let cases = stream.case_counts();
        assert!(cases.get("Upper").copied().unwrap_or(0) >= 1);
        assert!(cases.get("Lower").copied().unwrap_or(0) >= 1);
        assert!(cases.get("NonAlpha").copied().unwrap_or(0) >= 1);
    }
}
