//! Pack-wide accuracy homeostasis — not just self-model.
//!
//! When any organ accuracy sits below a healthy band, boost stimuli for that
//! organ (teach + mild skill pressure). Same pattern as self_model low-acc feed.
//!
//! Writes:
//!   state/habitat-rust/ACCURACY_AUDIT.json

use crate::export::export_dir;
use crate::organism::Organism;
use serde::Serialize;
use std::fs;

/// Healthy bands (soft floors). Below → stimulus boost.
pub mod thr {
    pub const CORE: f64 = 0.62; // cog / lang / code / vision
    pub const KNOWLEDGE: f64 = 0.85; // user bar — nothing less
    pub const SOCIAL: f64 = 0.55; // collective / social
    pub const MEDIA: f64 = 0.72; // audio / video — raised after seed/synth mastery push
    pub const SCRIPT: f64 = 0.55; // composition
    pub const JS: f64 = 0.85; // user bar — same as knowledge
    pub const META: f64 = 0.70; // self_model / gen_acc — generalization + rails
    pub const L2: f64 = 0.25; // code_l2 often sparse
    pub const BENCH: f64 = 0.80; // LLM-capability surface study→test
    pub const BENCH_ZS: f64 = 0.70; // zero-shot held-out (SOTA-shaped)
}

#[derive(Clone, Debug, Serialize)]
pub struct OrganAcc {
    pub name: String,
    pub value: f64,
    pub thr: f64,
    pub weak: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgAudit {
    pub id: String,
    pub mean_acc: f64,
    pub fitness: f64,
    pub organs: Vec<OrganAcc>,
    pub weak_count: usize,
    pub boosted: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AccuracyAudit {
    pub tick: u64,
    pub generation: u32,
    pub pack_mean: f64,
    pub pack_weak_organs: usize,
    pub orgs: Vec<OrgAudit>,
    pub note: String,
}

fn organ_list(o: &Organism) -> Vec<OrganAcc> {
    let pairs: [(&str, f64, f64); 15] = [
        ("cognitive", o.cognitive_accuracy, thr::CORE),
        ("language", o.language_accuracy, thr::CORE),
        ("code", o.code_accuracy, thr::CORE),
        ("vision", o.vision_accuracy, thr::CORE),
        ("knowledge", o.knowledge_accuracy, thr::KNOWLEDGE),
        ("collective", o.collective_accuracy, thr::SOCIAL),
        ("audio", o.audio_accuracy, thr::MEDIA),
        ("video", o.video_accuracy, thr::MEDIA),
        ("js", o.js_accuracy, thr::JS),
        ("composition", o.composition_accuracy, thr::SCRIPT),
        ("code_l2", o.code_l2_accuracy, thr::L2),
        ("gen_acc", o.lang_cortex.generalize_accuracy, thr::META),
        ("self_model", o.self_model_accuracy, thr::META),
        ("bench", o.bench_accuracy, thr::BENCH),
        ("bench_zs", o.bench_zs_accuracy, thr::BENCH_ZS),
    ];
    // (composition gets dedicated exam phase via preferred_exam_phase + habitat)
    pairs
        .iter()
        .map(|(n, v, t)| OrganAcc {
            name: (*n).into(),
            value: *v,
            thr: *t,
            weak: *v < *t,
        })
        .collect()
}

/// Stimulus strength from deficit × teach_gain.
fn boost_str(acc: f64, thr: f64, teach_gain: f64) -> f64 {
    let deficit = (thr - acc).max(0.0);
    (0.55 + deficit * 1.5) * teach_gain.clamp(0.4, 1.4)
}

/// Inject organ-specific teach when weak.
fn remediate(o: &mut Organism, organ: &str, strength: f64) {
    let s = strength.clamp(0.4, 1.8);
    match organ {
        "cognitive" => {
            o.teach(
                "ACCURACY_HOME: cognition — re-check codon table + regime signs; practice exam_cognition patterns.",
                "homeo_cog",
                s,
            );
            o.cognitive_accuracy = (o.cognitive_accuracy + 0.02 * s.min(1.0)).min(0.95);
        }
        "language" => {
            o.teach(
                "ACCURACY_HOME: language — grounded answers; plastic pathways; say regime and tools clearly.",
                "homeo_lang",
                s,
            );
            o.language_accuracy = (o.language_accuracy + 0.015 * s.min(1.0)).min(0.95);
            o.language_skill = (o.language_skill + 0.02).min(1.0);
        }
        "code" => {
            o.teach(
                "ACCURACY_HOME: code — python sandbox exams; prefer small verified snippets over freestyle.",
                "homeo_code",
                s,
            );
            o.code_skill = (o.code_skill + 0.025).min(1.0);
        }
        "vision" => {
            o.teach(
                "ACCURACY_HOME: vision — classify brightness/regime images; warm_emergence vs gray_neutral.",
                "homeo_vision",
                s,
            );
            o.vision_skill = (o.vision_skill + 0.025).min(1.0);
        }
        "knowledge" => {
            o.teach(
                "ACCURACY_HOME: knowledge mastery ≥85% — encode FACT[id], recall by id/keys, closed-book until bar.",
                "homeo_know",
                s,
            );
            o.knowledge_skill = (o.knowledge_skill + 0.05).min(1.0);
            if o.knowledge_accuracy < 0.40 {
                o.knowledge_accuracy = 0.40;
            }
        }
        "collective" => {
            o.teach(
                "ACCURACY_HOME: collective — pack votes + multi-hop; align with shared library.",
                "homeo_coll",
                s,
            );
        }
        "audio" => {
            o.teach(
                "ACCURACY_HOME: audio mastery ≥75% — seed battery high→emergence mid→neutral low→damping; then drive.",
                "homeo_audio",
                s,
            );
            o.audio_skill = (o.audio_skill + 0.05).min(1.0);
            if o.audio_accuracy < 0.40 {
                o.audio_accuracy = 0.40;
            }
        }
        "video" => {
            o.teach(
                "ACCURACY_HOME: video mastery ≥75% — synth temporal frames first; drive ffmpeg with fallback.",
                "homeo_video",
                s,
            );
            o.video_skill = (o.video_skill + 0.05).min(1.0);
            if o.video_accuracy < 0.40 {
                o.video_accuracy = 0.40;
            }
        }
        "js" => {
            o.teach(
                "ACCURACY_HOME: js mastery ≥85% — challenge battery + tools_js forge; node sandbox only.",
                "homeo_js",
                s,
            );
            o.js_skill = (o.js_skill + 0.05).min(1.0);
            if o.js_accuracy < 0.40 {
                o.js_accuracy = 0.40;
            }
        }
        "composition" => {
            o.teach(
                "ACCURACY_HOME: composition — multi-step tool chains; don't skip intermediate ground.",
                "homeo_comp",
                s,
            );
            o.composition_accuracy = (o.composition_accuracy + 0.02 * s.min(1.0)).min(0.9);
        }
        "code_l2" => {
            o.teach(
                "ACCURACY_HOME: code_l2 — multi-file practice under workspace; build small modules.",
                "homeo_l2",
                s,
            );
            // Soft open so organ is not stuck at absolute zero forever
            if o.code_l2_accuracy < 0.08 {
                o.code_l2_accuracy = 0.08;
            }
        }
        "gen_acc" => {
            o.teach(
                "ACCURACY_HOME: gen_acc mastery ≥75% — novel phrasings; plastic pathways transfer under exam pressure.",
                "homeo_gen",
                s,
            );
            o.lang_cortex.generalize_accuracy =
                (o.lang_cortex.generalize_accuracy + 0.03 * s.min(1.0)).min(0.92);
            o.lang_cortex.plasticity = (o.lang_cortex.plasticity + 0.015).min(0.40);
        }
        "self_model" => {
            o.teach(
                "ACCURACY_HOME: self_model mastery ≥70% — rails no auto-Rust; identity append-only; body e/l/r; gen.",
                "homeo_self",
                s,
            );
            if o.self_model_accuracy < 0.40 {
                o.self_model_accuracy = 0.40;
            }
        }
        "bench" => {
            // Homeostasis: stimuli + pressure only — never free accuracy grades
            o.teach(
                "ACCURACY_HOME: bench study→test under FSOT load; error-queue; CHEW when pressure high.",
                "homeo_bench",
                s,
            );
            o.pressure = (o.pressure + 0.03 * s.min(1.0)).min(1.5);
            o.recent_hits = (o.recent_hits + 0.05 * s.min(1.0)).min(5.0);
        }
        "bench_zs" => {
            // Allostatic: weak zs → pressure, plasticity, organ skill — not free zs points
            o.teach(
                "ACCURACY_HOME: zero-shot held-out — transfer schemas, motor under fitness, pack broadcast, deeper chew.",
                "homeo_zs",
                s,
            );
            o.pressure = (o.pressure + 0.06 * s.min(1.2)).min(1.5);
            o.lang_cortex.plasticity = (o.lang_cortex.plasticity + 0.02).min(0.45);
            o.code_skill = (o.code_skill + 0.02 * s.min(1.0)).min(1.0);
            o.knowledge_skill = (o.knowledge_skill + 0.02 * s.min(1.0)).min(1.0);
            o.recent_hits = (o.recent_hits + 0.08 * s.min(1.0)).min(5.0);
        }
        _ => {}
    }
    o.pressure = (o.pressure + 0.01).min(1.4);
    o.recompute_intelligence_pub();
}

/// Run pack accuracy homeostasis + write audit snapshot.
pub fn pack_accuracy_homeostasis(
    pack: &mut [Organism],
    tick: u64,
    generation: u32,
    teach_gain: f64,
) -> AccuracyAudit {
    let mut orgs = Vec::new();
    let mut weak_total = 0usize;
    let mut mean_sum = 0.0;

    for o in pack.iter_mut() {
        let organs = organ_list(o);
        let mut boosted = Vec::new();
        for oa in &organs {
            if oa.weak {
                let s = boost_str(oa.value, oa.thr, teach_gain);
                remediate(o, &oa.name, s);
                boosted.push(format!("{}:{:.0}%→stim{:.2}", oa.name, oa.value * 100.0, s));
                weak_total += 1;
            }
        }
        let mean = o.mean_accuracy();
        mean_sum += mean;
        orgs.push(OrgAudit {
            id: o.id.clone(),
            mean_acc: mean,
            fitness: o.fitness_score(),
            weak_count: organs.iter().filter(|a| a.weak).count(),
            organs,
            boosted,
        });
    }

    let n = pack.len().max(1) as f64;
    let audit = AccuracyAudit {
        tick,
        generation,
        pack_mean: mean_sum / n,
        pack_weak_organs: weak_total,
        orgs,
        note: "Weak organs get teach+skill stimulus. Accuracy still earned in exams — this is feed, not free grades."
            .into(),
    };
    let _ = write_audit(&audit);
    audit
}

fn write_audit(a: &AccuracyAudit) -> std::io::Result<()> {
    let dir = export_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("ACCURACY_AUDIT.json");
    let tmp = dir.join("ACCURACY_AUDIT.json.tmp");
    let json = serde_json::to_string_pretty(a)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&tmp, json)?;
    fs::rename(tmp, path)?;
    Ok(())
}

pub fn summary_line(a: &AccuracyAudit) -> String {
    let weak_orgs = a.orgs.iter().filter(|o| o.weak_count > 0).count();
    format!(
        "accuracy_homeo pack_mean={:.0}% weak_cells={} orgs_needing_boost={}/{} gen={}",
        a.pack_mean * 100.0,
        a.pack_weak_organs,
        weak_orgs,
        a.orgs.len(),
        a.generation
    )
}

pub fn human_report(a: &AccuracyAudit) -> String {
    let mut s = format!(
        "Accuracy audit t={} gen={} · pack_mean={:.1}% · weak_organ_cells={}\n\
         Thresholds: core≥{:.0}% media≥{:.0}% script≥{:.0}% meta≥{:.0}% l2≥{:.0}%\n\
         {}\n\n",
        a.tick,
        a.generation,
        a.pack_mean * 100.0,
        a.pack_weak_organs,
        thr::CORE * 100.0,
        thr::MEDIA * 100.0,
        thr::SCRIPT * 100.0,
        thr::META * 100.0,
        thr::L2 * 100.0,
        a.note
    );
    for o in &a.orgs {
        s.push_str(&format!(
            "· {} mean={:.0}% fit={:.1} weak={}\n",
            o.id,
            o.mean_acc * 100.0,
            o.fitness,
            o.weak_count
        ));
        for oa in o.organs.iter().filter(|x| x.weak) {
            s.push_str(&format!(
                "    - {}: {:.0}% < thr {:.0}%\n",
                oa.name,
                oa.value * 100.0,
                oa.thr * 100.0
            ));
        }
        if !o.boosted.is_empty() {
            s.push_str(&format!("    stim: {}\n", o.boosted.join(", ")));
        }
    }
    s
}

/// Which heavy-exam phase index to prefer for a weak organ (habitat rotation).
/// 0 code/l2 | 1 vision | 2 tool/js | 3 language/gen | 4 collective | 5 knowledge
/// 6 audio | 7 video | 8 compose/propose | 9 self_model | 10 bench (LLM surface)
pub fn preferred_exam_phase(o: &Organism) -> Option<usize> {
    let candidates = [
        (0usize, o.code_accuracy, thr::CORE),
        (0, o.code_l2_accuracy, thr::L2),
        (1, o.vision_accuracy, thr::CORE),
        (2, o.js_accuracy, thr::JS),
        (3, o.language_accuracy, thr::CORE),
        (3, o.lang_cortex.generalize_accuracy, thr::META),
        (4, o.collective_accuracy, thr::SOCIAL),
        (5, o.knowledge_accuracy, thr::KNOWLEDGE),
        (6, o.audio_accuracy, thr::MEDIA),
        (7, o.video_accuracy, thr::MEDIA),
        // phase 8: composition (also propose) — offensive target
        (8, o.composition_accuracy, thr::SCRIPT),
        (9, o.self_model_accuracy, thr::META),
        (10, o.bench_accuracy, thr::BENCH),
        (10, o.bench_zs_accuracy, thr::BENCH_ZS),
    ];
    let mut best: Option<(usize, f64)> = None;
    for (phase, acc, t) in candidates {
        if acc < t {
            let deficit = t - acc;
            if best.map(|(_, d)| deficit > d).unwrap_or(true) {
                best = Some((phase, deficit));
            }
        }
    }
    best.map(|(p, _)| p)
}
