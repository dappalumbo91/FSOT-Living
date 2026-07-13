//! Formal task battery after a live evolution window.
//! Produces TASK_REPORT.json for review + evolutionary pathway targets.

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

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Serialize)]
struct TaskResult {
    name: String,
    category: String,
    passed: bool,
    detail: String,
}

#[derive(Serialize)]
struct TaskReport {
    ticks: u64,
    generations: u32,
    hero_id: String,
    hero_s: f64,
    cognitive: f64,
    code_acc: f64,
    vision_acc: f64,
    code_skill: f64,
    vision_skill: f64,
    tools: Vec<String>,
    fitness: f64,
    codons: usize,
    tasks: Vec<TaskResult>,
    pass_count: usize,
    total: usize,
    pass_rate: f64,
    gaps: Vec<String>,
    pathway_next: Vec<String>,
    media_roots_found: Vec<String>,
    elapsed_ms: u128,
}

fn main() {
    println!("=== FSOT TASK BATTERY + EVOLUTION WINDOW ===\n");
    let t0 = Instant::now();

    // --- Evolution window ---
    let mut h = habitat::Habitat::new(99);
    let ticks = 600u64;
    for _ in 0..ticks {
        h.step();
    }
    let _ = export::write_live(&h);
    let hero = h.hero().clone_lite();

    println!(
        "After {ticks} ticks / gen {}: {} S={:.3} cog={:.0}% code={:.0}% vision={:.0}% fit={:.2} tools={:?}",
        h.generation,
        hero.id,
        hero.s,
        hero.cognitive_accuracy * 100.0,
        hero.code_accuracy * 100.0,
        hero.vision_accuracy * 100.0,
        hero.fitness_score(),
        hero.tools_owned
    );

    let mut tasks = Vec::new();

    // --- Query + natural language tasks ---
    let q_cases: &[(&str, &str, fn(&query::QueryResult) -> bool)] = &[
        ("greeting", "hello how are you", |r| r.kind == "feeling" || r.kind == "greeting"),
        ("status", "status", |r| r.kind == "status" && r.answer.contains("S=")),
        ("codon_atg", "codon ATG", |r| r.answer.contains('M')),
        ("codon_aaa", "codon AAA", |r| r.answer.contains('K')),
        ("nl_codon", "What amino acid does ATG code for?", |r| r.kind == "codon" && r.answer.contains('M')),
        ("nl_domain", "Tell me about the quantum domain", |r| r.kind == "domain"),
        ("nl_math", "What is 3+4?", |r| r.answer.contains('7')),
        ("nl_identity", "Who are you?", |r| r.kind == "identity"),
        ("nl_remember", "Remember that my name is TaskUser", |r| r.kind == "remember"),
        ("nl_recall", "What is my name?", |r| r.answer.contains("TaskUser")),
        ("domain_q", "domain quantum", |r| r.answer.contains("S=")),
        ("domain_c", "domain cosmological", |r| r.answer.contains("damping") || r.answer.contains("S=")),
        ("genome", "genome", |r| r.kind == "genome"),
        ("organs", "organs", |r| r.kind == "organs" || r.answer.contains("language")),
        ("py_add", "py: 2+40", |r| r.answer.contains("42")),
        ("py_sum", "py: print(sum(range(1,11)))", |r| r.answer.contains("55")),
        ("see_list", "see", |r| r.kind == "vision"),
        ("forge", "forge double", |r| r.answer.to_lowercase().contains("forge") || r.answer.contains("double")),
        ("tools", "tools", |r| r.kind == "tools"),
        ("fit", "fit", |r| r.kind == "fitness"),
    ];

    for (name, q, check) in q_cases {
        let r = query::query(h.hero_mut(), q);
        let passed = check(&r);
        tasks.push(TaskResult {
            name: (*name).into(),
            category: "query".into(),
            passed,
            detail: r.answer.chars().take(140).collect(),
        });
        println!(
            "[{}] query/{} -> {}",
            if passed { "PASS" } else { "FAIL" },
            name,
            r.kind
        );
    }

    // --- Code challenges (direct pressure) ---
    for ch in code_env::code_challenges() {
        let passed = code_env::run_code_exam(ch);
        tasks.push(TaskResult {
            name: format!("code_{}", ch.id),
            category: "code".into(),
            passed,
            detail: format!("expect {}", ch.expect_stdout),
        });
        println!(
            "[{}] code/{}",
            if passed { "PASS" } else { "FAIL" },
            ch.id
        );
    }

    // --- Vision challenges ---
    let _ = vision::ensure_vision_lib();
    for ch in vision::vision_challenges() {
        let passed = vision::run_vision_exam(ch);
        tasks.push(TaskResult {
            name: format!("vision_{}", ch.file),
            category: "vision".into(),
            passed,
            detail: format!("expect regime {}", ch.expect_regime),
        });
        println!(
            "[{}] vision/{}",
            if passed { "PASS" } else { "FAIL" },
            ch.file
        );
    }

    // --- Tool forge battery ---
    for tool in ["double", "add", "fsot_sign"] {
        let passed = code_env::forge_tool_exam(tool);
        tasks.push(TaskResult {
            name: format!("forge_{tool}"),
            category: "tools".into(),
            passed,
            detail: format!("tool {tool}.py"),
        });
        println!(
            "[{}] forge/{}",
            if passed { "PASS" } else { "FAIL" },
            tool
        );
    }

    // --- Scalar ledger ---
    let q = scalar::compute_for_domain("quantum");
    let c = scalar::compute_for_domain("cosmological");
    let scalar_ok = q.s > 0.0 && c.s < 0.0;
    tasks.push(TaskResult {
        name: "scalar_signs".into(),
        category: "scalar".into(),
        passed: scalar_ok,
        detail: format!("q={:.4} c={:.4}", q.s, c.s),
    });
    println!(
        "[{}] scalar signs",
        if scalar_ok { "PASS" } else { "FAIL" }
    );

    // --- Media roots (real data inventory, shallow) ---
    let media_roots = scan_media_roots();
    println!("\nMedia roots with content: {}", media_roots.len());
    for m in &media_roots {
        println!("  {m}");
    }

    let pass_count = tasks.iter().filter(|t| t.passed).count();
    let total = tasks.len();
    let pass_rate = pass_count as f64 / total as f64;

    // Gap analysis from failures + known architecture limits
    let mut gaps = Vec::new();
    for t in &tasks {
        if !t.passed {
            gaps.push(format!("FAIL task {}: {}", t.name, t.detail));
        }
    }
    if hero.vision_accuracy < 0.9 {
        gaps.push("Vision organ not saturated — needs harder real-image pressure".into());
    }
    if hero.code_accuracy < 0.9 {
        gaps.push("Code organ not saturated — expand multi-step Python challenges".into());
    }
    if hero.language_accuracy < 0.85 {
        gaps.push("Language organ not saturated — expand freeform paraphrase exams".into());
    }
    gaps.push("No audio organ yet (music on external drives ready as pressure source)".into());
    gaps.push("No video organ yet (frame sampling + temporal pressure next)".into());
    gaps.push("No JS sandbox yet (same interpretable pattern as Python)".into());
    gaps.push("Stage D fluency cortex (optional local LM mouth under truth veto) not yet".into());
    gaps.push("Multi-hop planning exams still thin — expand Stage C battery".into());

    let pathway_next = vec![
        "LANGUAGE_V2: larger paraphrase set + multi-turn dialogue pressure".into(),
        "LANGUAGE_D: optional local LM as speech cortex screened by grounded truth".into(),
        "AUDIO_PRESSURE: index D:/G: music; exams on tempo/energy proxies; fitness += audio_acc".into(),
        "VIDEO_PRESSURE: sample frames from drive video; vision exams on sequences; fitness += video_acc".into(),
        "CODE_PRESSURE_L2: multi-file Python projects with unit tests in organism_workspace/practice".into(),
        "TOOL_PRESSURE: freeform tool invention — organism writes py that must pass generated tests".into(),
        "JS_PRESSURE: node -e sandbox parallel to python for internal prototyping".into(),
        "DRIVE_CURRICULUM: stream real media paths into teach() as structured seeds".into(),
    ];

    let report = TaskReport {
        ticks,
        generations: h.generation,
        hero_id: hero.id.clone(),
        hero_s: hero.s,
        cognitive: hero.cognitive_accuracy,
        code_acc: hero.code_accuracy,
        vision_acc: hero.vision_accuracy,
        code_skill: hero.code_skill,
        vision_skill: hero.vision_skill,
        tools: hero.tools_owned.clone(),
        fitness: hero.fitness,
        codons: hero.codon_table_len,
        tasks,
        pass_count,
        total,
        pass_rate,
        gaps,
        pathway_next,
        media_roots_found: media_roots,
        elapsed_ms: t0.elapsed().as_millis(),
    };

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("state")
        .join("habitat-rust");
    let _ = fs::create_dir_all(&out_dir);
    let path = out_dir.join("TASK_REPORT.json");
    let pathway = out_dir.join("EVOLUTIONARY_PATHWAY.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    fs::write(
        &pathway,
        serde_json::to_string_pretty(&serde_json::json!({
            "principle": "Fill gaps only under evolutionary pressure — exams + fitness weights + inheritance",
            "current_strengths": [
                "FSOT scalar domain signs",
                "Genome guidance + codon learning",
                "Language organ intent→plan→speak + semantic memory",
                "Python sandbox + tool forge",
                "Vision structure exams",
                "Interactive natural-language TUI"
            ],
            "pressure_schedule": report.pathway_next,
            "media_roots": report.media_roots_found,
            "instruction_to_habitat": [
                "On each exam cycle, sample real media from indexed roots when available",
                "Increase fitness weight for organs that lag (adaptive pressure)",
                "Children inherit organ skills and verified tools",
                "Failed freeform code becomes practice curriculum, not deleted knowledge"
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    // Also human-readable pathway md
    let md = format!(
        "# Evolutionary Pathway (auto-generated)\n\n\
         **Pass rate:** {:.0}% ({}/{})\n\n\
         ## Hero after {} ticks\n\
         - id: {}\n- S: {:.4}\n- cognitive: {:.1}%\n- code: {:.1}%\n- vision: {:.1}%\n- tools: {:?}\n- fitness: {:.3}\n\n\
         ## Gaps\n{}\n\n\
         ## Next pressure schedule\n{}\n\n\
         ## Media roots detected\n{}\n",
        pass_rate * 100.0,
        pass_count,
        total,
        ticks,
        report.hero_id,
        report.hero_s,
        report.cognitive * 100.0,
        report.code_acc * 100.0,
        report.vision_acc * 100.0,
        report.tools,
        report.fitness,
        report
            .gaps
            .iter()
            .map(|g| format!("- {g}"))
            .collect::<Vec<_>>()
            .join("\n"),
        report
            .pathway_next
            .iter()
            .map(|g| format!("- {g}"))
            .collect::<Vec<_>>()
            .join("\n"),
        report
            .media_roots_found
            .iter()
            .map(|g| format!("- `{g}`"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    fs::write(out_dir.join("EVOLUTIONARY_PATHWAY.md"), md).unwrap();

    println!(
        "\n=== SUMMARY === {}/{} ({:.0}%)  elapsed {}ms",
        pass_count,
        total,
        pass_rate * 100.0,
        t0.elapsed().as_millis()
    );
    println!("Wrote {:?}", path);
    println!("Wrote {:?}", out_dir.join("EVOLUTIONARY_PATHWAY.md"));
}

/// Lightweight media discovery (no deep recursive crawl)
fn scan_media_roots() -> Vec<String> {
    let candidates = [
        r"G:\movies",
        r"G:\showes",
        r"G:\SteamLibrary",
        r"D:\FSOT linguistics",
        r"D:\FSOT NeuroLab",
        r"C:\Users\damia\Desktop\Genetics",
        r"C:\Users\damia\Music",
        r"C:\Users\damia\Videos",
        r"C:\Users\damia\Pictures",
    ];
    let mut found = Vec::new();
    for c in candidates {
        let p = PathBuf::from(c);
        if !p.exists() {
            continue;
        }
        // count a few media files at top levels
        let mut n = 0usize;
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten().take(50) {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.ends_with(".mp3")
                    || name.ends_with(".wav")
                    || name.ends_with(".mp4")
                    || name.ends_with(".mkv")
                    || name.ends_with(".jpg")
                    || name.ends_with(".png")
                    || e.path().is_dir()
                {
                    n += 1;
                }
            }
        }
        // also one-level children
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten().take(20) {
                if e.path().is_dir() {
                    if let Ok(rd2) = fs::read_dir(e.path()) {
                        for f in rd2.flatten().take(30) {
                            let name = f.file_name().to_string_lossy().to_lowercase();
                            if name.ends_with(".mp3")
                                || name.ends_with(".mp4")
                                || name.ends_with(".mkv")
                                || name.ends_with(".wav")
                                || name.ends_with(".flac")
                            {
                                n += 1;
                            }
                        }
                    }
                }
            }
        }
        if n > 0 {
            found.push(format!("{c} (~{n} media signals sampled)"));
        } else if p.is_dir() {
            found.push(format!("{c} (dir exists, sample empty at shallow depth)"));
        }
    }
    found
}

/// Minimal clone of hero metrics without full Organism Clone dependency issues
struct HeroLite {
    id: String,
    s: f64,
    cognitive_accuracy: f64,
    language_accuracy: f64,
    code_accuracy: f64,
    vision_accuracy: f64,
    code_skill: f64,
    vision_skill: f64,
    tools_owned: Vec<String>,
    codon_table_len: usize,
    fitness: f64,
}

trait CloneLite {
    fn clone_lite(&self) -> HeroLite;
}

impl CloneLite for organism::Organism {
    fn clone_lite(&self) -> HeroLite {
        HeroLite {
            id: self.id.clone(),
            s: self.s,
            cognitive_accuracy: self.cognitive_accuracy,
            language_accuracy: self.language_accuracy,
            code_accuracy: self.code_accuracy,
            vision_accuracy: self.vision_accuracy,
            code_skill: self.code_skill,
            vision_skill: self.vision_skill,
            tools_owned: self.tools_owned.clone(),
            codon_table_len: self.codon_table.len(),
            fitness: self.fitness_score(),
        }
    }
}

// silence unused
impl HeroLite {
    fn fitness_score(&self) -> f64 {
        self.fitness
    }
}
