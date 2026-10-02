//! Live status export + external query bridge (Python bench / agents)

use crate::habitat::Habitat;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct LiveSnapshot {
    pub tick: u64,
    pub generation: u32,
    pub fed: u64,
    pub exams: u64,
    pub births: u64,
    pub paused: bool,
    pub bio_target: f64,
    pub hero: OrgSnap,
    pub pack: Vec<OrgSnap>,
    pub lineage: Vec<LineageSnap>,
    pub recent_events: Vec<String>,
    pub last_query_kind: String,
    pub last_answer: String,
    pub current_phase: String,
    pub current_detail: String,
    pub pack_library_len: usize,
    pub pack_library_publishes: u64,
    pub collective_pass: u64,
    pub collective_fail: u64,
    pub perceived_drive: f64,
    pub perceived_sin: f64,
    pub perceived_cos: f64,
    pub couple_gain: f64,
    pub last_collective: String,
    pub proposals_pending: usize,
    pub audio_index_len: usize,
    pub video_index_len: usize,
    /// Trinary body chassis (Mind ABI)
    pub body_online: bool,
    pub body_tick: u64,
    pub body_genome_fp: String,
    pub body_repress: u64,
    pub body_latent: u64,
    pub body_express: u64,
    pub body_mean_expr: f64,
    pub body_plasticity: f64,
    pub body_couple: f64,
    pub body_motors_sent: u64,
    pub body_last_motor: String,
    pub body_transport: String,
    pub scale_tier: String,
    pub scale_ram_mb: u64,
    pub scale_vram_mb: u64,
    pub scale_pop: usize,
    pub scale_knowledge_cap: usize,
    pub capabilities: Capabilities,
    /// Constraint / monitor membrane
    pub constraint_deny: u64,
    pub constraint_throttle: u64,
    pub constraint_watch: u64,
    pub self_model_pressure: f64,
    pub monitor_level: String,
    pub updated_at_unix_ms: u128,
}

#[derive(Serialize, Clone)]
pub struct OrgSnap {
    pub id: String,
    pub s: f64,
    pub raw_s: f64,
    pub regime: String,
    pub coherence: f64,
    pub pressure: f64,
    pub reward: f64,
    pub d_eff: f64,
    pub delta_psi: f64,
    pub cognitive_accuracy: f64,
    pub intelligence: f64,
    pub fitness: f64,
    pub codons_known: usize,
    pub memory_len: usize,
    pub long_term_len: usize,
    pub aa_chain: String,
    pub genome: String,
    pub accepted_mut: u64,
    pub rejected_mut: u64,
    pub exam_count: u64,
    pub code_skill: f64,
    pub vision_skill: f64,
    pub tool_skill: f64,
    pub language_skill: f64,
    pub code_accuracy: f64,
    pub vision_accuracy: f64,
    pub language_accuracy: f64,
    pub semantic_len: usize,
    pub dialogue_turns: u64,
    pub social_skill: f64,
    pub social_alignment: f64,
    pub collective_accuracy: f64,
    pub generalize_accuracy: f64,
    pub lang_pathways: usize,
    pub composition_accuracy: f64,
    pub js_accuracy: f64,
    pub code_l2_accuracy: f64,
    pub proposal_skill: f64,
    pub proposals_emitted: u64,
    pub knowledge_accuracy: f64,
    pub knowledge_skill: f64,
    pub audio_accuracy: f64,
    pub video_accuracy: f64,
    pub self_model_accuracy: f64,
    pub self_model_skill: f64,
    pub bench_accuracy: f64,
    pub bench_zs_accuracy: f64,
    pub js_tools: Vec<String>,
    pub tools_owned: Vec<String>,
}

#[derive(Serialize)]
pub struct LineageSnap {
    pub gen: u32,
    pub champion: String,
    pub s: f64,
    pub acc: f64,
    pub fit: f64,
    pub codons: usize,
}

#[derive(Serialize)]
pub struct Capabilities {
    pub fsot_scalar: bool,
    pub domain_ledger: bool,
    pub genome_guidance: bool,
    pub structural_mutation: bool,
    pub cognitive_exams: bool,
    pub interactive_query: bool,
    pub multi_organism_selection: bool,
    pub lineage_tracking: bool,
    pub live_json_export: bool,
    pub notes: Vec<String>,
}

pub fn export_dir() -> PathBuf {
    // Prefer sibling of crate: ../state/habitat-rust
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("state");
    p.push("habitat-rust");
    p
}

pub fn default_live_path() -> PathBuf {
    export_dir().join("LIVE_STATUS.json")
}

// ─── External query bridge (same interface as TUI Enter) ─────────────────────

#[derive(Deserialize)]
struct QueryIn {
    id: String,
    text: String,
    #[serde(default)]
    source: String,
}

#[derive(Serialize)]
struct QueryOut {
    id: String,
    text: String,
    source: String,
    kind: String,
    intent: String,
    plan: String,
    answer: String,
    grounded: bool,
    tick: u64,
    generation: u32,
    hero_id: String,
    hero_s: f64,
    hero_regime: String,
    language_accuracy: f64,
    cognitive_accuracy: f64,
    code_accuracy: f64,
    vision_accuracy: f64,
    fitness: f64,
    semantic_len: usize,
    dialogue_turns: u64,
    unix_ms: u128,
    bridge: &'static str,
}

/// Poll QUERY_IN.json → ask champion → QUERY_OUT.json + query_log.jsonl
/// Safe to call every tick; no-op if no pending inbox.
pub fn serve_external_queries(h: &mut Habitat) {
    let dir = export_dir();
    let _ = fs::create_dir_all(&dir);
    let inbox = dir.join("QUERY_IN.json");
    if !inbox.exists() {
        return;
    }
    let raw = match fs::read_to_string(&inbox) {
        Ok(s) => s,
        Err(_) => return,
    };
    // Remove first so a crash mid-answer does not infinite-loop the same query
    let _ = fs::remove_file(&inbox);

    let req: QueryIn = match serde_json::from_str(&raw) {
        Ok(q) => q,
        Err(e) => {
            let _ = write_query_error(&dir, "parse_error", &raw, &e.to_string());
            return;
        }
    };
    if req.text.trim().is_empty() {
        return;
    }

    let ans = h.ask(&req.text);
    let hero = h.hero();
    let out = QueryOut {
        id: req.id.clone(),
        text: req.text.clone(),
        source: if req.source.is_empty() {
            "external".into()
        } else {
            req.source
        },
        kind: ans.kind,
        intent: ans.intent,
        plan: ans.plan,
        answer: ans.answer,
        grounded: ans.grounded,
        tick: h.tick,
        generation: h.generation,
        hero_id: hero.id.clone(),
        hero_s: hero.s,
        hero_regime: hero.regime.clone(),
        language_accuracy: hero.language_accuracy,
        cognitive_accuracy: hero.cognitive_accuracy,
        code_accuracy: hero.code_accuracy,
        vision_accuracy: hero.vision_accuracy,
        fitness: hero.fitness_score(),
        semantic_len: hero.semantic.len(),
        dialogue_turns: hero.dialogue.turns,
        unix_ms: now_ms(),
        bridge: "tui-query-equivalent",
    };

    if let Ok(json) = serde_json::to_string_pretty(&out) {
        let tmp = dir.join("QUERY_OUT.json.tmp");
        let path = dir.join("QUERY_OUT.json");
        let _ = fs::write(&tmp, &json);
        let _ = fs::rename(&tmp, &path);
    }
    // Append log for offline analysis
    if let Ok(line) = serde_json::to_string(&out) {
        use std::io::Write;
        let log = dir.join("query_log.jsonl");
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log) {
            let _ = writeln!(f, "{line}");
        }
    }
    // Refresh live status immediately so bench can correlate
    let _ = write_live(h);
}

fn write_query_error(dir: &std::path::Path, id: &str, raw: &str, err: &str) {
    let v = serde_json::json!({
        "id": id,
        "error": err,
        "raw": raw.chars().take(200).collect::<String>(),
        "unix_ms": now_ms(),
    });
    let _ = fs::write(dir.join("QUERY_OUT.json"), v.to_string());
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn write_live(h: &Habitat) -> std::io::Result<PathBuf> {
    let dir = export_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("LIVE_STATUS.json");

    let pack: Vec<OrgSnap> = h
        .pack
        .iter()
        .map(|o| OrgSnap {
            id: o.id.clone(),
            s: o.s,
            raw_s: o.raw_s,
            regime: o.regime.clone(),
            coherence: o.coherence,
            pressure: o.pressure,
            reward: o.reward,
            d_eff: o.d_eff,
            delta_psi: o.delta_psi,
            cognitive_accuracy: o.cognitive_accuracy,
            intelligence: o.intelligence,
            fitness: o.fitness_score(),
            codons_known: o.codon_table.len(),
            memory_len: o.memory.len(),
            long_term_len: o.long_term.len(),
            aa_chain: o.guidance.aa_chain.clone(),
            genome: o.genome.join("-"),
            accepted_mut: o.accepted_mut,
            rejected_mut: o.rejected_mut,
            exam_count: o.exam_count,
            code_skill: o.code_skill,
            vision_skill: o.vision_skill,
            tool_skill: o.tool_skill,
            language_skill: o.language_skill,
            code_accuracy: o.code_accuracy,
            vision_accuracy: o.vision_accuracy,
            language_accuracy: o.language_accuracy,
            semantic_len: o.semantic.len(),
            dialogue_turns: o.dialogue.turns,
            social_skill: o.social_skill,
            social_alignment: o.social_alignment,
            collective_accuracy: o.collective_accuracy,
            generalize_accuracy: o.lang_cortex.generalize_accuracy,
            lang_pathways: o.lang_cortex.pathway_count,
            composition_accuracy: o.composition_accuracy,
            js_accuracy: o.js_accuracy,
            code_l2_accuracy: o.code_l2_accuracy,
            proposal_skill: o.proposal_skill,
            proposals_emitted: o.proposals_emitted,
            knowledge_accuracy: o.knowledge_accuracy,
            knowledge_skill: o.knowledge_skill,
            audio_accuracy: o.audio_accuracy,
            video_accuracy: o.video_accuracy,
            self_model_accuracy: o.self_model_accuracy,
            self_model_skill: o.self_model_skill,
            bench_accuracy: o.bench_accuracy,
            bench_zs_accuracy: o.bench_zs_accuracy,
            js_tools: o.js_tools_owned.clone(),
            tools_owned: o.tools_owned.clone(),
        })
        .collect();

    let hero = pack
        .iter()
        .find(|o| o.id == h.hero().id)
        .cloned()
        .unwrap_or_else(|| pack[h.hero_idx.min(pack.len().saturating_sub(1))].clone());

    let snap = LiveSnapshot {
        tick: h.tick,
        generation: h.generation,
        fed: h.fed,
        exams: h.exams,
        births: h.births,
        paused: h.paused,
        bio_target: h.bio_target,
        hero,
        pack,
        lineage: h
            .lineage
            .iter()
            .map(|l| LineageSnap {
                gen: l.gen,
                champion: l.champion.clone(),
                s: l.s,
                acc: l.acc,
                fit: l.fit,
                codons: l.codons,
            })
            .collect(),
        recent_events: h
            .events
            .iter()
            .take(25)
            .map(|e| format!("[t{}] {}", e.tick, e.msg))
            .collect(),
        last_query_kind: h.last_query_kind.clone(),
        last_answer: h.last_answer.clone(),
        current_phase: h.current_phase.clone(),
        current_detail: h.current_detail.clone(),
        pack_library_len: h.shared_library.len(),
        pack_library_publishes: h.shared_library.publishes,
        collective_pass: h.shared_library.collective_pass,
        collective_fail: h.shared_library.collective_fail,
        perceived_drive: h.perceived.drive,
        perceived_sin: h.perceived.phase_sin,
        perceived_cos: h.perceived.phase_cos,
        couple_gain: h.perceived.couple_gain,
        last_collective: h.last_collective_detail.clone(),
        proposals_pending: crate::proposals::count_pending(),
        audio_index_len: h.audio_index.len(),
        video_index_len: h.video_index.len(),
        capabilities: Capabilities {
            fsot_scalar: true,
            domain_ledger: true,
            genome_guidance: true,
            structural_mutation: true,
            cognitive_exams: true,
            interactive_query: true,
            multi_organism_selection: true,
            lineage_tracking: true,
            live_json_export: true,
            notes: vec![
                "Audio energy organ: $FSOT_LOCAL_DATA/Debut/movies/showes via ffmpeg RMS → FSOT regime".into(),
                "Video drive pressure: frame sample → vision features + temporal hint".into(),
                "JS organ: tools_js forge (clamp/double/add/fsot_sign) + challenges".into(),
                "Knowledge packs from $FSOT_TRAINING_DATA (exam-backed FactCards)".into(),
                "LLM-capability bridge: MMLU·GSM8K·HellaSwag·TruthfulQA·MBPP → organs (CAPABILITY_AUDIT.json)".into(),
                "Plastic language + pack + code L2 + proposals (never auto-Rust)".into(),
                "Trinary body chassis: BODY_STATUS.json / BODY_MOTOR.txt Mind ABI (fsot-trinary-body)".into(),
                "Goldfish scale: probe RAM/VRAM → lean|standard|expanded|max (Omen 35L + 5070 → expanded/max)".into(),
                "Memory tiers: disk LTM (BODY_LTM) · RAM working set · TCP nerves · CPU genetic flow".into(),
                "Constraint membrane: no auto-Rust · no Mind identity clobber · sandbox · rate limits".into(),
                "Behavior monitor: BEHAVIOR_MONITOR.json · watch struggles without assuming malice".into(),
                "Accuracy homeo: ACCURACY_AUDIT.json · weak organs get teach+exam bias".into(),
            ],
        },
        constraint_deny: h.constraints.deny_count,
        constraint_throttle: h.constraints.throttle_count,
        constraint_watch: h.constraints.watch_count,
        self_model_pressure: h.constraints.self_model_pressure,
        monitor_level: h.monitor_level.clone(),
        body_online: h.body.online,
        body_tick: h.body.status.tick,
        body_genome_fp: h.body.status.genome_fp.clone(),
        body_repress: h.body.status.repress,
        body_latent: h.body.status.latent,
        body_express: h.body.status.express,
        body_mean_expr: h.body.status.mean_expr,
        body_plasticity: h.body.status.plasticity,
        body_couple: h.body.couple_pressure,
        body_motors_sent: h.body.motors_sent,
        body_last_motor: h.body.last_motor.clone(),
        body_transport: h.body.transport.clone(),
        scale_tier: h.scale.tier.clone(),
        scale_ram_mb: h.scale.ram_mb,
        scale_vram_mb: h.scale.vram_mb,
        scale_pop: h.scale.pop_size,
        scale_knowledge_cap: h.scale.knowledge_cap,
        updated_at_unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    };

    let json = serde_json::to_string_pretty(&snap)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // Atomic-ish write
    let tmp = dir.join("LIVE_STATUS.json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    // Also rolling log line
    let log = dir.join("live_ticks.jsonl");
    if let Ok(line) = serde_json::to_string(&serde_json::json!({
        "tick": snap.tick,
        "gen": snap.generation,
        "hero": snap.hero.id,
        "S": snap.hero.s,
        "acc": snap.hero.cognitive_accuracy,
        "lang": snap.hero.language_accuracy,
        "coll": snap.hero.collective_accuracy,
        "social": snap.hero.social_skill,
        "lib": snap.pack_library_len,
        "drive": snap.perceived_drive,
        "fit": snap.hero.fitness,
        "codons": snap.hero.codons_known,
        "body": snap.body_online,
        "body_couple": snap.body_couple,
        "body_motors": snap.body_motors_sent,
    })) {
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log) {
            let _ = writeln!(f, "{line}");
        }
    }
    Ok(path)
}
