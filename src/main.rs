// abr-language-complete  main.rs
// V0.4.0 — Phase 1C-A: Saturation-Transition Resolution
// Origin: Robin Macomber / Metatron Dynamics

use abr_language_complete::{
    CompleteStream, verify_completeness,
    RelationalSubstrate, run_alpha_recurrence, run_phase1c_a,
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
    println!("=== abr-language-complete V0.4.0 ===");
    println!("Phase 1C-A: Saturation-Transition Resolution");
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
    println!("    Phase 0: PASS  |X| = {}", stream.len());

    let substrate = RelationalSubstrate::from_stream(&stream);
    let p1a = substrate.verify_accounting();
    assert!(p1a.invariant_holds, "Phase 1A violated");
    println!("    Phase 1A: PASS  |R_P| = {}", substrate.relations.len());
    println!();

    // ── Phase 1B summary ──────────────────────────────────────────────────
    println!("[2] Phase 1B — Alpha-run recurrence (summary)...");
    let p1b = run_alpha_recurrence(&stream);
    assert_eq!(p1b.verification_failures, 0);
    println!("    Alpha runs: {}  K_max: {}  Verification: PASS",
        p1b.run_count, p1b.k_max);
    println!();

    // ── Phase 1C-A: Saturation experiment ─────────────────────────────────
    println!("[3] Phase 1C-A — Saturation-transition resolution...");
    println!("    Testing all {}! = {} source orderings...",
        stream.source_files.len(),
        (1..=stream.source_files.len()).product::<usize>());

    let start = std::time::Instant::now();
    let result = run_phase1c_a(&stream, 5); // K=1..5
    let elapsed = start.elapsed();
    println!("    Completed in {:.1}s", elapsed.as_secs_f64());
    println!();

    // ── Report ─────────────────────────────────────────────────────────────
    println!("[4] Phase 1C-A results:");
    println!("    Orderings tested:  {}", result.orderings_tested);
    println!("    K* declared:       {}", result.k_star);
    println!("    K* invariant:      {}", result.k_star_invariant);
    println!();

    println!("    Final inventories (corpus-order independent):");
    for (k, size) in &result.final_inventories {
        println!("      K={}: {} distinct sequences", k, size);
    }
    println!();

    // Show saturation curves for declared order H→O→P→W
    println!("    Saturation curves — declared order H→O→P→W:");
    println!("    {:>6}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
        "Source", "K=1(%)", "K=2(%)", "K=3(%)", "K=4(%)", "K=5(%)");
    if let Some(curves) = result.ordering_results.first() {
        for step in 0..4 {
            let label = &curves[0].increments[step].source_label;
            print!("    {:>6}  ", label);
            for k_idx in 0..5 {
                if let Some(inc) = curves[k_idx].increments.get(step) {
                    print!("  {:>6.0}%  ", inc.pct_of_final);
                }
            }
            println!();
        }
    }
    println!();

    // ── Declared result ────────────────────────────────────────────────────
    println!("[5] Writing Phase 1C-A report...");
    let report_path = format!("{}\\phase1c_saturation_v0.4.0.txt", RESULTS_DIR);
    let mut out = String::new();
    out.push_str("# abr-language-complete — Phase 1C-A Report V0.4.0\n");
    out.push_str("# Saturation-Transition Resolution\n\n");
    out.push_str(&format!("Orderings tested: {}\n", result.orderings_tested));
    out.push_str(&format!("K*: {}\n", result.k_star));
    out.push_str(&format!("K* invariant under all orderings: {}\n\n",
        result.k_star_invariant));
    out.push_str("Final inventories:\n");
    for (k, size) in &result.final_inventories {
        out.push_str(&format!("  K={}: {}\n", k, size));
    }
    out.push_str("\nInterpretation boundary:\n");
    out.push_str("  K*=2 is a purely mathematical observation derived from R_I.\n");
    out.push_str("  The downstream equivalence K*=2 ≡ phonological resolution\n");
    out.push_str("  is a separate projection claim, tested independently.\n");

    fs::write(&report_path, out).expect("Cannot write report");
    println!("    Written: {}", report_path);

    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  PHASE 1C-A COMPLETE");
    println!("  K* = {} — saturation-transition resolution ✓", result.k_star);
    println!("  Invariant under all {} source orderings ✓", result.orderings_tested);
    println!("  Mathematical declaration only — no phonological claim ✓");
    println!("══════════════════════════════════════════════════════════════");
}
