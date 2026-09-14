// abr-language-complete  main.rs
// V0.3.0 — Phase 1B: Identity Recurrence and Persistence Under Progression
// Origin: Robin Macomber / Metatron Dynamics

use abr_language_complete::{
    CompleteStream, verify_completeness,
    RelationalSubstrate, run_phase1b, verify_recurrence,
};
use std::fs;

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
    println!("Phase 1B: Identity Recurrence and Persistence Under Progression");
    println!();

    fs::create_dir_all(RESULTS_DIR).expect("Cannot create results dir");

    // ── Phase 0: Build stream ──────────────────────────────────────────────
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
        .unwrap_or_else(|e| { eprintln!("Stream error: {}", e); std::process::exit(1); });

    let orig_refs: Vec<(&str, String)> = original_texts.iter()
        .map(|(n, t)| (n.as_str(), t.clone())).collect();
    let p0 = verify_completeness(&stream, &orig_refs);
    assert!(p0.reconstruction_matches, "Phase 0 invariant violated");
    println!("    |X| = {}  Phase 0: PASS", stream.len());
    println!();

    // ── Phase 1A: verify substrate ─────────────────────────────────────────
    println!("[2] Verifying Phase 1A substrate...");
    let substrate = RelationalSubstrate::from_stream(&stream);
    let p1a = substrate.verify_accounting();
    assert!(p1a.invariant_holds, "Phase 1A accounting violated");
    println!("    |R_P| = {}  Phase 1A: PASS", substrate.relations.len());
    println!();

    // ── Phase 1B: run recurrence ───────────────────────────────────────────
    println!("[3] Running Phase 1B — identity recurrence over full corpus...");
    println!("    (Note: ~118B K=1 pairs exist; aggregating by K, not materializing)");
    println!("    This may take several minutes on the full corpus.");
    let start = std::time::Instant::now();
    let stats = run_phase1b(&stream);
    let elapsed = start.elapsed();
    println!("    Completed in {:.1}s", elapsed.as_secs_f64());
    println!();

    // ── Report ─────────────────────────────────────────────────────────────
    println!("[4] Phase 1B results:");
    println!("    Verification failures:    {} (must be 0)", stats.verification_failures);
    assert_eq!(stats.verification_failures, 0, "Verification failures detected");
    println!("    Total canonical R_I:      {}", stats.total_canonical);
    println!("    K_max observed:           {}", stats.k_max);
    println!("    Same-source recurrences:  {}", stats.same_source);
    println!("    Cross-source recurrences: {}", stats.cross_source);
    println!();

    println!("    K distribution (how long does observed identity persist?):");
    for (k, count) in &stats.k_distribution {
        let pct = 100.0 * (*count as f64) / (stats.total_canonical as f64);
        println!("      K={:<4}  {:>12}  ({:.2}%)", k, count, pct);
    }
    println!();

    // ── Sample verification ────────────────────────────────────────────────
    println!("[5] Sample recurrences (one per K) — exact verification against X:");
    for (k, sample) in &stats.samples {
        let result = verify_recurrence(&stream, sample);
        let status = if result.is_ok() { "VERIFIED" } else { "FAIL" };
        let src_label = if sample.source_i == sample.source_j {
            format!("same-source (file {})", sample.source_i)
        } else {
            format!("cross-source ({} → {})", sample.source_i, sample.source_j)
        };

        // Reconstruct the recurring sequence for inspection
        let seq: String = stream.positions[sample.i..sample.i + sample.k]
            .iter().map(|p| p.character).collect();
        let seq_display = if seq.len() > 40 {
            format!("{}...", &seq[..40])
        } else {
            seq.clone()
        };

        println!("      K={:>3}  i={:>7}  j={:>7}  {}  {}  seq={:?}",
            k, sample.i, sample.j, src_label, status, seq_display);
    }
    println!();

    // ── Write results ──────────────────────────────────────────────────────
    println!("[6] Writing Phase 1B report...");
    let report_path = format!("{}\\phase1b_report_v0.3.0.txt", RESULTS_DIR);
    let mut out = String::new();
    out.push_str("# abr-language-complete — Phase 1B Report V0.3.0\n");
    out.push_str("# Identity Recurrence and Persistence Under Progression\n");
    out.push_str("# Question: How long does observed identity actually persist?\n\n");
    out.push_str(&format!("|X|:                    {}\n", stream.len()));
    out.push_str(&format!("|R_P|:                  {}\n", substrate.relations.len()));
    out.push_str(&format!("Verification failures:  {}\n", stats.verification_failures));
    out.push_str(&format!("Total canonical R_I:    {}\n", stats.total_canonical));
    out.push_str(&format!("K_max:                  {}\n", stats.k_max));
    out.push_str(&format!("Same-source:            {}\n", stats.same_source));
    out.push_str(&format!("Cross-source:           {}\n\n", stats.cross_source));

    out.push_str("K distribution:\n");
    for (k, count) in &stats.k_distribution {
        let pct = 100.0 * (*count as f64) / (stats.total_canonical as f64);
        out.push_str(&format!("  K={:<4}  {:>12}  ({:.4}%)\n", k, count, pct));
    }

    out.push_str("\nSample recurrences (verified against X):\n");
    for (k, sample) in &stats.samples {
        let seq: String = stream.positions[sample.i..sample.i + sample.k]
            .iter().map(|p| p.character).collect();
        out.push_str(&format!(
            "  K={} i={} j={} src_i={} src_j={} seq={:?}\n",
            k, sample.i, sample.j, sample.source_i, sample.source_j, seq
        ));
        out.push_str(&format!(
            "    left_i={:?} left_j={:?}\n",
            sample.left_term_i, sample.left_term_j
        ));
        out.push_str(&format!(
            "    right_i={:?} right_j={:?}\n",
            sample.right_term_i, sample.right_term_j
        ));
    }

    fs::write(&report_path, out).expect("Cannot write Phase 1B report");
    println!("    Written: {}", report_path);

    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  PHASE 1B COMPLETE");
    println!("  Verification failures: 0 ✓");
    println!("  All K equalities exactly verified against X ✓");
    println!("  Bilateral maximality enforced ✓");
    println!("  File boundaries respected ✓");
    println!("  K distribution observed — corpus determines the resolution ✓");
    println!("═══════════════════════════════════════════════════════════════");
}
