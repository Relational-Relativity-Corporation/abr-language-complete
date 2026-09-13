// abr-language-complete  main.rs
// V0.1.0 — Phase 0: Complete Observable Stream
// Origin: Robin Macomber / Metatron Dynamics

use abr_language_complete::{CompleteStream, verify_completeness};
use std::fs;
use std::path::Path;

const CORPUS_DIR: &str = r"C:\Users\Robin Macomber\Documents\Metatron_Dynamics\GitHub_Repos\abr-language-complete\corpus";
const RESULTS_DIR: &str = r"C:\Users\Robin Macomber\Documents\Metatron_Dynamics\GitHub_Repos\abr-language-complete\results";

const CORPUS_FILES: &[&str] = &[
    "hamlet.txt",
    "origin_of_species.txt",
    "pride_and_prejudice.txt",
    "wikipedia.txt",
];

fn main() {
    println!("=== abr-language-complete V0.1.0 ===");
    println!("Phase 0: Complete Observable Stream");
    println!("Completeness invariant: ∀ x_i ∈ X, x_i remains represented.");
    println!("Reconstruction invariant: R⁻¹(R(X)) = X");
    println!();

    fs::create_dir_all(RESULTS_DIR).expect("Cannot create results dir");

    // ── Load corpus files ──────────────────────────────────────────────────
    println!("[1] Loading corpus files...");
    let mut file_paths: Vec<(String, String)> = Vec::new();
    let mut original_texts: Vec<(String, String)> = Vec::new();

    for fname in CORPUS_FILES {
        let full_path = format!("{}\\{}", CORPUS_DIR, fname);
        match fs::read_to_string(&full_path) {
            Ok(text) => {
                println!("    {} — {} chars", fname, text.chars().count());
                file_paths.push((fname.to_string(), full_path.clone()));
                original_texts.push((fname.to_string(), text));
            }
            Err(e) => {
                eprintln!("    ERROR: Cannot read {}: {}", full_path, e);
                std::process::exit(1);
            }
        }
    }
    println!();

    // ── Build complete stream ──────────────────────────────────────────────
    println!("[2] Building complete observable stream...");
    let path_refs: Vec<(&str, &str)> = file_paths
        .iter()
        .map(|(n, p)| (n.as_str(), p.as_str()))
        .collect();

    let stream = CompleteStream::from_files(&path_refs)
        .unwrap_or_else(|e| { eprintln!("Stream error: {}", e); std::process::exit(1); });

    println!("    Total positions: {}", stream.len());
    println!();

    // ── Verify completeness ────────────────────────────────────────────────
    println!("[3] Verifying completeness and reconstruction invariants...");
    let orig_refs: Vec<(&str, String)> = original_texts
        .iter()
        .map(|(n, t)| (n.as_str(), t.clone()))
        .collect();

    let report = verify_completeness(&stream, &orig_refs);

    println!("    Stream length:        {}", report.stream_length);
    println!("    Total file chars:     {}", report.total_file_chars);
    println!("    Lengths match:        {}", if report.lengths_match { "PASS" } else { "FAIL" });
    println!("    Reconstruction:       {}", if report.reconstruction_matches { "PASS ✓ R⁻¹(R(X))=X" } else { "FAIL" });
    println!("    All classes covered:  {}", if report.all_classes_covered { "PASS" } else { "FAIL" });

    if !report.missing_classes.is_empty() {
        println!("    Missing classes: {:?}", report.missing_classes);
    }
    println!();

    // ── Per-file report ────────────────────────────────────────────────────
    println!("[4] Per-file verification:");
    for fr in &report.file_reports {
        let status = if fr.reconstruction_matches { "PASS" } else { "FAIL" };
        println!("    {} — declared: {} chars, stream: {} chars, reconstruction: {}",
            fr.name, fr.declared_length, fr.stream_length, status);
    }
    println!();

    // ── Character class distribution ───────────────────────────────────────
    println!("[5] Character class distribution:");
    let mut classes: Vec<(String, usize)> = report.class_counts.into_iter().collect();
    classes.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (class, count) in &classes {
        let pct = 100.0 * (*count as f64) / (report.stream_length as f64);
        println!("    {:15} {:>8}  ({:.2}%)", class, count, pct);
    }
    println!();

    // ── Case state distribution ────────────────────────────────────────────
    println!("[6] Case state distribution:");
    let mut cases: Vec<(String, usize)> = report.case_counts.into_iter().collect();
    cases.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (case, count) in &cases {
        let pct = 100.0 * (*count as f64) / (report.stream_length as f64);
        println!("    {:15} {:>8}  ({:.2}%)", case, count, pct);
    }
    println!();

    // ── Write results ──────────────────────────────────────────────────────
    println!("[7] Writing stream report...");
    let report_path = format!("{}\\stream_report_v0.1.0.txt", RESULTS_DIR);
    let mut out = String::new();
    out.push_str("# abr-language-complete — Stream Report V0.1.0\n");
    out.push_str("# Phase 0: Complete Observable Stream\n\n");
    out.push_str(&format!("Stream length:       {}\n", report.stream_length));
    out.push_str(&format!("Total file chars:    {}\n", report.total_file_chars));
    out.push_str(&format!("Lengths match:       {}\n", report.lengths_match));
    out.push_str(&format!("Reconstruction:      {}\n", report.reconstruction_matches));
    out.push_str(&format!("All classes covered: {}\n\n", report.all_classes_covered));

    out.push_str("Character class distribution:\n");
    for (class, count) in &classes {
        let pct = 100.0 * (*count as f64) / (report.stream_length as f64);
        out.push_str(&format!("  {:15} {:>8}  ({:.2}%)\n", class, count, pct));
    }

    out.push_str("\nCase state distribution:\n");
    for (case, count) in &cases {
        let pct = 100.0 * (*count as f64) / (report.stream_length as f64);
        out.push_str(&format!("  {:15} {:>8}  ({:.2}%)\n", case, count, pct));
    }

    out.push_str("\nPer-file verification:\n");
    for fr in &report.file_reports {
        out.push_str(&format!("  {} — {} chars — reconstruction: {}\n",
            fr.name, fr.declared_length,
            if fr.reconstruction_matches { "PASS" } else { "FAIL" }));
    }

    fs::write(&report_path, out).expect("Cannot write stream report");
    println!("    Written: {}", report_path);

    // ── Final verdict ──────────────────────────────────────────────────────
    println!();
    let pass = report.lengths_match && report.reconstruction_matches;
    if pass {
        println!("═══════════════════════════════════════════");
        println!("  PHASE 0 COMPLETE — INVARIANTS HOLD");
        println!("  ∀ x_i ∈ X: represented ✓");
        println!("  R⁻¹(R(X)) = X ✓");
        println!("  Zero unaccounted positions ✓");
        println!("═══════════════════════════════════════════");
    } else {
        eprintln!("PHASE 0 FAILED — invariant violation detected");
        std::process::exit(1);
    }
}

// Allow Reverse for sort
use std::cmp::Reverse;
