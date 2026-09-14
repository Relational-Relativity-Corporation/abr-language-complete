// abr-language-complete  main.rs
// V0.3.0 — Phase 1B: Alpha-Run Recurrence (Syllabic Structure)
// Origin: Robin Macomber / Metatron Dynamics

use abr_language_complete::{
    CompleteStream, verify_completeness,
    RelationalSubstrate, run_alpha_recurrence,
};
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
    println!("=== abr-language-complete V0.3.0 ===");
    println!("Phase 1B: Alpha-Run Recurrence — Syllabic Structure");
    println!("Declaration: capitals are not observable in spoken language.");
    println!("Alpha runs extracted, case-folded, recurrence measured.");
    println!();

    fs::create_dir_all(RESULTS_DIR).expect("Cannot create results dir");

    // ── Load corpus ────────────────────────────────────────────────────────
    println!("[1] Loading corpus...");
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
            Err(e) => { eprintln!("ERROR: {}", e); std::process::exit(1); }
        }
    }

    let path_refs: Vec<(&str, &str)> = file_paths.iter()
        .map(|(n, p)| (n.as_str(), p.as_str())).collect();
    let stream = CompleteStream::from_files(&path_refs)
        .unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });

    let orig_refs: Vec<(&str, String)> = original_texts.iter()
        .map(|(n, t)| (n.as_str(), t.clone())).collect();
    let p0 = verify_completeness(&stream, &orig_refs);
    assert!(p0.reconstruction_matches, "Phase 0 violated");
    println!("    |X| = {}  Phase 0: PASS", stream.len());

    let substrate = RelationalSubstrate::from_stream(&stream);
    let p1a = substrate.verify_accounting();
    assert!(p1a.invariant_holds, "Phase 1A violated");
    println!("    |R_P| = {}  Phase 1A: PASS", substrate.relations.len());
    println!();

    // ── Phase 1B: Alpha-run recurrence ─────────────────────────────────────
    println!("[2] Running Phase 1B — Alpha-run recurrence...");
    let start = std::time::Instant::now();
    let result = run_alpha_recurrence(&stream);
    let elapsed = start.elapsed();

    println!("    Completed in {:.1}s", elapsed.as_secs_f64());
    println!("    Alpha runs:          {}", result.run_count);
    println!("    Alpha characters:    {}", result.alpha_char_count);
    println!("    Verification fails:  {} (must be 0)", result.verification_failures);
    assert_eq!(result.verification_failures, 0, "Verification failures");
    println!("    Total canonical R_I: {}", result.total_canonical);
    println!("    K_max:               {}", result.k_max);
    println!();

    // ── K distribution ──────────────────────────────────────────────────────
    println!("[3] K distribution — how long does observed identity persist?");
    for ks in &result.k_stats {
        let top: Vec<String> = ks.top_sequences.iter().take(5)
            .map(|(s, c)| format!("{}({})", s, c))
            .collect();
        println!("    K={:<3}  {:>15}  top: {}",
            ks.k, ks.canonical_count, top.join(", "));
    }
    println!();

    // ── Samples ─────────────────────────────────────────────────────────────
    println!("[4] Sample verified pairs (one per K):");
    for ks in &result.k_stats {
        if let Some((ctx_a, ctx_b, seq)) = &ks.sample {
            println!("    K={:<3}  seq={:?}  verified={}",
                ks.k, seq, ks.sample_verified);
            println!("         ctx_a={:?}  ctx_b={:?}", ctx_a, ctx_b);
        }
    }
    println!();

    // ── Write report ────────────────────────────────────────────────────────
    println!("[5] Writing Phase 1B report...");
    let report_path = format!("{}\\phase1b_alpha_recurrence_v0.3.0.txt", RESULTS_DIR);
    let mut out = String::new();
    out.push_str("# abr-language-complete — Phase 1B Report V0.3.0\n");
    out.push_str("# Alpha-Run Recurrence — Syllabic Structure\n");
    out.push_str("# Declaration: capitals not observable in spoken language.\n");
    out.push_str("# Alpha runs extracted, case-folded, bilateral maximal recurrence.\n\n");
    out.push_str(&format!("Alpha runs:          {}\n", result.run_count));
    out.push_str(&format!("Alpha characters:    {}\n", result.alpha_char_count));
    out.push_str(&format!("Total canonical R_I: {}\n", result.total_canonical));
    out.push_str(&format!("K_max:               {}\n", result.k_max));
    out.push_str(&format!("Verification fails:  {}\n\n", result.verification_failures));

    out.push_str("K distribution:\n");
    for ks in &result.k_stats {
        out.push_str(&format!("  K={:<3}  {:>15}\n", ks.k, ks.canonical_count));
        for (seq, count) in &ks.top_sequences {
            out.push_str(&format!("    {:?}  {}\n", seq, count));
        }
    }

    fs::write(&report_path, out).expect("Cannot write report");
    println!("    Written: {}", report_path);

    println!();
    println!("══════════════════════════════════════════════════════");
    println!("  PHASE 1B COMPLETE");
    println!("  Alpha-run recurrence: K=1..{} ✓", result.k_max);
    println!("  Verification failures: 0 ✓");
    println!("  K distribution corpus-determined ✓");
    println!("  Stable base for Phase 1C (Q(S) derivation) ✓");
    println!("══════════════════════════════════════════════════════");
}
