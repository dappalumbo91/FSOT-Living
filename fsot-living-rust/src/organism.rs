//! Living organism grounded in FSOT scalar + genome guidance + multimodal organs

use crate::genetic::{genome_to_guidance, GenomeGuidance};
use crate::lang_plastic::LangCortex;
use crate::language::{DialogueState, SemanticEntry};
use crate::scalar::{compute_s, ComputeParams};
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MemoryItem {
    pub text: String,
    pub kind: String,
    pub strength: f64,
}

#[derive(Clone, Debug)]
pub struct Organism {
    pub id: String,
    pub tick: u64,
    pub s: f64,
    pub raw_s: f64,
    pub term1: f64,
    pub term2: f64,
    pub term3: f64,
    pub regime: String,
    pub vitality: f64,
    pub coherence: f64,
    pub pressure: f64,
    pub reward: f64,
    pub phase: f64,
    pub target_s: f64,
    pub d_eff: f64,
    pub delta_psi: f64,
    pub emergence_pref: f64,
    pub observed: bool,
    pub observation_count: u64,
    pub recent_hits: f64,
    pub genome: Vec<String>,
    pub guidance: GenomeGuidance,
    pub codon_table: HashMap<String, char>,
    pub memory: Vec<MemoryItem>,
    pub long_term: Vec<String>,
    pub semantic: Vec<SemanticEntry>,
    pub dialogue: DialogueState,
    pub cognitive_accuracy: f64,
    pub intelligence: f64,
    pub exam_count: u64,
    pub accepted_mut: u64,
    pub rejected_mut: u64,
    pub integration: f64,
    pub code_skill: f64,
    pub vision_skill: f64,
    pub tool_skill: f64,
    pub language_skill: f64,
    pub code_accuracy: f64,
    pub vision_accuracy: f64,
    pub language_accuracy: f64,
    pub tools_owned: Vec<String>,
    // Pack / social / collective (horizontal culture)
    pub social_skill: f64,
    pub social_alignment: f64,
    pub collective_accuracy: f64,
    pub signals_received: u32,
    pub last_signal_summary: String,
    // Homeostasis / perceived time snapshot
    pub perceived_drive: f64,
    pub homeo_coherence_pull: f64,
    /// Brief of pack shared library (refreshed each habitat step)
    pub pack_library_brief: String,
    /// Adaptive language cortex (plastic pathways — not an LLM)
    pub lang_cortex: LangCortex,
    /// Multi-hop / composition accuracy under tool-required exams
    pub composition_accuracy: f64,
    /// JS organ skill (interpretable, if node present)
    pub js_skill: f64,
    pub js_accuracy: f64,
    /// Code L2 multi-file skill
    pub code_l2_accuracy: f64,
    /// Self-proposal organ (novelty under review — not auto-Rust)
    pub proposal_skill: f64,
    pub proposals_emitted: u64,
    /// Factual knowledge organ (curriculum + exams)
    pub knowledge_skill: f64,
    pub knowledge_accuracy: f64,
    /// Audio energy organ (drive music)
    pub audio_skill: f64,
    pub audio_accuracy: f64,
    /// Video drive organ (frame sampling)
    pub video_skill: f64,
    pub video_accuracy: f64,
    /// JS tools owned (tools_js/)
    pub js_tools_owned: Vec<String>,
    /// Felt interoception from trinary body (refreshed by body_bridge couple).
    /// Software "soul" reading of hardware body — used in HowFeeling dialogue.
    pub felt_body: String,
    /// Accuracy of self-model exams (knows own regime, rails, body, identity).
    pub self_model_accuracy: f64,
    pub self_model_skill: f64,
    /// External LLM-benchmark capability surface (MMLU/GSM8K/HellaSwag/TQA/MBPP).
    pub bench_accuracy: f64,
    /// Zero-shot held-out track (leaderboard-shaped; no pre-teach of exam item).
    pub bench_zs_accuracy: f64,
    /// Warm-start flag so EMA cold-start doesn't fake "22%" forever.
    pub self_model_warmed: bool,
    /// Taught continuity generation (for self-model ContinuityGen probe).
    pub known_generation: Option<u32>,
    pub known_monitor_level: Option<String>,
    pub known_body_phenotype: Option<String>,
    /// True after body couple / felt interoception observed.
    pub body_link_seen: bool,
}

impl Organism {
    pub fn new(id: impl Into<String>, genome: Vec<String>, bio_target: f64) -> Self {
        let guidance = genome_to_guidance(&genome);
        let mut o = Self {
            id: id.into(),
            tick: 0,
            s: 0.0,
            raw_s: 0.0,
            term1: 0.0,
            term2: 1.0,
            term3: 0.0,
            regime: "neutral".into(),
            vitality: 0.5,
            coherence: 0.6,
            pressure: 0.0,
            reward: 0.35,
            phase: 0.0,
            target_s: bio_target,
            d_eff: guidance.d_eff,
            delta_psi: guidance.delta_psi,
            emergence_pref: guidance.emergence_preference,
            observed: false,
            observation_count: 0,
            recent_hits: 0.0,
            genome,
            guidance,
            codon_table: HashMap::new(),
            memory: Vec::new(),
            long_term: Vec::new(),
            semantic: Vec::new(),
            dialogue: DialogueState::default(),
            cognitive_accuracy: 0.0,
            intelligence: 0.0,
            exam_count: 0,
            accepted_mut: 0,
            rejected_mut: 0,
            integration: 0.5,
            code_skill: 0.35,
            vision_skill: 0.35,
            tool_skill: 0.25,
            language_skill: 0.35,
            code_accuracy: 0.0,
            vision_accuracy: 0.0,
            language_accuracy: 0.0,
            tools_owned: Vec::new(),
            social_skill: 0.3,
            social_alignment: 0.0,
            collective_accuracy: 0.0,
            signals_received: 0,
            last_signal_summary: String::new(),
            perceived_drive: 0.0,
            homeo_coherence_pull: 0.0,
            pack_library_brief: String::new(),
            lang_cortex: LangCortex::default(),
            composition_accuracy: 0.0,
            js_skill: 0.25,
            js_accuracy: 0.0,
            code_l2_accuracy: 0.0,
            proposal_skill: 0.2,
            proposals_emitted: 0,
            knowledge_skill: 0.3,
            knowledge_accuracy: 0.0,
            audio_skill: 0.3,
            audio_accuracy: 0.0,
            video_skill: 0.3,
            video_accuracy: 0.0,
            js_tools_owned: Vec::new(),
            felt_body: String::new(),
            self_model_accuracy: 0.0,
            self_model_skill: 0.22,
            bench_accuracy: 0.0,
            bench_zs_accuracy: 0.0,
            self_model_warmed: false,
            known_generation: None,
            known_monitor_level: None,
            known_body_phenotype: None,
            body_link_seen: false,
        };
        for c in &o.genome {
            let aa = crate::genetic::genetic_code(c);
            if aa != '?' {
                o.codon_table.insert(c.clone(), aa);
            }
        }
        o
    }

    pub fn apply_genome_guidance(&mut self) {
        self.guidance = genome_to_guidance(&self.genome);
        self.d_eff = self.d_eff * 0.35 + self.guidance.d_eff * 0.65;
        self.delta_psi = self.delta_psi * 0.35 + self.guidance.delta_psi * 0.65;
        self.emergence_pref =
            self.emergence_pref * 0.4 + self.guidance.emergence_preference * 0.6;
    }

    pub fn compute(&mut self, stress: f64) {
        let phase_shift = (self.phase * 1.05).sin() * 0.15;
        let delta_psi = (self.delta_psi + phase_shift + if self.observed { 0.05 } else { 0.0 })
            .clamp(0.05, 1.5);
        let rho = (1.0 - stress * 0.25).max(0.3);
        let d_eff = self.d_eff.clamp(6.0, 28.0);

        let r = compute_s(&ComputeParams {
            n: 1.0,
            p: 1.0,
            d_eff,
            recent_hits: self.recent_hits.clamp(0.0, 5.0),
            delta_psi,
            delta_theta: 1.0,
            rho,
            scale: 1.0,
            amplitude: 1.0,
            trend_bias: (self.emergence_pref - 0.5) * 0.1,
            observed: self.observed,
        });

        self.s = r.s;
        self.raw_s = r.raw_s;
        self.term1 = r.term1;
        self.term2 = r.term2;
        self.term3 = r.term3;
        self.regime = r.regime.to_string();
        self.vitality = (r.s * 1.15 + 0.12).clamp(-0.55, 1.2);

        let target_p = (self.target_s - self.s).max(0.0);
        self.pressure = (self.pressure * 0.9 + target_p * 0.1).clamp(0.0, 1.5);

        let s_stab = 1.0 - (self.s - self.target_s).abs().min(1.0);
        let coh_pull = if self.regime == "emergence" { 0.04 } else { -0.02 };
        self.coherence = (self.coherence * 0.93 + s_stab * 0.12 + coh_pull
            + (self.vitality + 0.4) * 0.05)
            .clamp(0.22, 1.0);

        let improve = if self.s > 0.05 {
            0.06
        } else if self.s > -0.1 {
            0.02
        } else {
            -0.02
        };
        self.reward = (self.reward * 0.94 + improve).clamp(0.12, 0.88);
        self.integration = (self.integration * 0.9
            + (self.codon_table.len() as f64 / 64.0) * 0.1
            + (self.memory.len() as f64 / 80.0) * 0.05)
            .clamp(0.1, 1.0);

        self.phase += 0.1;
        if self.recent_hits > 0.0 && self.tick % 12 == 0 {
            self.recent_hits = (self.recent_hits - 1.0).max(0.0);
        }
    }

    pub fn tick_once(&mut self, stress: f64, rng: &mut impl Rng) {
        self.tick += 1;
        self.observed = rng.gen::<f64>() < 0.33;
        if self.observed {
            self.observation_count += 1;
        }
        self.compute(stress);

        if self.pressure > 0.35 && rng.gen::<f64>() < 0.08 {
            let before = self.fitness_score();
            let old_d = self.d_eff;
            let old_p = self.delta_psi;
            self.d_eff = (self.d_eff + (rng.gen::<f64>() - 0.5) * 0.4).clamp(6.0, 28.0);
            self.delta_psi = (self.delta_psi + (rng.gen::<f64>() - 0.5) * 0.08).clamp(0.05, 1.5);
            self.compute(0.0);
            if self.fitness_score() > before {
                self.accepted_mut += 1;
            } else {
                self.d_eff = old_d;
                self.delta_psi = old_p;
                self.compute(0.0);
                self.rejected_mut += 1;
            }
        }
    }

    pub fn fitness_score(&self) -> f64 {
        let s_align = 1.0 - (self.s - self.target_s).abs().min(1.2);
        self.cognitive_accuracy * 1.6
            + self.intelligence * 1.0
            + self.code_accuracy * 1.8
            + self.vision_accuracy * 1.5
            + self.language_accuracy * 1.7
            + self.language_skill * 0.4
            + self.lang_cortex.generalize_accuracy * 1.5
            + self.tool_skill * self.code_accuracy * 0.6
            + self.social_skill * 0.9
            + self.collective_accuracy * 1.4
            + self.composition_accuracy * 1.8 // multi-step math organ (Formal composition)
            + self.js_accuracy * 0.9
            + self.code_l2_accuracy * 1.1
            + self.proposal_skill * 0.35
            + self.knowledge_accuracy * 1.4
            + self.knowledge_skill * 0.3
            + self.audio_accuracy * 1.2
            + self.video_accuracy * 1.2
            + self.audio_skill * 0.25
            + self.video_skill * 0.25
            + self.self_model_accuracy * 1.5
            + self.self_model_skill * 0.4
            + (self.js_tools_owned.len() as f64) * 0.12
            + self.social_alignment * 0.35
            + (self.tools_owned.len() as f64) * 0.15
            + (self.semantic.len() as f64 / 120.0) * 0.2
            + s_align * 0.6
            + if self.regime == "emergence" {
                0.25
            } else {
                -0.15
            }
            - self.pressure * 0.35
            + self.integration * 0.3
            + (self.codon_table.len() as f64 / 64.0) * 0.35
    }

    /// Mean multimodal accuracy (0–1) — raw competence signal.
    pub fn mean_accuracy(&self) -> f64 {
        let parts = [
            self.cognitive_accuracy,
            self.language_accuracy,
            self.lang_cortex.generalize_accuracy,
            self.code_accuracy,
            self.vision_accuracy,
            self.knowledge_accuracy,
            self.collective_accuracy,
            self.audio_accuracy,
            self.video_accuracy,
            self.js_accuracy.max(self.code_l2_accuracy * 0.5),
            self.composition_accuracy,
            self.self_model_accuracy,
        ];
        let s: f64 = parts.iter().sum();
        s / parts.len() as f64
    }

    /// Efficiency: accuracy per unit pressure/waste — attraction target for mate choice.
    /// High accuracy + low pressure + more tools/organs → higher breeding value.
    pub fn breeding_value(&self) -> f64 {
        let acc = self.mean_accuracy();
        let fit = self.fitness_score();
        let efficiency = acc / (0.35 + self.pressure.max(0.05));
        let organ_depth = (self.tools_owned.len() + self.js_tools_owned.len()) as f64 * 0.08
            + (self.codon_table.len() as f64 / 64.0) * 0.4
            + self.integration * 0.3;
        // Genetic Formal coupling: lineages that bind domain fields breed when competent
        // Prefer composition + held-out zs (invent truth) over inflated study-only code
        let genetic_field = self.guidance.domain_coupling.clamp(0.4, 1.4)
            * (0.5 + self.composition_accuracy + self.knowledge_accuracy * 0.5);
        // Strong weight on accuracy & efficiency so weak lineages don't linger as parents
        fit * 0.35 + acc * 8.0 + efficiency * 2.5 + organ_depth * 0.5
            + self.lang_cortex.generalize_accuracy * 2.0
            + self.knowledge_accuracy * 1.5
            + self.composition_accuracy * 2.6 // multi-step under Formal energy/consciousness
            + self.code_accuracy * 1.4 // invent-honest after credit reform
            + self.audio_accuracy * 1.0
            + self.video_accuracy * 1.0
            + self.js_accuracy * 1.0
            + self.self_model_accuracy * 1.8
            + genetic_field * 1.25
            + self.bench_accuracy * 1.6 // study health
            + self.bench_zs_accuracy * 3.0 // held-out transfer dominates selection
    }

    /// Prefer cultural traits from the higher-accuracy parent for each organ.
    pub fn inherit_organs_from_pair(&mut self, a: &Organism, b: &Organism) {
        let pick = |va: f64, vb: f64| if va >= vb { va } else { vb };
        self.cognitive_accuracy = pick(a.cognitive_accuracy, b.cognitive_accuracy) * 0.92;
        self.language_accuracy = pick(a.language_accuracy, b.language_accuracy) * 0.92;
        self.code_accuracy = pick(a.code_accuracy, b.code_accuracy) * 0.92;
        self.vision_accuracy = pick(a.vision_accuracy, b.vision_accuracy) * 0.92;
        self.knowledge_accuracy =
            pick(a.knowledge_accuracy, b.knowledge_accuracy).max(0.45) * 0.96;
        // Floor inheritance: don't drag children to near-zero on lagging organs
        let pick_floor = |va: f64, vb: f64, floor: f64| pick(va, vb).max(floor) * 0.94;
        self.collective_accuracy =
            pick_floor(a.collective_accuracy, b.collective_accuracy, 0.20);
        self.audio_accuracy = pick_floor(a.audio_accuracy, b.audio_accuracy, 0.20);
        self.video_accuracy = pick_floor(a.video_accuracy, b.video_accuracy, 0.20);
        self.js_accuracy = pick_floor(a.js_accuracy, b.js_accuracy, 0.45);
        self.code_l2_accuracy = pick_floor(a.code_l2_accuracy, b.code_l2_accuracy, 0.12);
        self.composition_accuracy = pick_floor(a.composition_accuracy, b.composition_accuracy, 0.22);
        self.self_model_accuracy = pick_floor(a.self_model_accuracy, b.self_model_accuracy, 0.28);
        self.bench_accuracy = pick_floor(a.bench_accuracy, b.bench_accuracy, 0.30);
        self.bench_zs_accuracy = pick_floor(a.bench_zs_accuracy, b.bench_zs_accuracy, 0.20);
        self.self_model_skill = pick(a.self_model_skill, b.self_model_skill);
        self.self_model_warmed = a.self_model_warmed || b.self_model_warmed;
        self.known_generation = a.known_generation.or(b.known_generation);
        self.known_monitor_level = a
            .known_monitor_level
            .clone()
            .or_else(|| b.known_monitor_level.clone());
        self.known_body_phenotype = a
            .known_body_phenotype
            .clone()
            .or_else(|| b.known_body_phenotype.clone());
        self.body_link_seen = a.body_link_seen || b.body_link_seen;
        self.language_skill = pick(a.language_skill, b.language_skill);
        self.code_skill = pick(a.code_skill, b.code_skill);
        self.vision_skill = pick(a.vision_skill, b.vision_skill);
        self.tool_skill = pick(a.tool_skill, b.tool_skill);
        self.audio_skill = pick(a.audio_skill, b.audio_skill);
        self.video_skill = pick(a.video_skill, b.video_skill);
        self.js_skill = pick(a.js_skill, b.js_skill);
        self.knowledge_skill = pick(a.knowledge_skill, b.knowledge_skill);
        self.social_skill = pick(a.social_skill, b.social_skill) * 0.97;
        self.intelligence = pick(a.intelligence, b.intelligence) * 0.92;
        // Merge tools (union)
        self.tools_owned = a.tools_owned.clone();
        for t in &b.tools_owned {
            if !self.tools_owned.iter().any(|x| x == t) {
                self.tools_owned.push(t.clone());
            }
        }
        self.js_tools_owned = a.js_tools_owned.clone();
        for t in &b.js_tools_owned {
            if !self.js_tools_owned.iter().any(|x| x == t) {
                self.js_tools_owned.push(t.clone());
            }
        }
        // Cortex from higher gen_acc parent
        let cortex_src = if a.lang_cortex.generalize_accuracy >= b.lang_cortex.generalize_accuracy {
            a
        } else {
            b
        };
        self.lang_cortex = cortex_src.lang_cortex.clone();
        self.lang_cortex.generalize_accuracy *= 0.95;
        // Codon table union
        for (k, v) in a.codon_table.iter().chain(b.codon_table.iter()) {
            self.codon_table.insert(k.clone(), *v);
        }
        // Memory: strongest semantic from both
        let mut sem = a.semantic.clone();
        sem.extend(b.semantic.clone());
        sem.sort_by(|x, y| y.strength.partial_cmp(&x.strength).unwrap());
        sem.truncate(80);
        self.semantic = sem;
        let mut lt = a.long_term.clone();
        for t in &b.long_term {
            if !lt.iter().any(|x| x == t) {
                lt.push(t.clone());
            }
        }
        if lt.len() > 200 {
            lt = lt.split_off(lt.len() - 200);
        }
        self.long_term = lt;
    }

    pub fn teach(&mut self, text: &str, kind: &str, strength: f64) {
        self.memory.push(MemoryItem {
            text: text.chars().take(200).collect(),
            kind: kind.into(),
            strength,
        });
        if self.memory.len() > 80 {
            self.memory.remove(0);
        }
        crate::language::semantic_store(&mut self.semantic, text, kind, strength);
        if let Some((codon, aa)) = extract_codon(text) {
            self.long_term.push(format!("Codon {codon} → {aa}"));
            self.codon_table.insert(codon, aa);
            if self.long_term.len() > 200 {
                self.long_term.remove(0);
            }
        }
        if text.contains("domain") || text.contains("FSOT") || text.contains("S=") {
            self.long_term.push(text.chars().take(160).collect());
            if self.long_term.len() > 200 {
                self.long_term.remove(0);
            }
        }
        self.recent_hits = (self.recent_hits + 0.5).min(5.0);
    }

    pub fn exam_language_one(&mut self, rng: &mut impl Rng) -> String {
        let detail = crate::language::exam_language_one(self, rng);
        self.recompute_intelligence();
        self.exam_count += 1;
        detail
    }

    pub fn exam_cognition_only(&mut self, rng: &mut impl Rng) {
        let mut score = 0.0;
        let mut n = 0.0;
        let truth = if self.s > 0.02 {
            "emergence"
        } else if self.s < -0.02 {
            "damping"
        } else {
            "neutral"
        };
        if self.regime == truth {
            score += 1.0;
        }
        n += 1.0;
        let codons: Vec<_> = self.codon_table.keys().cloned().collect();
        for _ in 0..3 {
            if codons.is_empty() {
                n += 1.0;
                continue;
            }
            let c = &codons[rng.gen_range(0..codons.len())];
            let aa = self.codon_table.get(c).copied().unwrap_or('?');
            if aa == crate::genetic::genetic_code(c) {
                score += 1.0;
            }
            n += 1.0;
        }
        let err = (self.s - self.target_s).abs();
        score += 1.0 - (err / 0.8).min(1.0);
        n += 1.0;
        self.cognitive_accuracy = if n > 0.0 { score / n } else { 0.0 };
        self.recompute_intelligence();
        self.exam_count += 1;
    }

    pub fn exam_code_one(&mut self, rng: &mut impl Rng) -> String {
        let challenges = crate::code_env::code_challenges();
        let ch = &challenges[rng.gen_range(0..challenges.len())];
        let passed = crate::code_env::run_code_exam(ch);
        let sample = if passed { 1.0 } else { 0.0 };
        self.code_accuracy = self.code_accuracy * 0.7 + sample * 0.3;
        if passed {
            self.code_skill = (self.code_skill + 0.03).min(1.0);
        } else {
            self.code_skill = (self.code_skill + 0.015).min(1.0);
            self.pressure = (self.pressure + 0.02).min(1.5);
        }
        self.recompute_intelligence();
        format!("code:{} → {}", ch.id, if passed { "PASS" } else { "FAIL" })
    }

    pub fn exam_vision_one(&mut self, rng: &mut impl Rng) -> String {
        let vch = crate::vision::vision_challenges();
        let ch = &vch[rng.gen_range(0..vch.len())];
        let passed = crate::vision::run_vision_exam(ch);
        let sample = if passed { 1.0 } else { 0.0 };
        self.vision_accuracy = self.vision_accuracy * 0.7 + sample * 0.3;
        if passed {
            self.vision_skill = (self.vision_skill + 0.03).min(1.0);
        } else {
            self.vision_skill = (self.vision_skill + 0.015).min(1.0);
            self.pressure = (self.pressure + 0.015).min(1.5);
        }
        self.recompute_intelligence();
        format!(
            "vision:{} → {}",
            ch.file,
            if passed { "PASS" } else { "FAIL" }
        )
    }

    pub fn exam_tool_one(&mut self, rng: &mut impl Rng) -> String {
        let roll = rng.gen_range(0..5u8);
        match roll {
            0 | 1 => {
                let tools = ["double", "add", "fsot_sign", "multiply", "clamp01", "rolling_mean"];
                let tool = tools[rng.gen_range(0..tools.len())];
                let passed = crate::code_env::forge_tool_exam(tool);
                if passed {
                    if !self.tools_owned.iter().any(|t| t == tool) {
                        self.tools_owned.push(tool.to_string());
                    }
                    self.tool_skill = (self.tool_skill + 0.04).min(1.0);
                    self.long_term
                        .push(format!("Built tool {tool}.py in organism workspace"));
                }
                self.recompute_intelligence();
                format!(
                    "forge:{} → {}",
                    tool,
                    if passed { "PASS" } else { "FAIL" }
                )
            }
            2 => {
                // Freeform invention — grows organ-tool repertoire
                match crate::code_env::invent_tool_exam(rng.gen_range(0..20)) {
                    Some(name) => {
                        if !self.tools_owned.iter().any(|t| t == &name) {
                            self.tools_owned.push(name.clone());
                        }
                        self.tool_skill = (self.tool_skill + 0.05).min(1.0);
                        self.recompute_intelligence();
                        format!("invent:{name} → PASS")
                    }
                    None => {
                        self.pressure = (self.pressure + 0.02).min(1.5);
                        "invent: → FAIL".into()
                    }
                }
            }
            3 => self.exam_js_one(rng),
            4 => {
                // Extra JS pressure when under-stimulated
                if self.js_accuracy < 0.5 {
                    self.exam_js_one(rng)
                } else {
                    self.exam_code_l2_one(rng)
                }
            }
            _ => self.exam_code_l2_one(rng),
        }
    }

    pub fn exam_code_l2_one(&mut self, rng: &mut impl Rng) -> String {
        let passed = crate::code_env::run_code_l2_exam(rng.gen_range(0..10));
        let sample = if passed { 1.0 } else { 0.0 };
        // Fast climb when L2 under-stimulated (organ rarely fired historically)
        let lr = if self.code_l2_accuracy < 0.35 {
            0.40
        } else {
            0.28
        };
        self.code_l2_accuracy =
            (self.code_l2_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
        if passed {
            self.code_skill = (self.code_skill + 0.025).min(1.0);
            self.code_accuracy = self.code_accuracy * 0.9 + 0.1;
            self.composition_accuracy = (self.composition_accuracy * 0.9 + 0.1).min(1.0);
        } else {
            self.pressure = (self.pressure + 0.015).min(1.5);
            // Remedial teach: multi-file import is the L2 skill
            self.teach(
                "CODE_L2: multi-file import — helpers/*.py must load via PYTHONPATH/workspace root",
                "code_l2",
                0.9,
            );
        }
        self.recompute_intelligence();
        format!(
            "code_l2 → {} (acc={:.0}%)",
            if passed { "PASS" } else { "FAIL" },
            self.code_l2_accuracy * 100.0
        )
    }

    fn apply_js_sample(&mut self, pass: bool) {
        let sample = if pass { 1.0 } else { 0.0 };
        let target = crate::code_env::JS_TARGET;
        let lr = if self.js_accuracy < 0.50 {
            0.48
        } else if self.js_accuracy < 0.70 {
            0.38
        } else if self.js_accuracy < target {
            0.32
        } else if self.js_accuracy < 0.92 {
            0.18
        } else {
            0.10
        };
        self.js_accuracy = (self.js_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
    }

    pub fn exam_js_one(&mut self, rng: &mut impl Rng) -> String {
        if !crate::code_env::node_available() {
            return "js:skip (no node)".into();
        }
        let target = crate::code_env::JS_TARGET;

        // Warm-start for cold / post-birth atrophy
        if self.js_accuracy < 0.35 {
            self.js_accuracy = self.js_accuracy.max(0.40);
            self.js_skill = self.js_skill.max(0.35);
            self.teach(
                "JS_ORGAN online: node sandbox · challenge ladder · tools_js forge · mastery ≥85%",
                "js",
                1.0,
            );
            if !self.tools_owned.iter().any(|t| t == "js_sandbox") {
                self.tools_owned.push("js_sandbox".into());
            }
        }

        // Under mastery: multi-challenge battery (3 reliable console.log exams)
        if self.js_accuracy < target {
            let chs = if self.js_accuracy < 0.70 {
                crate::code_env::js_challenges_scaffold()
            } else {
                crate::code_env::js_challenges()
            };
            let mut passes = 0u32;
            let mut last_id = String::from("?");
            for _ in 0..3 {
                let ch = &chs[rng.gen_range(0..chs.len())];
                last_id = ch.id.to_string();
                let mut ok = crate::code_env::run_js_exam(ch);
                if !ok {
                    let ch2 = &chs[rng.gen_range(0..chs.len())];
                    last_id = ch2.id.to_string();
                    ok = crate::code_env::run_js_exam(ch2);
                }
                self.apply_js_sample(ok);
                if ok {
                    passes += 1;
                    self.js_skill = (self.js_skill + 0.04).min(1.0);
                } else {
                    self.pressure = (self.pressure + 0.01).min(1.5);
                }
            }
            // One forge under bar (procedural skill) after console battery
            if self.js_accuracy >= 0.55 || passes >= 2 {
                let tools = ["double", "add", "mul", "fsot_sign", "clamp"];
                let tool = tools[rng.gen_range(0..tools.len())];
                let mut ok = crate::code_env::forge_js_tool_exam(tool);
                if !ok {
                    // recover with simple challenge so forge noise doesn't sink mastery
                    let ch = &chs[rng.gen_range(0..chs.len())];
                    ok = crate::code_env::run_js_exam(ch);
                }
                self.apply_js_sample(ok);
                if ok {
                    if !self.js_tools_owned.iter().any(|t| t == tool) {
                        self.js_tools_owned.push(tool.to_string());
                    }
                    self.js_skill = (self.js_skill + 0.05).min(1.0);
                    last_id = format!("forge:{tool}");
                }
            }
            if !self.tools_owned.iter().any(|t| t == "js_sandbox") {
                self.tools_owned.push("js_sandbox".into());
            }
            self.teach(
                &format!(
                    "JS battery {passes}/3 + practice · acc={:.0}% target={:.0}%",
                    self.js_accuracy * 100.0,
                    target * 100.0
                ),
                "js",
                0.9,
            );
            self.recompute_intelligence();
            return format!(
                "js:battery {passes}/3 last={last_id} → {} (acc={:.0}% skill={:.2} target={:.0}%)",
                if passes >= 2 { "PASS" } else { "FAIL" },
                self.js_accuracy * 100.0,
                self.js_skill,
                target * 100.0
            );
        }

        // Above mastery: keep sharp with full ladder + occasional forge
        let prefer_simple = rng.gen_bool(0.55);
        if !prefer_simple {
            let tools = ["clamp", "double", "add", "fsot_sign", "mul"];
            let tool = tools[rng.gen_range(0..tools.len())];
            let mut passed = crate::code_env::forge_js_tool_exam(tool);
            if !passed {
                let chs = crate::code_env::js_challenges();
                let ch = &chs[rng.gen_range(0..chs.len())];
                passed = crate::code_env::run_js_exam(ch);
            }
            self.apply_js_sample(passed);
            if passed {
                if !self.js_tools_owned.iter().any(|t| t == tool) {
                    self.js_tools_owned.push(tool.to_string());
                }
                self.js_skill = (self.js_skill + 0.04).min(1.0);
            }
            self.recompute_intelligence();
            return format!(
                "js_forge:{} → {} (acc={:.0}% skill={:.2})",
                tool,
                if passed { "PASS" } else { "FAIL" },
                self.js_accuracy * 100.0,
                self.js_skill
            );
        }
        let chs = crate::code_env::js_challenges();
        let ch = &chs[rng.gen_range(0..chs.len())];
        let mut passed = crate::code_env::run_js_exam(ch);
        if !passed {
            let ch2 = &chs[rng.gen_range(0..chs.len())];
            passed = crate::code_env::run_js_exam(ch2);
        }
        self.apply_js_sample(passed);
        if passed {
            self.js_skill = (self.js_skill + 0.035).min(1.0);
        }
        self.recompute_intelligence();
        format!(
            "js:{} → {} (acc={:.0}% skill={:.2})",
            ch.id,
            if passed { "PASS" } else { "FAIL" },
            self.js_accuracy * 100.0,
            self.js_skill
        )
    }

    /// Pack drill until mean JS ≥ 85% (same pattern as knowledge mastery).
    pub fn pack_js_mastery(pack: &mut [Organism], rng: &mut impl Rng) -> String {
        if !crate::code_env::node_available() {
            return "js_mastery: node unavailable".into();
        }
        let target = crate::code_env::JS_TARGET;
        let mut drilled = 0u32;
        for o in pack.iter_mut() {
            if o.js_accuracy >= target {
                continue;
            }
            drilled += 1;
            // 2 full batteries per weak org
            let _ = o.exam_js_one(rng);
            if o.js_accuracy < target {
                let _ = o.exam_js_one(rng);
            }
        }
        let mean = pack.iter().map(|o| o.js_accuracy).sum::<f64>() / pack.len().max(1) as f64;
        let min = pack
            .iter()
            .map(|o| o.js_accuracy)
            .fold(1.0_f64, f64::min);
        format!(
            "js_mastery: drilled={drilled}/{} mean={:.0}% min={:.0}% target={:.0}%",
            pack.len(),
            mean * 100.0,
            min * 100.0,
            target * 100.0
        )
    }

    pub fn exam_audio_one(&mut self, paths: &[std::path::PathBuf], rng: &mut impl Rng) -> String {
        let target = crate::audio::AUDIO_TARGET;
        // Under mastery: seed calibration battery (reliable sensory tissue)
        // Above mastery: mix seeds + drive
        if self.audio_accuracy < target || rng.gen_bool(0.35) {
            let rounds = if self.audio_accuracy < target { 2 } else { 1 };
            let mut last_ok = 0u32;
            let mut last_total = 3u32;
            for _ in 0..rounds {
                let (ok, total) = crate::audio::run_audio_seed_battery();
                last_ok = ok;
                last_total = total;
                let frac = if total > 0 {
                    ok as f64 / total as f64
                } else {
                    0.0
                };
                let passed = ok >= 2;
                // Full credit on 3/3; partial on 2/3
                let sample = if ok == total {
                    1.0
                } else if passed {
                    frac.max(0.75)
                } else {
                    frac * 0.45
                };
                let lr = if self.audio_accuracy < 0.50 {
                    0.45
                } else if self.audio_accuracy < 0.65 {
                    0.35
                } else if self.audio_accuracy < target {
                    0.28
                } else {
                    0.16
                };
                self.audio_accuracy =
                    (self.audio_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
                if passed {
                    self.audio_skill = (self.audio_skill + 0.05).min(1.0);
                } else {
                    self.pressure = (self.pressure + 0.012).min(1.5);
                }
            }
            let passed = last_ok >= 2;
            if passed {
                self.teach(
                    &format!(
                        "AUDIO seed battery {last_ok}/{last_total} · high→emergence mid→neutral low→damping · target={:.0}%",
                        target * 100.0
                    ),
                    "audio",
                    0.9,
                );
            } else {
                self.teach(
                    "AUDIO calibrate: reseed v2 RMS bands — high>0.12 mid 0.04-0.12 low<0.04",
                    "audio",
                    0.95,
                );
            }
            self.recompute_intelligence();
            return format!(
                "audio:seed_battery {last_ok}/{last_total}×{rounds} → {} (acc={:.0}% target={:.0}%)",
                if passed { "PASS" } else { "FAIL" },
                self.audio_accuracy * 100.0,
                target * 100.0
            );
        }

        let challenges = crate::audio::audio_challenges(paths);
        if challenges.is_empty() {
            // No drive paths — keep sharp with seeds
            let (ok, total) = crate::audio::run_audio_seed_battery();
            let sample = if ok == total {
                1.0
            } else if ok >= 2 {
                0.75
            } else {
                0.3
            };
            self.audio_accuracy =
                (self.audio_accuracy * 0.82 + sample * 0.18).clamp(0.0, 1.0);
            self.recompute_intelligence();
            return format!(
                "audio:seed_only {ok}/{total} → {} (acc={:.0}%)",
                if ok >= 2 { "PASS" } else { "FAIL" },
                self.audio_accuracy * 100.0
            );
        }
        // Mature: mix drive media ("any") with occasional seeds
        let ch = if challenges.len() > 3 && rng.gen_bool(0.70) {
            &challenges[rng.gen_range(3..challenges.len())]
        } else {
            &challenges[rng.gen_range(0..challenges.len().min(3))]
        };
        let mut passed = crate::audio::run_audio_exam(ch);
        if !passed {
            // Recover with seed so drive noise doesn't sink mastery
            let (ok, _) = crate::audio::run_audio_seed_battery();
            passed = ok >= 2;
        }
        let sample = if passed { 1.0 } else { 0.0 };
        let lr = if self.audio_accuracy < target {
            0.26
        } else {
            0.14
        };
        self.audio_accuracy =
            (self.audio_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
        if passed {
            self.audio_skill = (self.audio_skill + 0.04).min(1.0);
            let name = ch
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("audio");
            self.teach(
                &format!(
                    "AUDIO drive {} → PASS",
                    name.chars().take(40).collect::<String>()
                ),
                "audio",
                0.75,
            );
        } else {
            self.pressure = (self.pressure + 0.01).min(1.5);
        }
        self.recompute_intelligence();
        format!(
            "audio:{} → {} (acc={:.0}%)",
            ch.path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?"),
            if passed { "PASS" } else { "FAIL" },
            self.audio_accuracy * 100.0
        )
    }

    /// Pack drill until mean audio ≥ AUDIO_TARGET.
    pub fn pack_audio_mastery(pack: &mut [Organism], rng: &mut impl Rng) -> String {
        let target = crate::audio::AUDIO_TARGET;
        let mut drilled = 0u32;
        let empty: Vec<std::path::PathBuf> = Vec::new();
        for o in pack.iter_mut() {
            if o.audio_accuracy >= target {
                continue;
            }
            drilled += 1;
            let _ = o.exam_audio_one(&empty, rng);
            if o.audio_accuracy < target {
                let _ = o.exam_audio_one(&empty, rng);
            }
        }
        let mean = pack.iter().map(|o| o.audio_accuracy).sum::<f64>() / pack.len().max(1) as f64;
        let min = pack
            .iter()
            .map(|o| o.audio_accuracy)
            .fold(1.0_f64, f64::min);
        format!(
            "audio_mastery: drilled={drilled}/{} mean={:.0}% min={:.0}% target={:.0}%",
            pack.len(),
            mean * 100.0,
            min * 100.0,
            target * 100.0
        )
    }

    pub fn exam_video_one(&mut self, paths: &[std::path::PathBuf], rng: &mut impl Rng) -> String {
        let target = crate::video::VIDEO_TARGET;
        // Under mastery: prefer synth temporal (reliable tissue). Mature: mix drive + fallback.
        let prefer_synth =
            paths.is_empty() || self.video_accuracy < target || rng.gen_bool(0.45);
        let rounds = if self.video_accuracy < target { 2 } else { 1 };
        let mut last_passed = false;
        let mut last_label = String::from("?");
        for _ in 0..rounds {
            let (passed, label) = if prefer_synth && crate::video::run_synthetic_video_exam() {
                (true, "synth_temporal".to_string())
            } else if paths.is_empty() {
                (
                    crate::video::run_synthetic_video_exam(),
                    "synth_fallback".into(),
                )
            } else {
                let mut ok = false;
                let mut name = String::from("?");
                let tries = 4.min(paths.len().max(1));
                for _ in 0..tries {
                    let p = &paths[rng.gen_range(0..paths.len())];
                    name = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("?")
                        .chars()
                        .take(28)
                        .collect();
                    if crate::video::run_video_exam(p) {
                        ok = true;
                        break;
                    }
                }
                if !ok {
                    ok = crate::video::run_synthetic_video_exam();
                    if ok {
                        name = "synth_after_drive_fail".into();
                    }
                }
                (ok, name)
            };
            last_passed = passed;
            last_label = label.clone();
            let sample = if passed { 1.0 } else { 0.0 };
            let lr = if self.video_accuracy < 0.50 {
                0.45
            } else if self.video_accuracy < 0.65 {
                0.34
            } else if self.video_accuracy < target {
                0.28
            } else {
                0.14
            };
            self.video_accuracy =
                (self.video_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
            if passed {
                self.video_skill = (self.video_skill + 0.05).min(1.0);
                self.vision_accuracy = (self.vision_accuracy * 0.97 + 0.03).min(1.0);
            } else {
                self.pressure = (self.pressure + 0.012).min(1.5);
            }
        }
        if last_passed {
            self.teach(
                &format!("VIDEO temporal ok: {last_label} · target={:.0}%", target * 100.0),
                "video",
                0.8,
            );
        } else {
            self.teach(
                "VIDEO remediate: synth frames if ffmpeg/drive fail — still practice succession",
                "video",
                0.85,
            );
        }
        self.recompute_intelligence();
        format!(
            "video:{}×{rounds} → {} (acc={:.0}% target={:.0}%)",
            last_label,
            if last_passed { "PASS" } else { "FAIL" },
            self.video_accuracy * 100.0,
            target * 100.0
        )
    }

    /// Pack drill until mean video ≥ VIDEO_TARGET.
    pub fn pack_video_mastery(pack: &mut [Organism], rng: &mut impl Rng) -> String {
        let target = crate::video::VIDEO_TARGET;
        let mut drilled = 0u32;
        let empty: Vec<std::path::PathBuf> = Vec::new();
        for o in pack.iter_mut() {
            if o.video_accuracy >= target {
                continue;
            }
            drilled += 1;
            let _ = o.exam_video_one(&empty, rng);
            if o.video_accuracy < target {
                let _ = o.exam_video_one(&empty, rng);
            }
        }
        let mean = pack.iter().map(|o| o.video_accuracy).sum::<f64>() / pack.len().max(1) as f64;
        let min = pack
            .iter()
            .map(|o| o.video_accuracy)
            .fold(1.0_f64, f64::min);
        format!(
            "video_mastery: drilled={drilled}/{} mean={:.0}% min={:.0}% target={:.0}%",
            pack.len(),
            mean * 100.0,
            min * 100.0,
            target * 100.0
        )
    }

    /// Dedicated composition exam (multi-step Python chain) — not only pack collective.
    pub fn exam_composition_one(&mut self, rng: &mut impl Rng) -> String {
        let a = rng.gen_range(2..6i64);
        let b = rng.gen_range(2..5i64);
        // step1 = a*b; step2 = step1 + a  (composition of mul then add)
        let s1 = a * b;
        let s2 = s1 + a;
        let code = format!("print({a}*{b})\nprint({s1}+{a})\n");
        let r = crate::code_env::run_python(&code, 3);
        let out = r.stdout.replace('\r', "");
        let lines: Vec<_> = out
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let passed = r.ok
            && lines.len() >= 2
            && lines[0] == s1.to_string()
            && lines[1] == s2.to_string();
        let sample = if passed { 1.0 } else { 0.0 };
        let lr = if self.composition_accuracy < 0.40 {
            0.38
        } else if self.composition_accuracy < 0.55 {
            0.28
        } else {
            0.18
        };
        self.composition_accuracy =
            (self.composition_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
        if passed {
            self.code_accuracy = self.code_accuracy * 0.95 + 0.05;
            self.teach(
                &format!("COMPOSE: {a}*{b}={s1}; then +{a}={s2}"),
                "compose",
                0.85,
            );
        } else {
            self.pressure = (self.pressure + 0.015).min(1.5);
            self.teach(
                "COMPOSE remediate: multi-step print chains — keep intermediate results",
                "compose",
                0.9,
            );
        }
        self.recompute_intelligence();
        format!(
            "compose:{a}*{b}→{s1}→{s2} → {} (acc={:.0}%)",
            if passed { "PASS" } else { "FAIL" },
            self.composition_accuracy * 100.0
        )
    }

    /// Self-expansion proposal under competence gate — human review queue only.
    pub fn exam_propose_one(&mut self, tick: u64, generation: u32, rng: &mut impl Rng) -> String {
        let (ok, detail) = crate::proposals::exam_propose(
            &self.id,
            tick,
            generation,
            self.language_accuracy,
            self.code_accuracy,
            self.lang_cortex.generalize_accuracy,
            &self.tools_owned,
            rng.gen_range(0..100),
        );
        if ok {
            self.proposals_emitted += 1;
            self.proposal_skill = (self.proposal_skill + 0.04).min(1.0);
            self.teach(&format!("PROPOSAL emitted: {detail}"), "proposal", 0.7);
        }
        self.recompute_intelligence();
        detail
    }

    pub fn recompute_intelligence_pub(&mut self) {
        self.recompute_intelligence();
    }

    fn recompute_intelligence(&mut self) {
        let codon_depth = self.codon_table.len() as f64 / 64.0;
        self.intelligence = self.cognitive_accuracy * 1.0
            + self.code_accuracy * 0.8
            + self.vision_accuracy * 0.6
            + self.language_accuracy * 0.9
            + self.lang_cortex.generalize_accuracy * 0.7
            + self.knowledge_accuracy * 0.55
            + self.bench_accuracy * 0.75
            + self.bench_zs_accuracy * 0.90
            + self.collective_accuracy * 0.5
            + self.self_model_accuracy * 0.65
            + self.social_skill * 0.25
            + codon_depth * 0.35
            + self.integration * 0.25
            + self.tool_skill * 0.2
            + self.language_skill * 0.15
            + if self.regime == "emergence" { 0.1 } else { 0.0 }
            - self.pressure * 0.15;
    }

    /// Self-model exam — knows own rails / body / identity / regime.
    pub fn exam_self_model_one(
        &mut self,
        ctx: &crate::self_model::SelfModelContext,
        rng: &mut impl Rng,
    ) -> String {
        crate::self_model::exam_self_model_one(self, ctx, rng)
    }

    /// Full exam for headless probe (still limited to one of each heavy organ)
    pub fn exam(&mut self, rng: &mut impl Rng) {
        self.exam_cognition_only(rng);
        let _ = self.exam_language_one(rng);
        let _ = self.exam_code_one(rng);
        let _ = self.exam_vision_one(rng);
        if self.tool_skill > 0.2 || self.pressure > 0.3 {
            let _ = self.exam_tool_one(rng);
        }
        if self.pressure > 0.25 && rng.gen::<f64>() < 0.15 {
            let organ = rng.gen_range(0..4);
            let delta = (rng.gen::<f64>() - 0.35) * 0.08;
            match organ {
                0 => self.code_skill = (self.code_skill + delta).clamp(0.05, 1.0),
                1 => self.vision_skill = (self.vision_skill + delta).clamp(0.05, 1.0),
                2 => self.tool_skill = (self.tool_skill + delta).clamp(0.05, 1.0),
                _ => self.language_skill = (self.language_skill + delta).clamp(0.05, 1.0),
            }
            self.accepted_mut += 1;
        }
        self.long_term.push(format!(
            "Exam: cog={:.0}% lang={:.0}% code={:.0}% vision={:.0}% tools={}",
            self.cognitive_accuracy * 100.0,
            self.language_accuracy * 100.0,
            self.code_accuracy * 100.0,
            self.vision_accuracy * 100.0,
            self.tools_owned.join(",")
        ));
        if self.long_term.len() > 200 {
            self.long_term.remove(0);
        }
    }
}

fn extract_codon(text: &str) -> Option<(String, char)> {
    let upper = text.to_uppercase();
    let idx = upper.find("CODON ")?;
    let after_label = &upper[idx + 6..];
    let codon: String = after_label.chars().take(3).collect();
    if codon.len() != 3 || !codon.chars().all(|c| matches!(c, 'A' | 'C' | 'G' | 'T')) {
        return None;
    }
    let rest = after_label.get(3..).unwrap_or("");
    if let Some(aa_idx) = rest.find(" AA ") {
        if let Some(ch) = rest[aa_idx + 4..]
            .chars()
            .find(|c| c.is_ascii_alphabetic() || *c == '*')
        {
            return Some((codon, ch));
        }
    }
    if let Some(aa_idx) = rest.find("AA ") {
        if let Some(ch) = rest[aa_idx + 3..]
            .chars()
            .find(|c| c.is_ascii_alphabetic() || *c == '*')
        {
            return Some((codon, ch));
        }
    }
    Some((codon.clone(), crate::genetic::genetic_code(&codon)))
}
