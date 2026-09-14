// abr-language-complete  main.rs
// V0.2.0 — Phase 1A: Primitive Relational Substrate
// Origin: Robin Macomber / Metatron Dynamics

use abr_language_complete::{CompleteStream, verify_completeness, RelationalSubstrate};
use std::fs;
use std::cmp::Reverse;

const CORPUS_DIR: &str = r"C:\Users\Robin Macomber\Documents\Metatron_Dynamics\GitHub_Repos\abr-language-complete\corpus";
const RESULTS_DIR: &str = r"C:\Users\Robin Macomber\Documents\Metatron_Dynamics\GitHub_Repos\abr-language-complete\results";

const CORPUS_FILES: &[&str] = &[
    "hamlet.txt",
    "origin_of_species.txt",
    "pride_and_prejudice.txt",
    "wikipedia.txt",
];

fn main() {
    println!("=== abr-language-complete V0.2.0 ===");
    println!("Phase 1A: Primitive Relational Substrate");
    println!("Transition is observed. Boundary is to be determined.");
    println!();

    fs::create_dir_all(RESULTS_DIR).expect("Cannot create results dir");

    // ── Phase 0: Load and verify complete stream ───────────────────────────
    println!("[1] Loading corpus and building complete stream...");
    let mut file_paths: Vec<(String, String)> = Vec::new();
    let mut original_texts: Vec<(String, String)> = Vec::new();

    for fname in CORPUS_FILES {
        let full_path = format!("{}\\{}", CORPUS_DIR, fname);
        match fs::read_to_string(&full_path) {
            Ok(text) => {
                println!("    {} — {} chars", fname, text.chars().count());
                file_paths.push((fname.to_string(), full_path));
                original_texts.push((fname.to_string(), text));
            }
            Err(e) => {
                eprintln!("    ERROR: Cannot read {}: {}", full_path, e);
                std::process::exit(1);
            }
        }
    }

    let path_refs: Vec<(&str, &str)> = file_paths
        .iter()
        .map(|(n, p)| (n.as_str(), p.as_str()))
        .collect();

    let stream = CompleteStream::from_files(&path_refs)
        .unwrap_or_else(|e| { eprintln!("Stream error: {}", e); std::process::exit(1); });

    println!("    Total positions |X|: {}", stream.len());

    // Verify Phase 0 invariants still hold
    let orig_refs: Vec<(&str, String)> = original_texts
        .iter()
        .map(|(n, t)| (n.as_str(), t.clone()))
        .collect();
    let phase0 = verify_completeness(&stream, &orig_refs);
    assert!(phase0.lengths_match, "Phase 0 length invariant violated");
    assert!(phase0.reconstruction_matches, "Phase 0 reconstruction invariant violated");
    println!("    Phase 0 invariants: PASS");
    println!();

    // ── Phase 1A: Derive relational substrate ─────────────────────────────
    println!("[2] Deriving Phase 1A relational substrate...");
    let substrate = RelationalSubstrate::from_stream(&stream);
    println!("    Relations derived: {}", substrate.relations.len());
    println!("    Source files (F):  {}", substrate.file_count);
    println!("    Expected |X| - F:  {}", stream.len() - substrate.file_count);
    println!();

    // ── Verify accounting invariant ────────────────────────────────────────
    println!("[3] Verifying accounting invariant |R_P| = |X| - F...");
    let report = substrate.verify_accounting();

    println!("    Expected: {}", report.expected_relations);
    println!("    Actual:   {}", report.actual_relations);
    println!("    Invariant holds:              {}",
        if report.invariant_holds { "PASS" } else { "FAIL" });
    println!("    Duplicate predecessor positions: {} (must be 0)",
        report.duplicate_predecessor_positions);
    println!("    Class transition loci (Δ_C):  {}", report.class_transition_loci);
    println!("    Case transition loci (Δ_E):   {}", report.case_transition_loci);
    println!();

    if !report.invariant_holds || report.duplicate_predecessor_positions > 0 {
        eprintln!("PHASE 1A FAILED — accounting invariant violated");
        std::process::exit(1);
    }

    // ── Class transition table ─────────────────────────────────────────────
    println!("[4] Class transition table R_C (top 30 by count):");
    let mut class_pairs: Vec<((String, String), usize)> =
        report.class_transition_table.into_iter().collect();
    class_pairs.sort_by_key(|(_, c)| Reverse(*c));
    for ((from, to), count) in class_pairs.iter().take(30) {
        let pct = 100.0 * (*count as f64) / (substrate.relations.len() as f64);
        println!("    {:15} → {:15}  {:>8}  ({:.2}%)", from, to, count, pct);
    }
    println!();

    // ── Case transition table ──────────────────────────────────────────────
    println!("[5] Case transition table R_E:");
    let mut case_pairs: Vec<((String, String), usize)> =
        report.case_transition_table.into_iter().collect();
    case_pairs.sort_by_key(|(_, c)| Reverse(*c));
    for ((from, to), count) in &case_pairs {
        let pct = 100.0 * (*count as f64) / (substrate.relations.len() as f64);
        println!("    {:15} → {:15}  {:>8}  ({:.2}%)", from, to, count, pct);
    }
    println!();

    // ── Write results ──────────────────────────────────────────────────────
    println!("[6] Writing Phase 1A report...");
    let report_path = format!("{}\\phase1a_report_v0.2.0.txt", RESULTS_DIR);
    let mut out = String::new();
    out.push_str("# abr-language-complete — Phase 1A Report V0.2.0\n");
    out.push_str("# Primitive Relational Substrate\n");
    out.push_str("# Transition is observed. Boundary is to be determined.\n\n");
    out.push_str(&format!("|X| (stream positions):    {}\n", stream.len()));
    out.push_str(&format!("F  (source files):         {}\n", substrate.file_count));
    out.push_str(&format!("|R_P| (relations):         {}\n", substrate.relations.len()));
    out.push_str(&format!("|X| - F (expected):        {}\n", report.expected_relations));
    out.push_str(&format!("Invariant |R_P|=|X|-F:     {}\n", report.invariant_holds));
    out.push_str(&format!("Duplicate predecessors:    {}\n", report.duplicate_predecessor_positions));
    out.push_str(&format!("Class transition loci Δ_C: {}\n", report.class_transition_loci));
    out.push_str(&format!("Case transition loci Δ_E:  {}\n\n", report.case_transition_loci));

    out.push_str("Class transition table R_C (class_from → class_to, count):\n");
    for ((from, to), count) in &class_pairs {
        let pct = 100.0 * (*count as f64) / (substrate.relations.len() as f64);
        out.push_str(&format!("  {:15} → {:15}  {}  ({:.3}%)\n", from, to, count, pct));
    }

    out.push_str("\nCase transition table R_E (case_from → case_to, count):\n");
    for ((from, to), count) in &case_pairs {
        let pct = 100.0 * (*count as f64) / (substrate.relations.len() as f64);
        out.push_str(&format!("  {:15} → {:15}  {}  ({:.3}%)\n", from, to, count, pct));
    }

    fs::write(&report_path, out).expect("Cannot write Phase 1A report");
    println!("    Written: {}", report_path);

    // ── Final verdict ──────────────────────────────────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("  PHASE 1A COMPLETE — ACCOUNTING INVARIANT HOLDS");
    println!("  |R_P| = |X| - F ✓");
    println!("  Every non-final position: exactly one successor ✓");
    println!("  Every adjacent pair: one R_C, one R_E observation ✓");
    println!("  Zero unaccounted pairs ✓");
    println!("  Transition observed. Boundary to be determined. ✓");
    println!("═══════════════════════════════════════════════════════");
}
