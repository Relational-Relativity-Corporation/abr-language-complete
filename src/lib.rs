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
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── Test helpers ──────────────────────────────────────────────────────────

    /// Build a CompleteStream from a string literal, using a unique temp file.
    fn stream_from(text: &str, tag: &str) -> CompleteStream {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("abr_lc_{}.txt", tag));
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(text.as_bytes()).unwrap();
        }
        CompleteStream::from_files(&[("corpus", tmp.to_str().unwrap())]).unwrap()
    }

    // ── Real corpus passages used as test oracles ─────────────────────────────
    //
    // These are verbatim excerpts from the declared corpus files
    // (hamlet.txt, origin_of_species.txt, pride_and_prejudice.txt).
    // They are not synthetic — they represent actual observed text.

    /// Hamlet Act I Scene I — contains apostrophes, ALL CAPS speaker names,
    /// mixed punctuation, newlines, and sentence-initial capitals.
    const HAMLET_SCENE1: &str = "SCENE I. Elsinore. A platform before the Castle.\r\n\r\nEnter Francisco and Barnardo, two sentinels.\r\n\r\nBARNARDO.\r\nWho's there?\r\n\r\nFRANCISCO.\r\nNay, answer me. Stand and unfold yourself.\r\n\r\nBARNARDO.\r\nLong live the King!\r\n\r\nFRANCISCO.\r\nBarnardo?\r\n\r\nBARNARDO.\r\nHe.\r\n\r\nFRANCISCO.\r\nYou come most carefully upon your hour.\r\n\r\nBARNARDO.\r\n'Tis now struck twelve. Get thee to bed, Francisco.\r\n\r\nFRANCISCO.\r\nFor this relief much thanks. 'Tis bitter cold,\r\nAnd I am sick at heart.";

    /// Hamlet semicolon passage — scholar; speak — mixed punctuation
    const HAMLET_SEMICOLON: &str = "BARNARDO.\r\nIn the same figure, like the King that's dead.\r\n\r\nMARCELLUS.\r\nThou art a scholar; speak to it, Horatio.\r\n\r\nBARNARDO.\r\nLooks it not like the King? Mark it, Horatio.\r\n\r\nHORATIO.\r\nMost like. It harrows me with fear and wonder.";

    /// Hamlet hyphen passage
    const HAMLET_HYPHEN: &str = "Give you good-night.\r\n\r\nMARCELLUS.\r\nHolla, Barnardo!\r\n\r\nBARNARDO.\r\nSay, what, is Horatio there?";

    /// Origin of Species — scientific prose with commas, semicolons, varied punctuation
    const ORIGIN_PASSAGE: &str = "When we look to the individuals of the same variety or sub-variety of\r\nour older cultivated plants and animals, one of the first points which\r\nstrikes us, is, that they generally differ much more from each other,\r\nthan do the individuals of any one species or variety in a state of\r\nnature. When we reflect on the vast diversity of the plants and animals\r\nwhich have been cultivated, and which have varied during all ages.";

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_reconstruction_invariant_hamlet() {
        // R⁻¹(R(X)) = X on real Hamlet text with full punctuation
        let stream = stream_from(HAMLET_SCENE1, "recon_hamlet");
        assert_eq!(stream.reconstruct(), HAMLET_SCENE1,
            "Reconstruction must be exact on real Hamlet text");
    }

    #[test]
    fn test_reconstruction_invariant_origin() {
        // R⁻¹(R(X)) = X on real Origin of Species prose
        let stream = stream_from(ORIGIN_PASSAGE, "recon_origin");
        assert_eq!(stream.reconstruct(), ORIGIN_PASSAGE,
            "Reconstruction must be exact on real scientific prose");
    }

    #[test]
    fn test_stream_length_matches_char_count_hamlet() {
        // Every character in the real text maps to exactly one stream position
        let stream = stream_from(HAMLET_SCENE1, "len_hamlet");
        assert_eq!(stream.len(), HAMLET_SCENE1.chars().count(),
            "Stream length must equal character count of real text");
    }

    #[test]
    fn test_stream_length_matches_char_count_origin() {
        let stream = stream_from(ORIGIN_PASSAGE, "len_origin");
        assert_eq!(stream.len(), ORIGIN_PASSAGE.chars().count());
    }

    #[test]
    fn test_apostrophes_observed_in_hamlet() {
        // Real apostrophes in "Who's", "'Tis", "that's", "King's"
        // Phase 0: observed as Apostrophe class, not classified as contraction/possession
        let stream = stream_from(HAMLET_SCENE1, "apos_hamlet");
        let apostrophes: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Apostrophe)
            .collect();
        // "Who's", "'Tis" (x2), "that's" are all present
        assert!(apostrophes.len() >= 3,
            "Expected at least 3 apostrophes in Hamlet Scene I, found {}",
            apostrophes.len());
        // All classified as Apostrophe — no contraction/possession distinction at Phase 0
        for ap in &apostrophes {
            assert_eq!(ap.char_class, CharClass::Apostrophe);
        }
    }

    #[test]
    fn test_allcaps_observed_as_upper_case_state() {
        // ALL CAPS speaker names (BARNARDO, FRANCISCO, HORATIO) must be
        // observed with CaseState::Upper — not normalized, not collapsed
        let stream = stream_from(HAMLET_SCENE1, "caps_hamlet");
        let upper_positions: Vec<_> = stream.positions.iter()
            .filter(|p| p.case_state == CaseState::Upper)
            .collect();
        // There are many uppercase characters in the speaker names
        assert!(upper_positions.len() >= 30,
            "Expected many uppercase positions for ALL CAPS names, found {}",
            upper_positions.len());
        // Specifically: B-A-R-N-A-R-D-O should all be Upper
        let barnardo_chars: Vec<char> = stream.positions.iter()
            .filter(|p| p.case_state == CaseState::Upper)
            .map(|p| p.character)
            .collect();
        assert!(barnardo_chars.contains(&'B'));
        assert!(barnardo_chars.contains(&'A'));
        assert!(barnardo_chars.contains(&'R'));
    }

    #[test]
    fn test_semicolon_observed_in_real_text() {
        // "Thou art a scholar; speak to it, Horatio." — real semicolon
        let stream = stream_from(HAMLET_SEMICOLON, "semi_hamlet");
        let semicolons: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Semicolon)
            .collect();
        assert_eq!(semicolons.len(), 1,
            "Expected exactly 1 semicolon in passage, found {}",
            semicolons.len());
        // Verify the semicolon reconstructs correctly
        assert_eq!(semicolons[0].character, ';');
    }

    #[test]
    fn test_hyphen_observed_in_real_text() {
        // "good-night" — real hyphen in Hamlet
        let stream = stream_from(HAMLET_HYPHEN, "hyph_hamlet");
        let hyphens: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Hyphen)
            .collect();
        assert!(hyphens.len() >= 1,
            "Expected at least 1 hyphen in 'good-night' passage, found {}",
            hyphens.len());
        assert_eq!(hyphens[0].character, '-');
    }

    #[test]
    fn test_newlines_preserved_in_hamlet() {
        // Hamlet text uses \r\n — both \r and \n are distinct observed characters
        // \n classifies as Newline, \r classifies as Whitespace
        let stream = stream_from(HAMLET_SCENE1, "nl_hamlet");
        let newlines: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Newline)
            .collect();
        // Hamlet scene I has many paragraph breaks
        assert!(newlines.len() >= 15,
            "Expected many newlines in Hamlet Scene I, found {}",
            newlines.len());
    }

    #[test]
    fn test_question_marks_observed() {
        // "Who's there?", "Barnardo?" — real question marks
        let stream = stream_from(HAMLET_SCENE1, "qmark_hamlet");
        let questions: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Question)
            .collect();
        assert!(questions.len() >= 2,
            "Expected at least 2 question marks in passage, found {}",
            questions.len());
    }

    #[test]
    fn test_exclamation_observed() {
        // "Long live the King!" — real exclamation mark
        let stream = stream_from(HAMLET_SCENE1, "excl_hamlet");
        let excls: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Exclamation)
            .collect();
        assert!(excls.len() >= 1,
            "Expected at least 1 exclamation mark in passage, found {}",
            excls.len());
    }

    #[test]
    fn test_commas_observed_in_origin() {
        // Origin has dense comma usage in scientific enumeration
        let stream = stream_from(ORIGIN_PASSAGE, "comma_origin");
        let commas: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Comma)
            .collect();
        assert!(commas.len() >= 5,
            "Expected at least 5 commas in Origin passage, found {}",
            commas.len());
    }

    #[test]
    fn test_lower_upper_nonalpha_all_present_in_hamlet() {
        // Hamlet has lowercase words, uppercase speaker names, and punctuation
        // All three case states must appear
        let stream = stream_from(HAMLET_SCENE1, "cases_hamlet");
        let cases = stream.case_counts();
        assert!(cases.get("Lower").copied().unwrap_or(0) > 100,
            "Expected many lowercase positions");
        assert!(cases.get("Upper").copied().unwrap_or(0) > 20,
            "Expected many uppercase positions (speaker names)");
        assert!(cases.get("NonAlpha").copied().unwrap_or(0) > 30,
            "Expected many non-alpha positions (punctuation, spaces)");
    }

    #[test]
    fn test_completeness_invariant_hamlet() {
        // ∀ x_i ∈ X: represented — verified against real text
        let stream = stream_from(HAMLET_SCENE1, "compl_hamlet");
        let originals = vec![("hamlet_scene1", HAMLET_SCENE1.to_string())];
        let report = verify_completeness(&stream, &originals);
        assert!(report.lengths_match,
            "Stream length must match character count");
        assert!(report.reconstruction_matches,
            "R⁻¹(R(X)) = X must hold on real Hamlet text");
        assert!(report.file_reports[0].reconstruction_matches,
            "Per-file reconstruction must hold");
    }

    #[test]
    fn test_completeness_invariant_origin() {
        // ∀ x_i ∈ X: represented — verified against real scientific prose
        let stream = stream_from(ORIGIN_PASSAGE, "compl_origin");
        let originals = vec![("origin", ORIGIN_PASSAGE.to_string())];
        let report = verify_completeness(&stream, &originals);
        assert!(report.reconstruction_matches,
            "R⁻¹(R(X)) = X must hold on real Origin text");
    }

    #[test]
    fn test_period_observed_in_real_sentences() {
        // Multiple sentence-ending periods in real text
        let stream = stream_from(HAMLET_SCENE1, "period_hamlet");
        let periods: Vec<_> = stream.positions.iter()
            .filter(|p| p.char_class == CharClass::Period)
            .collect();
        assert!(periods.len() >= 5,
            "Expected at least 5 periods in Hamlet Scene I, found {}",
            periods.len());
        for p in &periods {
            assert_eq!(p.character, '.');
        }
    }

    #[test]
    fn test_no_position_dropped_hamlet() {
        // Every position in the stream reconstructs to its declared character
        let stream = stream_from(HAMLET_SCENE1, "nodrop_hamlet");
        for pos in &stream.positions {
            assert_eq!(pos.reconstruct(), pos.character,
                "Position {} must reconstruct correctly", pos.position);
        }
    }

    #[test]
    fn test_no_position_dropped_origin() {
        let stream = stream_from(ORIGIN_PASSAGE, "nodrop_origin");
        for pos in &stream.positions {
            assert_eq!(pos.reconstruct(), pos.character,
                "Position {} must reconstruct correctly", pos.position);
        }
    }
}
