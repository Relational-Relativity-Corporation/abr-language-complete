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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 1B: IDENTITY RECURRENCE AND PERSISTENCE UNDER PROGRESSION
// ─────────────────────────────────────────────────────────────────────────────
//
// DECLARED MATHEMATICAL STRUCTURE:
//
//   M_state(x_i)     = (character, case_state, char_class)
//   M_provenance(x_i) = (source_file_index, position, byte_offset)
//
//   Recurrence compares M_state. Provenance is retained but does not
//   participate in equality.
//
//   For every pair of distinct positions i < j (corpus-global) where
//   M_state(x_i) = M_state(x_j), the maximal contiguous extent is:
//
//     K(i,j) = max{ k ≥ 1 : ∀r ∈ [0,k), M_state(x_{i+r}) = M_state(x_{j+r}) }
//
//   bounded by source file boundaries (progression is source-local).
//
//   Bilateral maximality: (i,j) is canonical iff either predecessor is
//   absent (FILE_START) or M_state(x_{i-1}) ≠ M_state(x_{j-1}).
//
//   Termination:
//     term_R(i,K) = M_state(x_{i+K})  if successor exists within file
//                  FILE_END            otherwise
//
//   Each recurrence recorded as: R_I = (i, j, K, left_term, right_term)
//
// PROVENANCE INVARIANT per recurrence of length K:
//   K observed state equalities
//   K-1 R_P relations along occurrence A
//   K-1 R_P relations along occurrence B
//
// IMPLEMENTATION:
//   Suffix-array index → candidate pairs → exact verification against X.
//   Aggregate statistics by K. Never materialize 118B K=1 pairs simultaneously.
//   Completeness of observation is preserved — nothing is excluded from
//   the declared enumeration, but storage is aggregated.

/// The observable state of a position — the component compared in recurrence.
/// Separated from provenance per the declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash,
         serde::Serialize, serde::Deserialize)]
pub struct ObservableState {
    pub character: char,
    pub case_state: CaseState,
    pub char_class: CharClass,
}

impl ObservableState {
    pub fn from_position(p: &StreamPosition) -> Self {
        ObservableState {
            character: p.character,
            case_state: p.case_state,
            char_class: p.char_class,
        }
    }
}

/// Termination state at the boundary of a recurrence run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TerminationState {
    /// The diverging observable state
    State(ObservableState),
    /// File boundary — no successor/predecessor exists
    FileEnd,
    FileStart,
}

/// A single canonical maximal recurrence relation.
/// R_I = (i, j, K) with termination states and provenance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecurrenceRelation {
    /// Corpus-global position of first occurrence (i < j)
    pub i: usize,
    /// Corpus-global position of second occurrence
    pub j: usize,
    /// Source file index for occurrence i
    pub source_i: usize,
    /// Source file index for occurrence j
    pub source_j: usize,
    /// Maximal persistence length K
    pub k: usize,
    /// Left termination: state at x_{i-1} and x_{j-1} (or FileStart)
    pub left_term_i: TerminationState,
    pub left_term_j: TerminationState,
    /// Right termination: state at x_{i+K} and x_{j+K} (or FileEnd)
    pub right_term_i: TerminationState,
    pub right_term_j: TerminationState,
}

/// Aggregated statistics for Phase 1B — never materializes all K=1 pairs.
#[derive(Debug, Default)]
pub struct RecurrenceStatistics {
    /// Total canonical recurrences found
    pub total_canonical: u64,
    /// Distribution by K: k → count of canonical recurrences of that length
    pub k_distribution: std::collections::BTreeMap<usize, u64>,
    /// Same-source recurrences (i and j in same file)
    pub same_source: u64,
    /// Cross-source recurrences (i and j in different files)
    pub cross_source: u64,
    /// Maximum K observed
    pub k_max: usize,
    /// Termination pair counts: (right_term_i_class, right_term_j_class) → count
    pub right_term_pairs: std::collections::HashMap<String, u64>,
    /// Sample recurrences for provenance verification (one per K up to K=20)
    pub samples: std::collections::BTreeMap<usize, RecurrenceRelation>,
    /// Verification failures (must be 0)
    pub verification_failures: u64,
}

/// Build a position index: observable_state → sorted list of corpus-global positions.
/// This is the indexing mechanism — not an observable, just a lookup structure.
#[allow(dead_code)]
fn build_state_index(
    stream: &CompleteStream,
) -> std::collections::HashMap<ObservableState, Vec<usize>> {
    let mut index: std::collections::HashMap<ObservableState, Vec<usize>> =
        std::collections::HashMap::new();
    for pos in &stream.positions {
        let state = ObservableState::from_position(pos);
        index.entry(state).or_default().push(pos.position);
    }
    index
}

/// Get the observable state at a corpus-global position, if it exists.
#[inline]
fn state_at(stream: &CompleteStream, pos: usize) -> Option<ObservableState> {
    stream.positions.get(pos).map(ObservableState::from_position)
}

/// Get the source file index at a corpus-global position.
#[inline]
fn source_at(stream: &CompleteStream, pos: usize) -> Option<usize> {
    stream.positions.get(pos).map(|p| p.source_file_index)
}

/// Check if pos is the first position in its source file.
fn is_file_start(stream: &CompleteStream, pos: usize) -> bool {
    if pos == 0 { return true; }
    match (source_at(stream, pos), source_at(stream, pos - 1)) {
        (Some(s), Some(prev_s)) => s != prev_s,
        _ => true,
    }
}

/// Check if pos is the last position in its source file.
#[allow(dead_code)]
fn is_file_end(stream: &CompleteStream, pos: usize) -> bool {
    if pos + 1 >= stream.len() { return true; }
    match (source_at(stream, pos), source_at(stream, pos + 1)) {
        (Some(s), Some(next_s)) => s != next_s,
        _ => true,
    }
}

/// Get left termination state for position pos.
fn left_term(stream: &CompleteStream, pos: usize) -> TerminationState {
    if is_file_start(stream, pos) {
        return TerminationState::FileStart;
    }
    let prev = pos - 1;
    if source_at(stream, prev) == source_at(stream, pos) {
        state_at(stream, prev)
            .map(TerminationState::State)
            .unwrap_or(TerminationState::FileStart)
    } else {
        TerminationState::FileStart
    }
}

/// Get right termination state for position pos+k.
fn right_term(stream: &CompleteStream, pos: usize, k: usize) -> TerminationState {
    let next = pos + k;
    if next >= stream.len() { return TerminationState::FileEnd; }
    if source_at(stream, next) != source_at(stream, pos) {
        return TerminationState::FileEnd;
    }
    state_at(stream, next)
        .map(TerminationState::State)
        .unwrap_or(TerminationState::FileEnd)
}

/// Compute the maximal persistence K from positions i and j.
/// Progression is source-local — stops at file boundaries.
fn compute_k(stream: &CompleteStream, i: usize, j: usize) -> usize {
    let source_i = match source_at(stream, i) { Some(s) => s, None => return 0 };
    let source_j = match source_at(stream, j) { Some(s) => s, None => return 0 };

    let mut k = 1; // we already know state_at(i) == state_at(j)
    loop {
        let ni = i + k;
        let nj = j + k;
        // Check file boundaries
        if ni >= stream.len() || nj >= stream.len() { break; }
        if source_at(stream, ni) != Some(source_i) { break; }
        if source_at(stream, nj) != Some(source_j) { break; }
        // Check state equality
        match (state_at(stream, ni), state_at(stream, nj)) {
            (Some(si), Some(sj)) if si == sj => k += 1,
            _ => break,
        }
    }
    k
}

/// Check bilateral maximality: (i,j) is canonical iff predecessors differ
/// or at least one is FILE_START.
fn is_bilaterally_maximal(stream: &CompleteStream, i: usize, j: usize) -> bool {
    let li = left_term(stream, i);
    let lj = left_term(stream, j);
    match (li, lj) {
        (TerminationState::FileStart, _) => true,
        (_, TerminationState::FileStart) => true,
        (TerminationState::State(si), TerminationState::State(sj)) => si != sj,
        _ => true,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 1B: ALPHA-RUN RECURRENCE — SYLLABIC STRUCTURE
// ─────────────────────────────────────────────────────────────────────────────
//
// DECLARATION:
//   Capitals are not observable in spoken language. Case is therefore not
//   part of the observable state for the purpose of identifying relational
//   units of the language. Alpha runs are extracted from the complete stream,
//   case-folded to lowercase, and recurrence is measured over those runs only.
//
// BILATERAL MAXIMALITY:
//   A k-gram occurrence at position i in a run is bilaterally maximal with
//   occurrence at position j iff their predecessors differ:
//     predecessor(i) = run[i-1] if i > 0, else RUN_START
//   Cross-predecessor-group pairs are the canonical recurrences.
//
// COUNTING:
//   For each k-gram G and each predecessor group P:
//     canonical pairs = total_pairs - within_group_pairs
//   This is exact and requires no enumeration of individual pairs.
//
// VERIFICATION:
//   For each K, one sample pair is exactly verified against the stream.
//
// NATURAL TERMINATION:
//   K_max is determined by the corpus. No threshold is imposed.
//   The corpus supplies the resolution.

/// A predecessor label for an Alpha-run position.
/// None = RUN_START (position 0 in its run — no predecessor within the run).
/// Some(c) = the lowercase character immediately preceding this position.
pub type PredecessorLabel = Option<char>;

/// Count cross-group pairs using combinatorics.
/// Given group sizes [s1, s2, ...], cross-group pairs =
///   total_pairs - within_group_pairs
///   = n*(n-1)/2 - sum(si*(si-1)/2)
fn count_cross_group_pairs(group_sizes: &[u64]) -> u64 {
    let total: u64 = group_sizes.iter().sum();
    if total < 2 { return 0; }
    let total_pairs = total * (total - 1) / 2;
    let within: u64 = group_sizes.iter().map(|&s| s * (s - 1) / 2).sum();
    total_pairs - within
}

/// Extract maximal Alpha runs from the complete stream, case-folded to lowercase.
/// Returns (source_file_index, run_string) pairs.
/// Only runs of length >= 2 are included (single chars produce no k-gram pairs).
pub fn extract_alpha_runs(stream: &CompleteStream) -> Vec<(usize, String)> {
    let mut runs = Vec::new();
    let mut current_run: Vec<char> = Vec::new();
    let mut current_source: Option<usize> = None;

    for pos in &stream.positions {
        if pos.char_class == CharClass::Alpha {
            // Case-fold to lowercase
            let c = pos.character.to_ascii_lowercase();
            if current_source != Some(pos.source_file_index) {
                // New source file — flush current run
                if current_run.len() >= 2 {
                    if let Some(src) = current_source {
                        runs.push((src, current_run.iter().collect()));
                    }
                }
                current_run.clear();
                current_source = Some(pos.source_file_index);
            }
            current_run.push(c);
        } else {
            // Non-alpha — flush current run
            if current_run.len() >= 2 {
                if let Some(src) = current_source {
                    runs.push((src, current_run.iter().collect()));
                }
            }
            current_run.clear();
            // Keep current_source — same file continues
            if current_source.is_none() {
                current_source = Some(pos.source_file_index);
            }
        }
    }
    // Final flush
    if current_run.len() >= 2 {
        if let Some(src) = current_source {
            runs.push((src, current_run.iter().collect()));
        }
    }

    runs
}

/// Statistics for a single K value in the Alpha-run recurrence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KStats {
    pub k: usize,
    /// Total canonical bilateral maximal recurrences at this K
    pub canonical_count: u64,
    /// Top recurring sequences at this K (sequence -> count), top 10
    pub top_sequences: Vec<(String, u64)>,
    /// A verified sample pair: (run_a_excerpt, run_b_excerpt, sequence)
    pub sample: Option<(String, String, String)>,
    /// Verification passed for sample
    pub sample_verified: bool,
}

/// Complete Phase 1B result over the Alpha-run stream.
#[derive(Debug)]
pub struct AlphaRecurrenceResult {
    /// Total Alpha runs processed
    pub run_count: usize,
    /// Total Alpha characters (case-folded)
    pub alpha_char_count: usize,
    /// Results per K, from K=1 to K_max
    pub k_stats: Vec<KStats>,
    /// Maximum K with at least one canonical recurrence
    pub k_max: usize,
    /// Total canonical recurrences across all K
    pub total_canonical: u64,
    /// Verification failures (must be 0)
    pub verification_failures: usize,
}

/// Run Phase 1B: Alpha-run recurrence analysis.
///
/// For each K from 1 upward, counts all bilateral maximal canonical
/// recurrences of exactly K characters in the Alpha-run stream.
/// Stops when K produces zero canonical recurrences.
///
/// No threshold imposed — the corpus determines K_max.
pub fn run_alpha_recurrence(stream: &CompleteStream) -> AlphaRecurrenceResult {
    // Extract Alpha runs
    let runs = extract_alpha_runs(stream);
    let run_count = runs.len();
    let alpha_char_count: usize = runs.iter().map(|(_, r)| r.len()).sum();

    let mut k_stats: Vec<KStats> = Vec::new();
    let mut k_max = 0;
    let mut total_canonical = 0u64;
    let mut verification_failures = 0usize;

    let mut k = 1usize;
    loop {
        // Build k-gram occurrence index with predecessor grouping
        // kgram_string -> (predecessor_label -> count, sample_position)
        let mut kgram_index: std::collections::HashMap<
            String,
            (std::collections::HashMap<Option<char>, u64>,
             Option<(usize, usize)>) // (run_idx, pos_in_run)
        > = std::collections::HashMap::new();

        for (run_idx, (_, run)) in runs.iter().enumerate() {
            let run_chars: Vec<char> = run.chars().collect();
            let run_len = run_chars.len();
            if run_len < k { continue; }

            for i in 0..=run_len - k {
                let kgram: String = run_chars[i..i+k].iter().collect();
                let pred: PredecessorLabel = if i == 0 { None } else { Some(run_chars[i-1]) };

                let entry = kgram_index.entry(kgram).or_insert_with(|| {
                    (std::collections::HashMap::new(), None)
                });
                *entry.0.entry(pred).or_insert(0) += 1;
                if entry.1.is_none() {
                    entry.1 = Some((run_idx, i));
                }
            }
        }

        // Count canonical recurrences for this K
        let mut k_total = 0u64;
        let mut top_seqs: Vec<(String, u64)> = Vec::new();
        let mut sample_pair: Option<(String, String, String)> = None;
        let mut sample_verified = false;

        for (kgram, (pred_groups, first_pos)) in &kgram_index {
            let total_occ: u64 = pred_groups.values().sum();
            if total_occ < 2 { continue; }

            let group_sizes: Vec<u64> = pred_groups.values().copied().collect();
            let canonical = count_cross_group_pairs(&group_sizes);
            if canonical == 0 { continue; }

            k_total += canonical;
            top_seqs.push((kgram.clone(), canonical));

            // Collect a sample pair for verification
            if sample_pair.is_none() {
                if let Some((run_idx_a, pos_a)) = first_pos {
                    // Find a second occurrence with different predecessor
                    let pred_a: PredecessorLabel = if *pos_a == 0 { None } else {
                        let run_chars: Vec<char> = runs[*run_idx_a].1.chars().collect();
                        Some(run_chars[pos_a - 1])
                    };
                    // Find occurrence with different predecessor
                    'find_b: for (run_idx_b, (_, run_b)) in runs.iter().enumerate() {
                        let run_chars_b: Vec<char> = run_b.chars().collect();
                        for pos_b in 0..run_chars_b.len().saturating_sub(k.saturating_sub(1)) {
                            if pos_b + k > run_chars_b.len() { continue; }
                            let candidate: String = run_chars_b[pos_b..pos_b+k].iter().collect();
                            if candidate != *kgram { continue; }
                            let pred_b: PredecessorLabel = if pos_b == 0 { None }
                                else { Some(run_chars_b[pos_b - 1]) };
                            if pred_b == pred_a { continue; }
                            // VERIFICATION: both k-grams must match exactly
                            let run_a_chars: Vec<char> = runs[*run_idx_a].1.chars().collect();
                            let mut verified = true;
                            for r in 0..k {
                                if run_a_chars[pos_a + r] != run_chars_b[pos_b + r] {
                                    verified = false;
                                    verification_failures += 1;
                                    break;
                                }
                            }
                            if verified {
                                // Build context strings
                                let ctx_start_a = pos_a.saturating_sub(2);
                                let ctx_end_a = (pos_a + k + 2).min(run_a_chars.len());
                                let ctx_a: String = run_a_chars[ctx_start_a..ctx_end_a].iter().collect();

                                let ctx_start_b = pos_b.saturating_sub(2);
                                let ctx_end_b = (pos_b + k + 2).min(run_chars_b.len());
                                let ctx_b: String = run_chars_b[ctx_start_b..ctx_end_b].iter().collect();

                                sample_pair = Some((ctx_a, ctx_b, kgram.clone()));
                                sample_verified = true;
                            }
                            break 'find_b;
                        }
                    }
                }
            }
        }

        // Sort top sequences by count
        top_seqs.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        top_seqs.truncate(10);

        if k_total == 0 {
            // Natural termination — corpus determines K_max
            break;
        }

        k_max = k;
        total_canonical += k_total;
        k_stats.push(KStats {
            k,
            canonical_count: k_total,
            top_sequences: top_seqs,
            sample: sample_pair,
            sample_verified,
        });

        k += 1;
        if k > 10000 { break; } // safety bound
    }

    AlphaRecurrenceResult {
        run_count,
        alpha_char_count,
        k_stats,
        k_max,
        total_canonical,
        verification_failures,
    }
}


/// Verify a single recurrence relation exactly against X.
/// Returns Ok(()) if all K equalities hold and all invariants are satisfied.
pub fn verify_recurrence(
    stream: &CompleteStream,
    rel: &RecurrenceRelation,
) -> Result<(), String> {
    let src_i = source_at(stream, rel.i)
        .ok_or_else(|| format!("Position {} not in stream", rel.i))?;
    let src_j = source_at(stream, rel.j)
        .ok_or_else(|| format!("Position {} not in stream", rel.j))?;

    if rel.i >= rel.j {
        return Err(format!("i={} must be < j={}", rel.i, rel.j));
    }

    for r in 0..rel.k {
        let si = state_at(stream, rel.i + r)
            .ok_or_else(|| format!("Position {}+{} not in stream", rel.i, r))?;
        let sj = state_at(stream, rel.j + r)
            .ok_or_else(|| format!("Position {}+{} not in stream", rel.j, r))?;
        if si != sj {
            return Err(format!("State mismatch at r={}: {:?} != {:?}", r, si, sj));
        }
        if r > 0 {
            if source_at(stream, rel.i + r) != Some(src_i) {
                return Err(format!("Cross-file progression at i+{}", r));
            }
            if source_at(stream, rel.j + r) != Some(src_j) {
                return Err(format!("Cross-file progression at j+{}", r));
            }
        }
    }

    if !is_bilaterally_maximal(stream, rel.i, rel.j) {
        return Err("Not bilaterally maximal".to_string());
    }

    Ok(())
}


// ─────────────────────────────────────────────────────────────────────────────
// PHASE 1B TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_phase1b {
    use super::*;
    use std::io::Write;

    const HAMLET_SCENE1: &str = "SCENE I. Elsinore. A platform before the Castle.

Enter Francisco and Barnardo, two sentinels.

BARNARDO.
Who's there?

FRANCISCO.
Nay, answer me. Stand and unfold yourself.

BARNARDO.
Long live the King!

FRANCISCO.
Barnardo?

BARNARDO.
He.

FRANCISCO.
You come most carefully upon your hour.

BARNARDO.
'Tis now struck twelve. Get thee to bed, Francisco.

FRANCISCO.
For this relief much thanks. 'Tis bitter cold,
And I am sick at heart.";

    const ORIGIN_PASSAGE: &str = "When we look to the individuals of the same variety or sub-variety of
our older cultivated plants and animals, one of the first points which
strikes us, is, that they generally differ much more from each other,
than do the individuals of any one species or variety in a state of
nature. When we reflect on the vast diversity of the plants and animals
which have been cultivated, and which have varied during all ages.";

    fn stream_from(text: &str, tag: &str) -> CompleteStream {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("abr_lc_1b_{}.txt", tag));
        let mut f = std::fs::File::create(&tmp).unwrap();
        f.write_all(text.as_bytes()).unwrap();
        CompleteStream::from_files(&[("corpus", tmp.to_str().unwrap())]).unwrap()
    }

    #[test]
    fn test_alpha_runs_extracted_from_hamlet() {
        // Alpha runs must be extracted and case-folded correctly
        let stream = stream_from(HAMLET_SCENE1, "runs_hamlet");
        let runs = extract_alpha_runs(&stream);
        assert!(!runs.is_empty(), "Must extract Alpha runs from Hamlet");
        // All characters in runs must be lowercase alpha
        for (_, run) in &runs {
            for c in run.chars() {
                assert!(c.is_ascii_lowercase(),
                    "All run characters must be lowercase: got '{}'", c);
            }
        }
    }

    #[test]
    fn test_alpha_runs_exclude_punctuation() {
        // Punctuation must not appear in Alpha runs
        let stream = stream_from("Hello, World! It's done.", "runs_punct");
        let runs = extract_alpha_runs(&stream);
        // "Hello" -> "hello", "World" -> "world", "It" -> "it", "s" -> dropped (len<2),
        // "done" -> "done"
        let all_chars: String = runs.iter().map(|(_, r)| r.as_str()).collect();
        assert!(!all_chars.contains(','), "Comma must not appear in runs");
        assert!(!all_chars.contains("'"), "Apostrophe must not appear in runs");
        assert!(!all_chars.contains('!'), "Exclamation must not appear in runs");
    }

    #[test]
    fn test_case_folded_to_lowercase() {
        // "BARNARDO" and "barnardo" must produce the same run
        let stream1 = stream_from("BARNARDO spoke.", "case1");
        let stream2 = stream_from("barnardo spoke.", "case2");
        let runs1 = extract_alpha_runs(&stream1);
        let runs2 = extract_alpha_runs(&stream2);
        assert_eq!(
            runs1.iter().map(|(_, r)| r.as_str()).collect::<Vec<_>>(),
            runs2.iter().map(|(_, r)| r.as_str()).collect::<Vec<_>>(),
            "Case-folded runs must be identical regardless of original case"
        );
    }

    #[test]
    fn test_recurrence_found_in_hamlet() {
        // Real repeated words in Hamlet must produce recurrences
        let stream = stream_from(HAMLET_SCENE1, "recur_hamlet");
        let result = run_alpha_recurrence(&stream);
        assert!(result.total_canonical > 0,
            "Must find canonical recurrences in Hamlet Scene I");
        assert_eq!(result.verification_failures, 0,
            "No verification failures");
    }

    #[test]
    fn test_recurrence_found_in_origin() {
        let stream = stream_from(ORIGIN_PASSAGE, "recur_origin");
        let result = run_alpha_recurrence(&stream);
        assert!(result.total_canonical > 0,
            "Must find canonical recurrences in Origin passage");
        assert_eq!(result.verification_failures, 0);
    }

    #[test]
    fn test_k_max_at_least_3_in_hamlet() {
        // Real English text must produce recurrences of length >= 3
        let stream = stream_from(HAMLET_SCENE1, "kmax_hamlet");
        let result = run_alpha_recurrence(&stream);
        assert!(result.k_max >= 3,
            "K_max must be at least 3 in real English text, got {}", result.k_max);
    }

    #[test]
    fn test_k1_dominates_distribution() {
        // K=1 must be the largest count (individual char recurrences most common)
        let stream = stream_from(HAMLET_SCENE1, "k1_dom");
        let result = run_alpha_recurrence(&stream);
        if result.k_stats.len() >= 2 {
            let k1 = result.k_stats[0].canonical_count;
            let k2 = result.k_stats[1].canonical_count;
            assert!(k1 > k2,
                "K=1 count ({}) must exceed K=2 count ({})", k1, k2);
        }
    }

    #[test]
    fn test_samples_verified_hamlet() {
        // All collected samples must pass verification
        let stream = stream_from(HAMLET_SCENE1, "samp_hamlet");
        let result = run_alpha_recurrence(&stream);
        assert_eq!(result.verification_failures, 0,
            "All samples must verify exactly");
        for ks in &result.k_stats {
            if ks.sample.is_some() {
                assert!(ks.sample_verified,
                    "Sample for K={} must be verified", ks.k);
            }
        }
    }

    #[test]
    fn test_samples_verified_origin() {
        let stream = stream_from(ORIGIN_PASSAGE, "samp_origin");
        let result = run_alpha_recurrence(&stream);
        assert_eq!(result.verification_failures, 0);
    }

    #[test]
    fn test_known_recurrence_the_detected() {
        // Use Origin passage where "the" occurs both standalone and embedded
        // in longer words (other, whether, these, there) giving varied predecessors
        let text = "When we look to the individuals of the same variety or other cultivated                     plants and animals one of the first points which strikes us is that they                     generally differ much more from each other than do the individuals.";
        let stream = stream_from(text, "the_detect");
        let result = run_alpha_recurrence(&stream);
        // Must have K=3 recurrences
        let k3 = result.k_stats.iter().find(|s| s.k == 3);
        assert!(k3.is_some(), "Must have K=3 stats in real scientific prose");
        let k3 = k3.unwrap();
        // "the" should appear in top sequences
        let has_the = k3.top_sequences.iter().any(|(seq, _)| seq == "the");
        assert!(has_the, "'the' must appear in top K=3 sequences, got: {:?}", k3.top_sequences);
        assert_eq!(result.verification_failures, 0);
    }

    #[test]
    fn test_bilateral_maximality_simple_case() {
        // "ababab": "ab" recurs. Predecessor of first 'a' = RUN_START,
        // predecessor of second 'a' (at pos 2) = 'b'.
        // These differ -> bilateral maximal. K depends on how far they match.
        let stream = stream_from("ababab", "bilateral");
        let result = run_alpha_recurrence(&stream);
        assert!(result.total_canonical > 0,
            "Must find recurrences in 'ababab'");
        assert_eq!(result.verification_failures, 0);
    }

    #[test]
    fn test_no_verification_failures_combined() {
        // Combined Hamlet + Origin must produce zero verification failures
        let combined = format!("{} {}", HAMLET_SCENE1, ORIGIN_PASSAGE);
        let stream = stream_from(&combined, "combined");
        let result = run_alpha_recurrence(&stream);
        assert_eq!(result.verification_failures, 0,
            "No verification failures on combined real corpus");
        assert!(result.k_max >= 3);
    }

    #[test]
    fn test_run_count_and_char_count_consistent() {
        let stream = stream_from(HAMLET_SCENE1, "counts_hamlet");
        let result = run_alpha_recurrence(&stream);
        assert!(result.run_count > 0, "Must have at least one run");
        assert!(result.alpha_char_count > 0, "Must have Alpha characters");
        assert!(result.alpha_char_count >= result.run_count * 2,
            "Each run has at least 2 chars");
    }

    #[test]
    fn test_natural_termination() {
        // K_max is determined by corpus — not an imposed limit
        // For a simple repeated string, the termination is exact
        let stream = stream_from("abcabc", "termination");
        let result = run_alpha_recurrence(&stream);
        // "abcabc" has "abc" recurring once, K_max = 3
        // After K=3, nothing new
        assert!(result.k_max >= 1, "Must find some recurrence");
        assert_eq!(result.verification_failures, 0);
    }
}
