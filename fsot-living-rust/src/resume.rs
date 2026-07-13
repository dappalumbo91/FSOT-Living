//! Mind STM couple — resume cortex continuity from body + live state.
//!
//! Layers (mind · body · soul framing as engineering map):
//!   hardware body  = BODY_LTM / soma ticks / home VFS
//!   software soul  = LIVE_STATUS + pack genome + skills (this module)
//!   mind process   = habitat loop that *feels* and selects
//!
//! Boot order for continuous self:
//!   1) Habitat zygote (default pack)
//!   2) LIVE_STATUS.json hero genome / generation / skills if fresh
//!   3) home/genome/LATEST.txt germline if LIVE missing
//!   4) home/identity session lines → hero long_term (autobiography)
//!   5) BODY_STATUS couple flags

use crate::export::export_dir;
use crate::habitat::Habitat;
use crate::organism::Organism;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct LiveResume {
    #[serde(default)]
    generation: u32,
    #[serde(default)]
    tick: u64,
    #[serde(default)]
    hero: Option<HeroResume>,
    #[serde(default)]
    lineage: Vec<LineageResume>,
    #[serde(default)]
    body_motors_sent: u64,
    #[serde(default)]
    last_answer: String,
}

#[derive(Debug, Deserialize)]
struct HeroResume {
    #[serde(default)]
    id: String,
    #[serde(default)]
    genome: String,
    #[serde(default)]
    s: f64,
    #[serde(default)]
    regime: String,
    #[serde(default)]
    cognitive_accuracy: f64,
    #[serde(default)]
    code_accuracy: f64,
    #[serde(default)]
    language_accuracy: f64,
    #[serde(default)]
    vision_accuracy: f64,
    #[serde(default)]
    knowledge_accuracy: f64,
    #[serde(default)]
    social_skill: f64,
    #[serde(default)]
    fitness: f64,
    #[serde(default)]
    tools_owned: Vec<String>,
    #[serde(default)]
    js_tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LineageResume {
    #[serde(default)]
    gen: u32,
    #[serde(default)]
    champion: String,
    #[serde(default)]
    s: f64,
    #[serde(default)]
    acc: f64,
    #[serde(default)]
    fit: f64,
    #[serde(default)]
    codons: usize,
}

fn home_root() -> PathBuf {
    export_dir().join("home")
}

fn parse_genome(s: &str) -> Vec<String> {
    s.split(|c: char| c == '-' || c == ',' || c.is_whitespace())
        .map(|t| t.trim().to_ascii_uppercase())
        .filter(|t| {
            t.len() == 3 && t.chars().all(|c| matches!(c, 'A' | 'C' | 'G' | 'T'))
        })
        .collect()
}

fn load_latest_germline() -> Option<(u32, String, f64, String)> {
    let path = home_root().join("genome").join("LATEST.txt");
    let text = fs::read_to_string(path).ok()?;
    let mut gen = 0u32;
    let mut hero = String::from("unknown");
    let mut fit = 0.0f64;
    let mut genome = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("gen=") {
            gen = v.parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("hero=") {
            hero = v.to_string();
        } else if let Some(v) = line.strip_prefix("fit=") {
            fit = v.parse().unwrap_or(0.0);
        } else if let Some(v) = line.strip_prefix("genome=") {
            genome = v.to_string();
        }
    }
    if genome.is_empty() {
        None
    } else {
        Some((gen, hero, fit, genome))
    }
}

fn load_identity_sessions() -> Vec<String> {
    let path = home_root().join("identity").join("NOTES.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && (l.starts_with("session:") || l.starts_with("core:"))
        })
        .collect()
}

fn apply_genome_to_hero(hero: &mut Organism, genome: &[String], id: &str) {
    if genome.is_empty() {
        return;
    }
    hero.genome = genome.to_vec();
    hero.guidance = crate::genetic::genome_to_guidance(&hero.genome);
    if !id.is_empty() && id != "unknown" {
        hero.id = id.to_string();
    }
}

/// Resume mind STM from disk body + previous live export.
/// Returns human-readable resume log lines.
pub fn resume_mind_stm(h: &mut Habitat) -> Vec<String> {
    let mut log = Vec::new();
    let live_path = export_dir().join("LIVE_STATUS.json");
    let mut resumed = false;

    // 1) LIVE_STATUS — full cortex snapshot if present
    if live_path.exists() {
        if let Ok(raw) = fs::read_to_string(&live_path) {
            if let Ok(live) = serde_json::from_str::<LiveResume>(&raw) {
                if let Some(hero_snap) = live.hero {
                    let g = parse_genome(&hero_snap.genome);
                    if !g.is_empty() {
                        apply_genome_to_hero(h.hero_mut(), &g, &hero_snap.id);
                        // Soft skill continuity (not full weight dump — phenotype continuity)
                        let o = h.hero_mut();
                        if hero_snap.cognitive_accuracy > 0.01 {
                            o.cognitive_accuracy = hero_snap.cognitive_accuracy;
                        }
                        if hero_snap.code_accuracy > 0.01 {
                            o.code_accuracy = hero_snap.code_accuracy;
                        }
                        if hero_snap.language_accuracy > 0.01 {
                            o.language_accuracy = hero_snap.language_accuracy;
                        }
                        if hero_snap.vision_accuracy > 0.01 {
                            o.vision_accuracy = hero_snap.vision_accuracy;
                        }
                        if hero_snap.knowledge_accuracy > 0.01 {
                            o.knowledge_accuracy = hero_snap.knowledge_accuracy;
                        }
                        if hero_snap.social_skill > 0.01 {
                            o.social_skill = hero_snap.social_skill;
                        }
                        if !hero_snap.tools_owned.is_empty() {
                            o.tools_owned = hero_snap.tools_owned;
                        }
                        if !hero_snap.js_tools.is_empty() {
                            o.js_tools_owned = hero_snap.js_tools;
                        }
                        if !hero_snap.regime.is_empty() {
                            o.regime = hero_snap.regime;
                        }
                        if hero_snap.s != 0.0 {
                            o.s = hero_snap.s;
                        }
                        log.push(format!(
                            "STM resume LIVE hero={} gen={} tick={} fit≈{:.2} genome_codons={}",
                            o.id,
                            live.generation,
                            live.tick,
                            hero_snap.fitness,
                            g.len()
                        ));
                        resumed = true;
                    }
                }
                if live.generation > 0 {
                    h.generation = live.generation;
                    h.body.last_genome_gen = live.generation;
                }
                if !live.lineage.is_empty() {
                    h.lineage = live
                        .lineage
                        .into_iter()
                        .map(|e| crate::habitat::LineageEntry {
                            gen: e.gen,
                            champion: e.champion,
                            s: e.s,
                            acc: e.acc,
                            fit: e.fit,
                            codons: e.codons,
                        })
                        .collect();
                    log.push(format!("STM lineage entries={}", h.lineage.len()));
                }
                if live.body_motors_sent > 0 {
                    h.body.motors_sent = live.body_motors_sent;
                }
                if !live.last_answer.is_empty() && live.last_answer.len() < 400 {
                    h.last_answer = format!(
                        "[resumed session]\n{}\n\n(Continuing — ask 'how are you' or 'body status'.)",
                        live.last_answer.chars().take(280).collect::<String>()
                    );
                }
            }
        }
    }

    // 2) Germline LATEST if LIVE had no usable genome
    if !resumed {
        if let Some((gen, hero, fit, genome)) = load_latest_germline() {
            let g = parse_genome(&genome);
            if !g.is_empty() {
                apply_genome_to_hero(h.hero_mut(), &g, &hero);
                if gen > 0 {
                    h.generation = gen;
                    h.body.last_genome_gen = gen;
                }
                log.push(format!(
                    "STM resume GERMLINE gen={gen} hero={hero} fit={fit:.3} codons={}",
                    g.len()
                ));
                resumed = true;
            }
        }
    } else if let Some((gen, hero, fit, _)) = load_latest_germline() {
        // Cross-check: body germline generation should not lag LIVE
        if gen > h.generation {
            h.generation = gen;
            h.body.last_genome_gen = gen;
            log.push(format!(
                "STM germline ahead → gen={gen} hero={hero} fit={fit:.3}"
            ));
        }
    }

    // 3) Identity autobiography → long_term (soul narrative thread)
    let sessions = load_identity_sessions();
    if !sessions.is_empty() {
        let hero = h.hero_mut();
        for s in sessions.iter().rev().take(24).rev() {
            if !hero.long_term.iter().any(|x| x == s) {
                hero.long_term.push(s.clone());
            }
        }
        // Cap
        if hero.long_term.len() > 200 {
            let drop_n = hero.long_term.len() - 200;
            hero.long_term.drain(0..drop_n);
        }
        log.push(format!(
            "STM identity sessions absorbed={} (long_term={})",
            sessions.len().min(24),
            hero.long_term.len()
        ));
    }

    // 4) Body online probe (file status)
    if let Some(st) = crate::body_bridge::read_body_status() {
        h.body.online = true;
        h.body.status = st;
        if h.body.transport.is_empty() {
            h.body.transport = "file".into();
        }
        log.push(format!(
            "STM body couple tick={} fp={}",
            h.body.status.tick, h.body.status.genome_fp
        ));
    }

    if !resumed && log.is_empty() {
        log.push("STM cold start — no LIVE/germline yet (zygote mind)".into());
    }

    // Remember resume in hero short memory
    let summary = log.join(" · ");
    h.hero_mut().memory.push(crate::organism::MemoryItem {
        text: format!("resume: {summary}"),
        kind: "stm_resume".into(),
        strength: 0.9,
    });
    h.push(format!("mind STM: {summary}"));
    log
}
