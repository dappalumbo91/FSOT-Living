//! Headless capability probe — no TUI. Writes LIVE_STATUS.json and prints battery results.

#[path = "../accuracy_homeo.rs"]
mod accuracy_homeo;
#[path = "../activity.rs"]
mod activity;
#[path = "../audio.rs"]
mod audio;
#[path = "../behavior_monitor.rs"]
mod behavior_monitor;
#[path = "../bench.rs"]
mod bench;
#[path = "../body_bridge.rs"]
mod body_bridge;
#[path = "../capability.rs"]
mod capability;
#[path = "../code_env.rs"]
mod code_env;
#[path = "../constraints.rs"]
mod constraints;
#[path = "../export.rs"]
mod export;
#[path = "../fsot_tool.rs"]
mod fsot_tool;
#[path = "../genetic.rs"]
mod genetic;
#[path = "../habitat.rs"]
mod habitat;
#[path = "../homeostasis.rs"]
mod homeostasis;
#[path = "../knowledge.rs"]
mod knowledge;
#[path = "../lang_plastic.rs"]
mod lang_plastic;
#[path = "../language.rs"]
mod language;
#[path = "../lean_ground.rs"]
mod lean_ground;
#[path = "../organism.rs"]
mod organism;
#[path = "../pack.rs"]
mod pack;
#[path = "../proposals.rs"]
mod proposals;
#[path = "../query.rs"]
mod query;
#[path = "../resume.rs"]
mod resume;
#[path = "../scalar.rs"]
mod scalar;
#[path = "../self_model.rs"]
mod self_model;
#[path = "../video.rs"]
mod video;
#[path = "../vision.rs"]
mod vision;

use std::io::Write;
use std::time::Instant;

fn flush() {
    let _ = std::io::stdout().flush();
}

fn main() {
    println!("FSOT headless probe starting…");
    flush();
    let mut h = habitat::Habitat::new(7);
    let t0 = Instant::now();
    // FSOT_PROBE_TICKS=80 for fast integrity smoke; default 500
    let ticks: u64 = std::env::var("FSOT_PROBE_TICKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500)
        .max(10);
    println!("probe ticks={ticks}");
    flush();

    for i in 0..ticks {
        h.step();
        // Progress always (redirected stdout is fully buffered otherwise)
        if i == 0 || (i + 1) % 5 == 0 || i + 1 == ticks {
            println!(
                "…tick {}/{} py_budget_left={}",
                i + 1,
                ticks,
                code_env::python_budget_remaining()
            );
            flush();
        }
    }
    let _ = export::write_live(&h);
    println!("habitat done in {:.1}s", t0.elapsed().as_secs_f64());
    flush();

    let hero = h.hero();
    println!("\n=== AFTER 500 TICKS ===");
    println!(
        "tick={} gen={} fed={} exams={} births={}",
        h.tick, h.generation, h.fed, h.exams, h.births
    );
    println!(
        "body online={} couple={:.3} motors={} genome={}",
        h.body.online,
        h.body.couple_pressure,
        h.body.motors_sent,
        h.body.status.genome_fp
    );
    println!("{}", body_bridge::body_summary(&h));
    println!(
        "hero={} S={:.4} regime={} acc={:.1}% lang={:.1}% intel={:.3} fit={:.3}",
        hero.id,
        hero.s,
        hero.regime,
        hero.cognitive_accuracy * 100.0,
        hero.language_accuracy * 100.0,
        hero.intelligence,
        hero.fitness_score()
    );
    println!(
        "code_acc={:.0}% vision_acc={:.0}% lang_skill={:.2} semantic={} tools={:?}",
        hero.code_accuracy * 100.0,
        hero.vision_accuracy * 100.0,
        hero.language_skill,
        hero.semantic.len(),
        hero.tools_owned
    );
    // JS organ mastery (target ≥85%)
    let js_mean = h.pack.iter().map(|o| o.js_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let js_min = h
        .pack
        .iter()
        .map(|o| o.js_accuracy)
        .fold(1.0_f64, f64::min);
    let js_ok = h.pack.iter().filter(|o| o.js_accuracy >= code_env::JS_TARGET).count();
    println!(
        "js_acc hero={:.0}% pack mean={:.0}% min={:.0}% ≥85%={}/{} node={}",
        hero.js_accuracy * 100.0,
        js_mean * 100.0,
        js_min * 100.0,
        js_ok,
        h.pack.len(),
        code_env::node_available()
    );
    let k_mean = h.pack.iter().map(|o| o.knowledge_accuracy).sum::<f64>()
        / h.pack.len().max(1) as f64;
    let k_min = h
        .pack
        .iter()
        .map(|o| o.knowledge_accuracy)
        .fold(1.0_f64, f64::min);
    println!(
        "knowledge hero={:.0}% pack mean={:.0}% min={:.0}% (target 85%)",
        hero.knowledge_accuracy * 100.0,
        k_mean * 100.0,
        k_min * 100.0
    );
    let g_mean = h
        .pack
        .iter()
        .map(|o| o.lang_cortex.generalize_accuracy)
        .sum::<f64>()
        / h.pack.len().max(1) as f64;
    let g_min = h
        .pack
        .iter()
        .map(|o| o.lang_cortex.generalize_accuracy)
        .fold(1.0_f64, f64::min);
    let a_mean = h.pack.iter().map(|o| o.audio_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let a_min = h.pack.iter().map(|o| o.audio_accuracy).fold(1.0_f64, f64::min);
    let v_mean = h.pack.iter().map(|o| o.video_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let v_min = h.pack.iter().map(|o| o.video_accuracy).fold(1.0_f64, f64::min);
    let sm_mean =
        h.pack.iter().map(|o| o.self_model_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let sm_min = h
        .pack
        .iter()
        .map(|o| o.self_model_accuracy)
        .fold(1.0_f64, f64::min);
    println!(
        "gen_acc hero={:.0}% pack mean={:.0}% min={:.0}% (target 75%)",
        hero.lang_cortex.generalize_accuracy * 100.0,
        g_mean * 100.0,
        g_min * 100.0
    );
    println!(
        "audio mean={:.0}% min={:.0}% | video mean={:.0}% min={:.0}% (target 75%)",
        a_mean * 100.0,
        a_min * 100.0,
        v_mean * 100.0,
        v_min * 100.0
    );
    println!(
        "self_model mean={:.0}% min={:.0}% (target 70%)",
        sm_mean * 100.0,
        sm_min * 100.0
    );
    let b_mean =
        h.pack.iter().map(|o| o.bench_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let b_min = h
        .pack
        .iter()
        .map(|o| o.bench_accuracy)
        .fold(1.0_f64, f64::min);
    let b_ok = h
        .pack
        .iter()
        .filter(|o| o.bench_accuracy >= bench::BENCH_TARGET)
        .count();
    let bz_mean =
        h.pack.iter().map(|o| o.bench_zs_accuracy).sum::<f64>() / h.pack.len().max(1) as f64;
    let bz_min = h
        .pack
        .iter()
        .map(|o| o.bench_zs_accuracy)
        .fold(1.0_f64, f64::min);
    let bz_ok = h
        .pack
        .iter()
        .filter(|o| o.bench_zs_accuracy >= bench::BENCH_ZS_TARGET)
        .count();
    println!(
        "bench study mean={:.0}% min={:.0}% ≥80%={}/{} bank={}",
        b_mean * 100.0,
        b_min * 100.0,
        b_ok,
        h.pack.len(),
        h.bench.items.len()
    );
    println!(
        "bench ZERO-SHOT mean={:.0}% min={:.0}% ≥70%={}/{} bank_zs={:.0}% · {}",
        bz_mean * 100.0,
        bz_min * 100.0,
        bz_ok,
        h.pack.len(),
        h.bench.zs_mean() * 100.0,
        h.bench.summary_line()
    );
    println!(
        "codons={}/64 mem={} lt={} mut+/-={}/{}",
        hero.codon_table.len(),
        hero.memory.len(),
        hero.long_term.len(),
        hero.accepted_mut,
        hero.rejected_mut
    );
    println!("AA={}", hero.guidance.aa_chain);
    println!("lineage_gens={}", h.lineage.len());
    // Drop immutable hero borrow before mut raw zs sweep
    let _ = hero;

    // Anti-fudge: fresh hard-count zs (not sticky pack EMA)
    println!("\n=== RAW ZS SNAPSHOT (hard counts, same-item blocked, code=test-exec) ===");
    let mut rng = rand::thread_rng();
    let raw = {
        let org = h.pack.first_mut().expect("pack non-empty");
        bench::raw_zs_snapshot(org, &h.bench, 10, &mut rng)
    };
    for s in &raw {
        println!(
            "  raw_zs {}: {:.1}% ({}/~{})",
            s.suite,
            s.accuracy * 100.0,
            (s.accuracy * s.n as f64).round() as u64,
            s.n
        );
    }
    let audit = bench::write_capability_audit_with_raw(h.tick, h.generation, &h.pack, &h.bench, raw);
    println!(
        "CAPABILITY_AUDIT zs_target={:.0}% stretch={:.0}% suites_zs={} raw_zs={:.1}% (n={})",
        audit.zs_target * 100.0,
        audit.zs_stretch * 100.0,
        audit.suites_zeroshot.len(),
        audit.integrity.raw_zs_accuracy * 100.0,
        audit.integrity.raw_zs_n_total
    );
    println!(
        "EVAL INTEGRITY: zs_blocks_same_item={} · code_zs_test_exec={} · study≠invent · small_n={}",
        audit.integrity.zs_blocks_same_item_engrams,
        audit.integrity.code_zs_requires_test_exec,
        audit.integrity.small_n_suites.len()
    );
    for n in &audit.integrity.notes {
        println!("  · {n}");
    }

    // Dialogue is a separate metabolic breath (must not inherit depleted raw_zs budget)
    code_env::reset_python_budget(32);

    // Command + freeform language battery
    let tests: &[(&str, Box<dyn Fn(&query::QueryResult) -> bool>)] = &[
        (
            "hello how are you",
            Box::new(|r| r.kind == "feeling" || r.kind == "greeting"),
        ),
        ("status", Box::new(|r| r.kind == "status" && r.answer.contains("S="))),
        (
            "codon ATG",
            Box::new(|r| r.kind == "codon" && r.answer.contains('M')),
        ),
        (
            "What amino acid does ATG code for?",
            Box::new(|r| r.kind == "codon" && r.answer.contains('M')),
        ),
        (
            "Tell me about the quantum domain",
            Box::new(|r| r.kind == "domain" && r.answer.contains("S=")),
        ),
        (
            "domain cosmological",
            Box::new(|r| r.kind == "domain"),
        ),
        ("genome", Box::new(|r| r.kind == "genome")),
        ("fit", Box::new(|r| r.kind == "fitness")),
        ("Who are you?", Box::new(|r| r.kind == "identity")),
        ("What can you do?", Box::new(|r| {
            r.kind == "capabilities" || r.kind == "help"
        })),
        ("What is 7+5?", Box::new(|r| r.answer.contains("12"))),
        ("Remember that my name is ProbeUser", Box::new(|r| r.kind == "remember")),
        ("What is my name?", Box::new(|r| {
            r.answer.to_lowercase().contains("probeuser")
        })),
        ("organs", Box::new(|r| r.kind == "organs")),
        ("py: 2+40", Box::new(|r| r.answer.contains("42"))),
        ("see", Box::new(|r| r.kind == "vision")),
        ("forge double", Box::new(|r| {
            r.answer.to_lowercase().contains("forge") || r.answer.contains("double")
        })),
        ("xyzzy nonsense", Box::new(|r| {
            r.kind == "unknown" || r.kind == "recall"
        })),
    ];

    println!("\n=== QUERY / LANGUAGE BATTERY ===");
    let mut pass = 0usize;
    for (q, check) in tests {
        let r = query::query(h.hero_mut(), q);
        let ok = check(&r);
        if ok {
            pass += 1;
        }
        let preview: String = r.answer.chars().take(110).collect::<String>().replace('\n', " | ");
        println!(
            "[{}] {} -> {} | {}",
            if ok { "PASS" } else { "FAIL" },
            q,
            r.kind,
            preview
        );
    }

    let q = scalar::compute_for_domain("quantum");
    let c = scalar::compute_for_domain("cosmological");
    let ai = scalar::compute_for_domain("ai_tech");
    println!(
        "\n=== SCALAR LEDGER (Lean-aligned) === quantum S={:.4} cosmo S={:.4} ai_tech S={:.4} k={:.6}",
        q.s,
        c.s,
        ai.s,
        scalar::constants::k()
    );
    let lean = lean_ground::check_runtime_invariants();
    println!("{}", lean.summary_line());
    for ch in &lean.checks {
        println!(
            "  lean[{}] {} — {}",
            if ch.ok { "OK" } else { "FAIL" },
            ch.name,
            ch.detail
        );
    }
    println!(
        "query_pass={}/{} elapsed_ms={}",
        pass,
        tests.len(),
        t0.elapsed().as_millis()
    );
    println!(
        "LIVE_STATUS written to {:?}",
        export::default_live_path()
    );

    println!("\n=== CAPABILITY MAP ===");
    println!("[YES] FSOT scalar compute (domain ledger signs)");
    println!("[YES] Genome + dual-axis trinary guidance");
    println!("[YES] Multi-organism pack + generational selection");
    println!("[YES] Cognitive exams (codon/regime/bio-align)");
    println!("[YES] Language organ: intent → plan → ground → speak");
    println!("[YES] Semantic memory (hashed embeddings) + dialogue facts");
    println!("[YES] Language exams under fitness pressure");
    println!("[YES] Pack signals + shared library + collective multihop/consistency");
    println!("[YES] Homeostasis sin/cos perceived-time loops (FSOT 2.1)");
    println!("[YES] FSOT framework tool (fsot_domain)");
    println!("[YES] Code L2 + inventable tools + JS sandbox organ (if node)");
    println!("[YES] Self-proposals → PROPOSALS/ human review (never auto-Rust)");
    println!("[YES] Drive curriculum seeds from media roots");
    println!("[YES] Live JSON export + QUERY_IN/OUT bridge");
    println!("[NEXT] Audio/video organs; ESP32 sensory edge later");
}
