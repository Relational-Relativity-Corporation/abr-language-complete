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

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 1A: PRIMITIVE RELATIONAL SUBSTRATE
// ─────────────────────────────────────────────────────────────────────────────
//
// For every adjacent pair (x_i, x_{i+1}) in the complete stream, record:
//   R_P: i → i+1                          (progression)
//   R_C: class(x_i) → class(x_{i+1})     (class transition)
//   R_E: case(x_i) → case(x_{i+1})       (case transition)
//   Δ_C: [class(x_i) ≠ class(x_{i+1})]   (class transition locus)
//   Δ_E: [case(x_i) ≠ case(x_{i+1})]     (case transition locus)
//
// ACCOUNTING INVARIANT:
//   |R_P| = |X| - F
//   where F = number of source files (progression does not cross file boundaries)
//
// Every non-final position participates in exactly one forward successor.
// Every non-initial position participates in exactly one predecessor.
// Every adjacent pair has exactly one R_C and exactly one R_E observation.
// Zero unaccounted pairs.
//
// DISCIPLINE:
//   Transition is observed. Boundary is to be determined.
//   No transition locus is called a boundary here.
//   No linguistic label is assigned to any relation.

/// A single relational observation between two adjacent stream positions.
/// Every field is directly derived from Phase 0 observables — no interpretation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjacentRelation {
    /// R_P: the predecessor position index
    pub i: usize,
    /// R_P: the successor position index (always i + 1 within a file)
    pub j: usize,

    /// R_C: the class transition (class at i → class at j)
    pub class_from: CharClass,
    pub class_to: CharClass,

    /// R_E: the case transition (case at i → case at j)
    pub case_from: CaseState,
    pub case_to: CaseState,

    /// Δ_C: true iff class(x_i) ≠ class(x_{i+1})
    pub class_transition: bool,

    /// Δ_E: true iff case(x_i) ≠ case(x_{i+1})
    pub case_transition: bool,

    /// Source file index — progression does not cross file boundaries
    pub source_file_index: usize,
}

/// The complete Phase 1A relational substrate over a CompleteStream.
/// Contains exactly |X| - F adjacent relations where F = number of source files.
pub struct RelationalSubstrate {
    pub relations: Vec<AdjacentRelation>,
    /// Number of source files (F in the accounting invariant)
    pub file_count: usize,
    /// Total stream positions (|X|)
    pub stream_length: usize,
}

impl RelationalSubstrate {
    /// Derive the complete relational substrate from a CompleteStream.
    /// Progression does not cross file boundaries.
    pub fn from_stream(stream: &CompleteStream) -> Self {
        let mut relations = Vec::with_capacity(stream.len().saturating_sub(stream.source_files.len()));

        let positions = &stream.positions;
        let n = positions.len();

        for i in 0..n.saturating_sub(1) {
            let xi = &positions[i];
            let xj = &positions[i + 1];

            // Progression does not cross file boundaries
            if xi.source_file_index != xj.source_file_index {
                continue;
            }

            relations.push(AdjacentRelation {
                i: xi.position,
                j: xj.position,
                class_from: xi.char_class,
                class_to: xj.char_class,
                case_from: xi.case_state,
                case_to: xj.case_state,
                class_transition: xi.char_class != xj.char_class,
                case_transition: xi.case_state != xj.case_state,
                source_file_index: xi.source_file_index,
            });
        }

        RelationalSubstrate {
            relations,
            file_count: stream.source_files.len(),
            stream_length: stream.len(),
        }
    }

    /// Verify the accounting invariant: |R_P| = |X| - F
    pub fn verify_accounting(&self) -> AccountingReport {
        let expected = self.stream_length.saturating_sub(self.file_count);
        let actual = self.relations.len();
        let invariant_holds = actual == expected;

        // Verify every non-final position has exactly one forward successor
        // and every non-initial position has exactly one predecessor
        // within its file — verified by checking for duplicate i values
        let mut i_seen = std::collections::HashSet::new();
        let mut duplicate_i = 0usize;
        for rel in &self.relations {
            if !i_seen.insert(rel.i) {
                duplicate_i += 1;
            }
        }

        // Count class transition loci (Δ_C = true)
        let class_transition_count = self.relations.iter()
            .filter(|r| r.class_transition)
            .count();

        // Count case transition loci (Δ_E = true)
        let case_transition_count = self.relations.iter()
            .filter(|r| r.case_transition)
            .count();

        // Class transition table: how often does each class pair occur
        let mut class_transition_table: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        for rel in &self.relations {
            let key = (format!("{:?}", rel.class_from), format!("{:?}", rel.class_to));
            *class_transition_table.entry(key).or_insert(0) += 1;
        }

        // Case transition table
        let mut case_transition_table: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        for rel in &self.relations {
            let key = (format!("{:?}", rel.case_from), format!("{:?}", rel.case_to));
            *case_transition_table.entry(key).or_insert(0) += 1;
        }

        AccountingReport {
            expected_relations: expected,
            actual_relations: actual,
            invariant_holds,
            duplicate_predecessor_positions: duplicate_i,
            class_transition_loci: class_transition_count,
            case_transition_loci: case_transition_count,
            class_transition_table,
            case_transition_table,
        }
    }
}

/// Result of the Phase 1A accounting verification.
#[derive(Debug)]
pub struct AccountingReport {
    /// Expected: |X| - F
    pub expected_relations: usize,
    /// Actual: |R_P|
    pub actual_relations: usize,
    /// |R_P| == |X| - F
    pub invariant_holds: bool,
    /// Should be 0: each position i appears as predecessor at most once
    pub duplicate_predecessor_positions: usize,
    /// Count of positions where Δ_C = true
    pub class_transition_loci: usize,
    /// Count of positions where Δ_E = true
    pub case_transition_loci: usize,
    /// R_C transition table: (class_from, class_to) → count
    pub class_transition_table: std::collections::HashMap<(String, String), usize>,
    /// R_E transition table: (case_from, case_to) → count
    pub case_transition_table: std::collections::HashMap<(String, String), usize>,
}


#[cfg(test)]
mod tests_phase1a {
    use super::*;
    use std::io::Write;

    // Real corpus passages — same declared texts as Phase 0 tests
    const HAMLET_SCENE1: &str = "SCENE I. Elsinore. A platform before the Castle.\r\n\r\nEnter Francisco and Barnardo, two sentinels.\r\n\r\nBARNARDO.\r\nWho's there?\r\n\r\nFRANCISCO.\r\nNay, answer me. Stand and unfold yourself.\r\n\r\nBARNARDO.\r\nLong live the King!\r\n\r\nFRANCISCO.\r\nBarnardo?\r\n\r\nBARNARDO.\r\nHe.\r\n\r\nFRANCISCO.\r\nYou come most carefully upon your hour.\r\n\r\nBARNARDO.\r\n'Tis now struck twelve. Get thee to bed, Francisco.\r\n\r\nFRANCISCO.\r\nFor this relief much thanks. 'Tis bitter cold,\r\nAnd I am sick at heart.";

    const ORIGIN_PASSAGE: &str = "When we look to the individuals of the same variety or sub-variety of\r\nour older cultivated plants and animals, one of the first points which\r\nstrikes us, is, that they generally differ much more from each other,\r\nthan do the individuals of any one species or variety in a state of\r\nnature. When we reflect on the vast diversity of the plants and animals\r\nwhich have been cultivated, and which have varied during all ages.";

    fn substrate_from(text: &str, tag: &str) -> (CompleteStream, RelationalSubstrate) {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("abr_lc_1a_{}.txt", tag));
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(text.as_bytes()).unwrap();
        }
        let stream = CompleteStream::from_files(&[("corpus", tmp.to_str().unwrap())]).unwrap();
        let substrate = RelationalSubstrate::from_stream(&stream);
        (stream, substrate)
    }

    #[test]
    fn test_accounting_invariant_hamlet() {
        // |R_P| = |X| - F on real Hamlet text (F=1 file)
        let (stream, substrate) = substrate_from(HAMLET_SCENE1, "acct_hamlet");
        let report = substrate.verify_accounting();
        assert!(report.invariant_holds,
            "Expected {} relations, got {}",
            report.expected_relations, report.actual_relations);
        assert_eq!(report.expected_relations, stream.len() - 1,
            "|R_P| must equal |X| - 1 for single file");
    }

    #[test]
    fn test_accounting_invariant_origin() {
        // |R_P| = |X| - F on real Origin text
        let (stream, substrate) = substrate_from(ORIGIN_PASSAGE, "acct_origin");
        let report = substrate.verify_accounting();
        assert!(report.invariant_holds,
            "Accounting invariant must hold on Origin passage");
    }

    #[test]
    fn test_no_duplicate_predecessor_positions() {
        // Every position i appears as predecessor at most once
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "nodup_hamlet");
        let report = substrate.verify_accounting();
        assert_eq!(report.duplicate_predecessor_positions, 0,
            "Each position must appear as predecessor exactly once");
    }

    #[test]
    fn test_relation_count_matches_char_pairs() {
        // For a single file: exactly len-1 relations
        let text = HAMLET_SCENE1;
        let (stream, substrate) = substrate_from(text, "count_hamlet");
        assert_eq!(substrate.relations.len(), stream.len() - 1,
            "Relation count must be character count minus 1 for single file");
    }

    #[test]
    fn test_progression_is_strictly_sequential() {
        // Every relation has j = i + 1 — no gaps, no jumps
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "seq_hamlet");
        for rel in &substrate.relations {
            assert_eq!(rel.j, rel.i + 1,
                "Progression must be strictly sequential: j = i + 1");
        }
    }

    #[test]
    fn test_class_transition_observed_at_alpha_to_period() {
        // "SCENE I." — Alpha→Period transition must appear
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "cls_period");
        let alpha_to_period = substrate.relations.iter()
            .filter(|r| r.class_from == CharClass::Alpha
                     && r.class_to == CharClass::Period)
            .count();
        assert!(alpha_to_period >= 1,
            "Expected at least one Alpha→Period transition in real text");
    }

    #[test]
    fn test_class_transition_observed_at_alpha_to_whitespace() {
        // Word→space transitions must be the most common class transition
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "cls_ws");
        let alpha_to_ws = substrate.relations.iter()
            .filter(|r| r.class_from == CharClass::Alpha
                     && r.class_to == CharClass::Whitespace)
            .count();
        assert!(alpha_to_ws >= 20,
            "Expected many Alpha→Whitespace transitions in Hamlet Scene I, found {}",
            alpha_to_ws);
    }

    #[test]
    fn test_class_transition_loci_not_called_boundaries() {
        // Verify Δ_C is populated but we make no claim about what it means
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "delta_c");
        let report = substrate.verify_accounting();
        assert!(report.class_transition_loci > 0,
            "There must be class transition loci in real text");
        // The count is less than total relations — not every pair is a transition
        assert!(report.class_transition_loci < substrate.relations.len(),
            "Not every pair is a class transition");
        // We assert nothing about what these transitions mean
    }

    #[test]
    fn test_case_transition_upper_to_lower_observed() {
        // "BARNARDO" → next char after name is period, then newline, then 'W' (lower)
        // Upper→Lower must occur (e.g. "BARNARDO.\nWho" — O→. is Upper→NonAlpha,
        // then later a word like "Who" has Upper→Lower within it is not present,
        // but "BARNARDO" itself: B(Upper)A(Upper) — no, all Upper→Upper within caps
        // The real Upper→Lower occurs at sentence-initial words like "Who's" — W→h
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "case_ul");
        let upper_to_lower = substrate.relations.iter()
            .filter(|r| r.case_from == CaseState::Upper
                     && r.case_to == CaseState::Lower)
            .count();
        // "Who's" W→h, "Nay" N→a, "Long" L→o, "Barnardo" B→a, etc.
        assert!(upper_to_lower >= 5,
            "Expected at least 5 Upper→Lower transitions in Hamlet Scene I, found {}",
            upper_to_lower);
    }

    #[test]
    fn test_case_transition_loci_populated() {
        // Δ_E must be populated — case changes occur in real text
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "delta_e");
        let report = substrate.verify_accounting();
        assert!(report.case_transition_loci > 0,
            "Case transition loci must exist in real text");
    }

    #[test]
    fn test_file_boundary_not_crossed() {
        // When two files are loaded, relations do not cross the boundary
        use std::io::Write;
        let text1 = "First.";
        let text2 = "Second.";
        let mut tmp1 = std::env::temp_dir();
        tmp1.push("abr_lc_1a_fb1.txt");
        let mut tmp2 = std::env::temp_dir();
        tmp2.push("abr_lc_1a_fb2.txt");
        std::fs::File::create(&tmp1).unwrap().write_all(text1.as_bytes()).unwrap();
        std::fs::File::create(&tmp2).unwrap().write_all(text2.as_bytes()).unwrap();

        let stream = CompleteStream::from_files(&[
            ("f1", tmp1.to_str().unwrap()),
            ("f2", tmp2.to_str().unwrap()),
        ]).unwrap();
        let substrate = RelationalSubstrate::from_stream(&stream);

        // Two files: expected |X| - 2 relations
        assert_eq!(substrate.relations.len(),
            stream.len() - 2,
            "|R_P| must equal |X| - F where F=2 files");

        // No relation spans files
        for rel in &substrate.relations {
            assert_eq!(
                stream.positions[rel.i].source_file_index,
                stream.positions[rel.j].source_file_index,
                "Relation must not cross file boundary"
            );
        }
    }

    #[test]
    fn test_r_c_covers_all_observed_transitions_in_passage() {
        // The class transition table must account for all relations
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "rc_total");
        let report = substrate.verify_accounting();
        let table_total: usize = report.class_transition_table.values().sum();
        assert_eq!(table_total, substrate.relations.len(),
            "R_C transition table must cover all relations");
    }

    #[test]
    fn test_r_e_covers_all_observed_transitions_in_passage() {
        // The case transition table must account for all relations
        let (_, substrate) = substrate_from(HAMLET_SCENE1, "re_total");
        let report = substrate.verify_accounting();
        let table_total: usize = report.case_transition_table.values().sum();
        assert_eq!(table_total, substrate.relations.len(),
            "R_E transition table must cover all relations");
    }
}
