//! Autonomous multi-organism habitat — pack signals, homeostasis, collective exams

use crate::genetic::{default_organism_genome, mutate_genome};
use crate::homeostasis::{self, PerceivedTime};
use crate::organism::Organism;
use crate::pack::{self, SharedLibrary};
use crate::scalar::compute_for_domain;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Clone, Debug)]
pub struct LineageEntry {
    pub gen: u32,
    pub champion: String,
    pub s: f64,
    pub acc: f64,
    pub fit: f64,
    pub codons: usize,
}

#[derive(Clone, Debug)]
pub struct HabitatEvent {
    pub tick: u64,
    pub msg: String,
}

pub struct Habitat {
    pub tick: u64,
    pub generation: u32,
    pub pack: Vec<Organism>,
    pub hero_idx: usize,
    pub bio_target: f64,
    pub events: Vec<HabitatEvent>,
    pub lineage: Vec<LineageEntry>,
    pub last_answer: String,
    pub last_query_kind: String,
    pub paused: bool,
    pub fed: u64,
    pub exams: u64,
    pub births: u64,
    pub rng: StdRng,
    pub pop_size: usize,
    pub elite: usize,
    pub feed_every: u64,
    pub exam_every: u64,
    pub gen_every: u64,
    pub curriculum: Vec<(String, String)>,
    pub curriculum_i: usize,
    pub heavy_cursor: usize,
    pub current_phase: String,
    pub current_detail: String,
    /// Shared verified pack library (information farming)
    pub shared_library: SharedLibrary,
    /// Last perceived-time snapshot
    pub perceived: PerceivedTime,
    pub last_collective_detail: String,
    /// Factual + FSOT knowledge corpus (exam-backed)
    pub knowledge_facts: Vec<crate::knowledge::FactCard>,
    pub knowledge_i: usize,
    /// External LLM-benchmark bank (MMLU/GSM8K/…) → organ capability surface
    pub bench: crate::bench::BenchBank,
    /// Drive media indexes (audio/video organs)
    pub audio_index: Vec<std::path::PathBuf>,
    pub video_index: Vec<std::path::PathBuf>,
    /// Trinary body chassis link (Mind ABI file bridge)
    pub body: crate::body_bridge::BodyLink,
    /// Goldfish-scale hardware profile
    pub scale: crate::capability::HostScale,
    /// Constraint membrane (rate limits, hard rails)
    pub constraints: crate::constraints::ConstraintState,
    /// Last behavior monitor alert level
    pub monitor_level: String,
}

impl Habitat {
    pub fn new(seed: u64) -> Self {
        let bio = compute_for_domain("biological");
        let bio_target = bio.s;
        let mut rng = StdRng::seed_from_u64(seed);
        let base = default_organism_genome();
        // Goldfish rule: probe RAM/VRAM/cores (Omen 35L + 5070 → expanded/max)
        let scale = crate::capability::probe();
        let pop_size = scale.pop_size;
        let mut pack = Vec::new();
        for i in 0..pop_size {
            let g = if i == 0 {
                base.clone()
            } else {
                mutate_genome(&base, 0.1 + i as f64 * 0.02, &mut rng)
            };
            let mut o = Organism::new(format!("org-g0-{i}"), g, bio_target);
            seed_knowledge(&mut o, i);
            pack.push(o);
        }

        let mut curriculum = build_curriculum();

        let mut h = Self {
            tick: 0,
            generation: 0,
            pack,
            hero_idx: 0,
            bio_target,
            events: Vec::new(),
            lineage: Vec::new(),
            last_answer: "Welcome. Speak naturally. I am a pack-aware FSOT organism.\nTry: How are you? | pack library | fsot domain quantum | What does ATG code for? | What is 2+2?\nBody/memory: ask 'body status' or 'scale'.".into(),
            last_query_kind: "boot".into(),
            paused: false,
            fed: 0,
            exams: 0,
            births: 0,
            rng,
            pop_size,
            elite: scale.elite,
            feed_every: scale.feed_every,
            exam_every: scale.exam_every,
            gen_every: 80,
            curriculum,
            curriculum_i: 0,
            heavy_cursor: 0,
            current_phase: "boot".into(),
            current_detail: "initializing".into(),
            shared_library: SharedLibrary::default(),
            perceived: homeostasis::perceived_time(0, 0),
            last_collective_detail: String::new(),
            knowledge_facts: crate::knowledge::load_all_facts_capped(scale.knowledge_cap),
            knowledge_i: 0,
            bench: crate::bench::BenchBank::default(),
            audio_index: Vec::new(),
            video_index: Vec::new(),
            body: crate::body_bridge::BodyLink::default(),
            scale: scale.clone(),
            constraints: crate::constraints::ConstraintState::new(),
            monitor_level: "green".into(),
        };
        let _ = crate::code_env::ensure_workspace();
        let _ = crate::vision::ensure_vision_lib();
        let _ = crate::audio::ensure_audio_lib();
        let _ = crate::video::ensure_video_scratch();
        let _ = crate::proposals::ensure_dirs();
        // LLM-capability bridge: load external benchmarks from training data drive
        let bench_cap = (scale.knowledge_cap / 2).clamp(128, 2_500);
        h.bench = crate::bench::load_bench_bank(bench_cap);
        if !h.bench.items.is_empty() {
            let cards = crate::bench::items_as_fact_cards(&h.bench.items, bench_cap.min(1_200));
            // Fuse into knowledge curriculum (same surface, exam-backed)
            for c in cards {
                if !h.knowledge_facts.iter().any(|f| f.id == c.id) {
                    h.knowledge_facts.push(c);
                }
            }
            h.push(h.bench.summary_line());
        } else {
            h.push(
                "bench: no external suites found — set FSOT_TRAINING_DATA or mount $FSOT_TRAINING_DATA"
                    .to_string(),
            );
        }
        // Index drive media — caps scale with tank size (goldfish rule)
        h.audio_index = crate::audio::index_audio_paths(h.scale.audio_index_cap);
        h.video_index = crate::video::index_video_paths(h.scale.video_index_cap);
        for seed in crate::video::drive_audio_curriculum_seeds(&h.audio_index, 12) {
            h.curriculum.push(seed);
        }
        for seed in crate::video::drive_video_curriculum_seeds(&h.video_index, 12) {
            h.curriculum.push(seed);
        }
        // Birth knowledge injection
        let facts = h.knowledge_facts.clone();
        for (i, o) in h.pack.iter_mut().enumerate() {
            crate::knowledge::teach_batch(o, &facts, i, facts.len().min(64).max(1));
        }
        h.push(format!(
            "knowledge={} cards · audio_idx={} · video_idx={} · ffmpeg={} · node={}",
            facts.len(),
            h.audio_index.len(),
            h.video_index.len(),
            crate::audio::ffmpeg_available(),
            crate::code_env::node_available()
        ));
        // DRIVE_CURRICULUM — real media roots as structured teach seeds
        for (root, tag) in [
            (r"../data_external/local/movies", "video_corpus"),
            (r"../data_external/local/showes", "series_corpus"),
            (r"../data_external/local/Debut", "debut_media"),
            (r"../data_external/ai_datasets", "ai_datasets"),
        ] {
            if std::path::Path::new(root).exists() {
                h.curriculum.push((
                    format!(
                        "DRIVE_CURRICULUM [{tag}]: root={root} — use as future vision/audio pressure; do not invent paths"
                    ),
                    "drive".into(),
                ));
                // Shallow sample of a few entry names for grounded awareness
                if let Ok(rd) = std::fs::read_dir(root) {
                    for (i, e) in rd.flatten().take(4).enumerate() {
                        let name = e.file_name().to_string_lossy().chars().take(60).collect::<String>();
                        h.curriculum.push((
                            format!("DRIVE_SAMPLE [{tag} #{i}]: {name}"),
                            "drive".into(),
                        ));
                    }
                }
            }
        }
        // Seed library with framework priors
        h.shared_library.publish(
            "FSOT S = k*(T1+T2+T3); use fsot_domain tool for verified domain S",
            "fsot",
            "habitat",
            1.5,
            true,
            0,
        );
        h.shared_library.publish(
            "Pack signals couple organisms; collective exams farm composed knowledge",
            "pack",
            "habitat",
            1.2,
            true,
            0,
        );

        h.push("Habitat online — pack signals · homeostasis · collective · FSOT tool");
        h.push(h.scale.summary());
        h.push(format!(
            "bio S={:.4} · pack={} · knowledge={} · curriculum={} · python={}",
            bio_target,
            pop_size,
            h.knowledge_facts.len(),
            h.curriculum.len(),
            crate::code_env::python_available()
        ));
        // Mind STM couple: resume genome / gen / identity / body from disk
        let _ = crate::resume::resume_mind_stm(&mut h);
        // Probe body bridge once at boot (nerves + felt interoception)
        crate::body_bridge::couple_body(&mut h);
        h.push(if h.body.online {
            format!(
                "body chassis ONLINE · genome={} · couple={:.3} · via={}",
                h.body.status.genome_fp, h.body.couple_pressure, h.body.transport
            )
        } else {
            "body chassis offline (start mind_bridge --tcp or QEMU serial nerve)".into()
        });
        // Lean-grounded scalar sanity (FSOT S=k·(T1+T2+T3) family)
        let lean = crate::lean_ground::check_runtime_invariants();
        h.push(lean.summary_line());
        h.push(format!("constraint membrane online · {}", h.constraints.summary()));
        h.rank_hero();
        h.set_activity("idle", "ready — continuous self · constrained growth");
        h
    }

    fn set_activity(&mut self, phase: impl Into<String>, detail: impl Into<String>) {
        let phase = phase.into();
        let detail = detail.into();
        self.current_phase = phase.clone();
        self.current_detail = detail.clone();
        let act = crate::activity::Activity::new(self.tick, phase, detail);
        crate::activity::write_activity(&act);
    }

    pub(crate) fn push(&mut self, msg: impl Into<String>) {
        self.events.insert(
            0,
            HabitatEvent {
                tick: self.tick,
                msg: msg.into(),
            },
        );
        if self.events.len() > 40 {
            self.events.pop();
        }
    }

    pub fn hero(&self) -> &Organism {
        &self.pack[self.hero_idx]
    }

    pub fn hero_mut(&mut self) -> &mut Organism {
        &mut self.pack[self.hero_idx]
    }

    fn rank_hero(&mut self) {
        let mut best = 0usize;
        let mut best_f = f64::MIN;
        for (i, o) in self.pack.iter().enumerate() {
            // Champion = breeding value (accuracy + efficiency), not raw fit alone
            let f = o.breeding_value();
            if f > best_f {
                best_f = f;
                best = i;
            }
        }
        self.hero_idx = best;
    }

    pub fn step(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;
        // Metabolic membrane: cap Python spawns this tick (invent storms ≠ more capability)
        crate::code_env::reset_python_budget(20);

        // FSOT perceived-time homeostasis (sin/cos multi-frequency)
        self.perceived = homeostasis::perceived_time(self.tick, self.generation);
        // Allostatic coupling: pack bench/zs deficit raises exam+teach gain (depth comes from pressure)
        {
            let n = self.pack.len().max(1) as f64;
            let mean_zs = self.pack.iter().map(|o| o.bench_zs_accuracy).sum::<f64>() / n;
            let mean_b = self.pack.iter().map(|o| o.bench_accuracy).sum::<f64>() / n;
            let allostatic = ((crate::bench::BENCH_ZS_TARGET - mean_zs).max(0.0) * 0.50
                + (crate::bench::BENCH_TARGET - mean_b).max(0.0) * 0.25)
                .clamp(0.0, 0.55);
            self.perceived.exam_gain =
                (self.perceived.exam_gain + allostatic).clamp(0.5, 1.55);
            self.perceived.teach_gain =
                (self.perceived.teach_gain + allostatic * 0.65).clamp(0.4, 1.45);
            // Mild pack pressure nudge when competence deficit high (stimulates chew depth)
            if allostatic > 0.12 && self.tick % 5 == 0 {
                for o in &mut self.pack {
                    o.pressure = (o.pressure + allostatic * 0.03).min(1.5);
                }
            }
        }
        let pt = self.perceived.clone();

        // Trinary body chassis (Mind ABI file bridge) — pressure couple + MOTOR policy
        crate::body_bridge::couple_body(self);

        // Constraint notes on body phenotype + fitness (soft watches)
        if self.tick % 10 == 0 {
            self.constraints.on_generation(self.generation);
            let fit = self.hero().fitness_score();
            self.constraints.note_fitness(self.tick, fit);
            if self.body.online {
                self.constraints.note_body_express(
                    self.tick,
                    self.body.status.express,
                    self.body.status.latent,
                    self.body.status.repress,
                );
            }
        }
        // Behavior monitor snapshot (rails + alerts — not malice assumption)
        if self.tick % 25 == 0 {
            let snap = crate::behavior_monitor::observe(self, &self.constraints);
            self.monitor_level = snap.alert_level.clone();
            if snap.alert_level != "green" && self.tick % 100 == 0 {
                self.push(format!(
                    "monitor[{}] {}",
                    snap.alert_level,
                    snap.alerts.first().cloned().unwrap_or_else(|| "—".into())
                ));
            }
        }

        // Environmental stress: classic + perceived drive + body couple (scale-weighted)
        let stress = (self.tick as f64 / 27.0).sin() * 0.18
            + (self.tick as f64 / 91.0).sin() * 0.08
            + pt.drive * 0.06
            + self.body.couple_pressure * self.scale.body_couple_gain;

        // Self-model homeostasis: low accuracy → stronger self-knowledge stimuli
        if self.tick % 40 == 0 {
            let detail = crate::self_model::pack_self_model_homeostasis(
                &mut self.pack,
                self.generation,
                self.body.online,
                self.body.status.express,
                self.body.status.latent,
                self.body.status.repress,
                &self.monitor_level,
                pt.teach_gain,
            );
            if detail.contains("boosted=") && !detail.contains("boosted=0/") {
                self.push(detail.clone());
                self.set_activity("self_model_homeo", &detail);
            }
        }
        // Pack-wide organ accuracy homeostasis (all modalities, not only self_model)
        if self.tick % 35 == 0 {
            let audit = crate::accuracy_homeo::pack_accuracy_homeostasis(
                &mut self.pack,
                self.tick,
                self.generation,
                pt.teach_gain,
            );
            if audit.pack_weak_organs > 0 {
                let line = crate::accuracy_homeo::summary_line(&audit);
                self.push(line.clone());
                if self.tick % 70 == 0 {
                    self.set_activity("accuracy_homeo", &line);
                }
            }
        }
        // Knowledge mastery drill — stay on it until pack mean ≥ 85%
        if self.tick % 28 == 0 && !self.knowledge_facts.is_empty() {
            let mean_k = self
                .pack
                .iter()
                .map(|o| o.knowledge_accuracy)
                .sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_k < crate::knowledge::KNOWLEDGE_TARGET {
                let facts = self.knowledge_facts.clone();
                let line = crate::knowledge::pack_knowledge_mastery(
                    &mut self.pack,
                    &facts,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("knowledge_mastery", &line);
            }
        }
        // JS mastery drill — stay on it until pack mean ≥ 85%
        if self.tick % 24 == 0 && crate::code_env::node_available() {
            let mean_j = self.pack.iter().map(|o| o.js_accuracy).sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_j < crate::code_env::JS_TARGET {
                let line = crate::organism::Organism::pack_js_mastery(
                    &mut self.pack,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("js_mastery", &line);
            }
        }
        // Gen_acc mastery — novel phrasing transfer ≥ 75%
        if self.tick % 26 == 0 {
            let mean_g = self
                .pack
                .iter()
                .map(|o| o.lang_cortex.generalize_accuracy)
                .sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_g < crate::language::GEN_ACC_TARGET {
                let line = crate::language::pack_gen_mastery(&mut self.pack, &mut self.rng);
                self.push(line.clone());
                self.set_activity("gen_mastery", &line);
            }
        }
        // Audio / video media mastery — seed/synth tissue ≥ 75%
        if self.tick % 22 == 0 {
            let mean_a = self.pack.iter().map(|o| o.audio_accuracy).sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_a < crate::audio::AUDIO_TARGET {
                let line =
                    crate::organism::Organism::pack_audio_mastery(&mut self.pack, &mut self.rng);
                self.push(line.clone());
                self.set_activity("audio_mastery", &line);
            }
            let mean_v = self.pack.iter().map(|o| o.video_accuracy).sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_v < crate::video::VIDEO_TARGET {
                let line =
                    crate::organism::Organism::pack_video_mastery(&mut self.pack, &mut self.rng);
                self.push(line.clone());
                self.set_activity("video_mastery", &line);
            }
        }
        // LLM-capability bridge mastery — external benches via organs
        // Period fixed: density of work *per call* was the hang (Python invent storms),
        // not tick count. Soft-spot + budget membrane handle climb without thrashing wall-time.
        let mean_zs_now = self.pack.iter().map(|o| o.bench_zs_accuracy).sum::<f64>()
            / self.pack.len().max(1) as f64;
        // Match original organ cadence: capability mastery is periodic, not every breath
        let zs_period = 40u64;
        if self.tick % zs_period == 0 && !self.bench.items.is_empty() {
            let mean_b = self.pack.iter().map(|o| o.bench_accuracy).sum::<f64>()
                / self.pack.len().max(1) as f64;
            if mean_b < crate::bench::BENCH_TARGET {
                let line = crate::bench::pack_bench_mastery(
                    &mut self.pack,
                    &mut self.bench,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("bench_mastery", &line);
            }
            // Error-queue remediation: pressure where we already failed
            if !self.bench.error_queue.is_empty() {
                let line = crate::bench::pack_error_remediate(
                    &mut self.pack,
                    &mut self.bench,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("err_remediate", &line);
            }
            // Zero-shot climb: soft-suite drills + pack schema broadcast
            let mean_zs = mean_zs_now;
            if mean_zs < crate::bench::BENCH_ZS_TARGET
                || self.bench.zs_mean() < crate::bench::BENCH_ZS_TARGET
            {
                let line = crate::bench::pack_zs_mastery(
                    &mut self.pack,
                    &mut self.bench,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("zs_mastery", &line);
            } else if mean_zs < crate::bench::BENCH_ZS_STRETCH && self.tick % 40 == 0 {
                // Stretch toward 80% zs
                let line = crate::bench::pack_zs_mastery(
                    &mut self.pack,
                    &mut self.bench,
                    &mut self.rng,
                );
                self.push(line.clone());
                self.set_activity("zs_stretch", &line);
            }
            if self.tick % 40 == 0 {
                let audit = crate::bench::write_capability_audit(
                    self.tick,
                    self.generation,
                    &self.pack,
                    &self.bench,
                );
                self.push(format!(
                    "CAPABILITY_AUDIT bench={:.0}%/{:.0}% zs={:.0}%/{:.0}% bank={}",
                    audit.pack_bench_mean * 100.0,
                    audit.pack_bench_min * 100.0,
                    audit.pack_zs_mean * 100.0,
                    audit.pack_zs_min * 100.0,
                    audit.bank_size
                ));
            }
        }

        // feed (modulated by teach_gain) + knowledge curriculum
        if self.tick % self.feed_every == 0 {
            self.set_activity("feed", "curriculum + knowledge + pack library");
            let (text, kind) = self.curriculum[self.curriculum_i % self.curriculum.len()].clone();
            let strength = 0.75 * pt.teach_gain;
            for o in &mut self.pack {
                // Extra gain when self-model is under-stimulated
                let sm_boost = if o.self_model_accuracy < 0.55 {
                    1.0 + (0.55 - o.self_model_accuracy)
                } else {
                    1.0
                };
                let k_boost = if kind == "self_model" { sm_boost * 1.25 } else { sm_boost };
                o.teach(&text, &kind, strength * k_boost);
                self.fed += 1;
            }
            // Factual knowledge bites — more volume when pack knowledge is soft
            if !self.knowledge_facts.is_empty() {
                let start = self.knowledge_i;
                let mean_k = self
                    .pack
                    .iter()
                    .map(|o| o.knowledge_accuracy)
                    .sum::<f64>()
                    / self.pack.len().max(1) as f64;
                let n_cards = if mean_k < 0.70 {
                    10
                } else if mean_k < crate::knowledge::KNOWLEDGE_TARGET {
                    7
                } else {
                    3
                };
                for o in &mut self.pack {
                    let n = if o.knowledge_accuracy < crate::knowledge::KNOWLEDGE_TARGET {
                        n_cards + 4
                    } else {
                        n_cards
                    };
                    // Prioritize innate seeds then corpus slice
                    let seeds = crate::knowledge::seed_facts();
                    crate::knowledge::teach_batch(o, &seeds, start % seeds.len().max(1), 4);
                    crate::knowledge::teach_batch(o, &self.knowledge_facts, start, n);
                }
                self.knowledge_i = self.knowledge_i.wrapping_add(n_cards);
            }
            if !self.shared_library.entries.is_empty() && self.tick % 16 == 0 {
                let brief = self.shared_library.summary_text(1);
                for o in &mut self.pack {
                    o.teach(&format!("PACK_LIB: {brief}"), "pack_lib", 0.7 * pt.teach_gain);
                }
            }
            self.curriculum_i += 1;
        }

        self.set_activity("live", format!(
            "scalar+homeo sin={:.2} cos={:.2}",
            pt.phase_sin, pt.phase_cos
        ));
        for o in &mut self.pack {
            if self.tick % 20 == 0 {
                o.apply_genome_guidance();
            }
            o.tick_once(stress, &mut self.rng);
            // Homeostasis loop on each organism
            homeostasis::apply_to_organism(
                &mut o.coherence,
                &mut o.pressure,
                &mut o.reward,
                &mut o.vitality,
                &mut o.integration,
                &pt,
                o.social_alignment,
            );
            o.perceived_drive = pt.drive;
            o.homeo_coherence_pull = pt.phase_cos;
        }

        // Pack communication (chemical analogue) every tick after live compute
        let signals: Vec<_> = self.pack.iter().map(pack::emit_signal).collect();
        let _align = pack::couple_pack(
            &mut self.pack,
            &signals,
            &pt,
            &mut self.shared_library,
            self.tick,
        );
        let lib_brief = self.shared_library.summary_text(5);
        for o in &mut self.pack {
            o.pack_library_brief = lib_brief.clone();
        }

        // Light cognition
        if self.tick % self.exam_every == 0 {
            self.set_activity("exam_cognition", "fast codon/regime checks");
            for o in &mut self.pack {
                o.exam_cognition_only(&mut self.rng);
            }
            self.exams += self.pack.len() as u64;
        }

        // Heavy rotation (one organ/action per 3 ticks — includes audio/video drive pressure)
        // 0 code | 1 vision | 2 tool/js | 3 language | 4 collective | 5 knowledge
        // 6 audio | 7 video | 8 propose | 9 self_model
        // Bias ~40% of slots toward each organism's weakest organ (accuracy homeo).
        // Every 4th heavy slot: external LLM-benchmark capability exam (bridge organs↔SOTA surface).
        if self.tick % 3 == 0 && !self.pack.is_empty() {
            let idx = self.heavy_cursor % self.pack.len();
            let pack_bench_mean = self.pack.iter().map(|o| o.bench_accuracy).sum::<f64>()
                / self.pack.len().max(1) as f64;
            // Bench is one organ among many — sparse slots only (restore multi-organ life)
            let force_bench = !self.bench.items.is_empty()
                && self.heavy_cursor % 8 == 0
                && (pack_bench_mean < crate::bench::BENCH_TARGET || self.heavy_cursor % 16 == 0);
            let phase = if force_bench {
                10 // dedicated bench phase
            } else if self.heavy_cursor % 5 != 0 {
                if let Some(p) = crate::accuracy_homeo::preferred_exam_phase(&self.pack[idx]) {
                    p
                } else {
                    self.heavy_cursor % 10
                }
            } else {
                self.heavy_cursor % 10
            };
            let id = self.pack[idx].id.clone();
            match phase {
                4 => {
                    self.set_activity("exam_collective", "pack multihop / consistency");
                    let detail = pack::exam_collective(
                        &mut self.pack,
                        &mut self.shared_library,
                        &mut self.rng,
                        self.tick,
                    );
                    self.last_collective_detail = detail.clone();
                    self.push(format!("Collective {detail}"));
                    let act = crate::activity::Activity::new(self.tick, "exam_collective", &detail);
                    crate::activity::write_activity(&act);
                }
                5 => {
                    self.set_activity("exam_knowledge", format!("factual curriculum · {id}"));
                    let facts = self.knowledge_facts.clone();
                    // Always pressure the weakest knowledge org first
                    let exam_idx = self
                        .pack
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.knowledge_accuracy
                                .partial_cmp(&b.knowledge_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(idx);
                    let exam_id = self.pack[exam_idx].id.clone();
                    let mut parts = Vec::new();
                    // Triple exam while under 85% mastery bar
                    let rounds = if self.pack[exam_idx].knowledge_accuracy
                        < crate::knowledge::KNOWLEDGE_TARGET
                    {
                        3
                    } else {
                        1
                    };
                    for _ in 0..rounds {
                        let d = crate::knowledge::exam_knowledge_one(
                            &mut self.pack[exam_idx],
                            &facts,
                            &mut self.rng,
                        );
                        parts.push(d);
                    }
                    // Also lift a second-weakest if pack mean low
                    let mean_k = self
                        .pack
                        .iter()
                        .map(|o| o.knowledge_accuracy)
                        .sum::<f64>()
                        / self.pack.len().max(1) as f64;
                    if mean_k < crate::knowledge::KNOWLEDGE_TARGET {
                        let mut idxs: Vec<usize> = (0..self.pack.len()).collect();
                        idxs.sort_by(|&i, &j| {
                            self.pack[i]
                                .knowledge_accuracy
                                .partial_cmp(&self.pack[j].knowledge_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        if let Some(&i2) = idxs.get(1) {
                            if i2 != exam_idx
                                && self.pack[i2].knowledge_accuracy
                                    < crate::knowledge::KNOWLEDGE_TARGET
                            {
                                let d = crate::knowledge::exam_knowledge_one(
                                    &mut self.pack[i2],
                                    &facts,
                                    &mut self.rng,
                                );
                                parts.push(format!("+[{}] {d}", self.pack[i2].id));
                            }
                        }
                    }
                    let detail2 = parts.join(" | ");
                    self.push(format!("Heavy[{exam_id}] {detail2}"));
                    let act =
                        crate::activity::Activity::new(self.tick, "exam_knowledge", &detail2)
                            .with_org(exam_id);
                    crate::activity::write_activity(&act);
                }
                6 => {
                    // Prefer weakest audio org until pack clears mastery
                    let aidx = self
                        .pack
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.audio_accuracy
                                .partial_cmp(&b.audio_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(idx);
                    let aid = self.pack[aidx].id.clone();
                    self.set_activity("exam_audio", format!("drive audio energy · {aid}"));
                    let paths = self.audio_index.clone();
                    let d1 = self.pack[aidx].exam_audio_one(&paths, &mut self.rng);
                    let detail = if self.pack[aidx].audio_accuracy < crate::audio::AUDIO_TARGET {
                        let d2 = self.pack[aidx].exam_audio_one(&paths, &mut self.rng);
                        format!("{d1} | {d2}")
                    } else {
                        d1
                    };
                    self.push(format!("Heavy[{aid}] {detail}"));
                    let act = crate::activity::Activity::new(self.tick, "exam_audio", &detail)
                        .with_org(aid);
                    crate::activity::write_activity(&act);
                }
                7 => {
                    let vidx = self
                        .pack
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.video_accuracy
                                .partial_cmp(&b.video_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(idx);
                    let vid = self.pack[vidx].id.clone();
                    self.set_activity("exam_video", format!("drive video frames · {vid}"));
                    let paths = self.video_index.clone();
                    let d1 = self.pack[vidx].exam_video_one(&paths, &mut self.rng);
                    let detail = if self.pack[vidx].video_accuracy < crate::video::VIDEO_TARGET {
                        let d2 = self.pack[vidx].exam_video_one(&paths, &mut self.rng);
                        format!("{d1} | {d2}")
                    } else {
                        d1
                    };
                    self.push(format!("Heavy[{vid}] {detail}"));
                    let act = crate::activity::Activity::new(self.tick, "exam_video", &detail)
                        .with_org(vid);
                    crate::activity::write_activity(&act);
                }
                8 => {
                    // Offensive: composition first when lagging; propose is rarer
                    let need_compose = self.pack[idx].composition_accuracy < 0.52;
                    if need_compose || self.tick % 5 != 0 {
                        self.set_activity("exam_compose", format!("multi-step chain · {id}"));
                        let detail = self.pack[idx].exam_composition_one(&mut self.rng);
                        self.push(format!("Heavy[{id}] {detail}"));
                        let act =
                            crate::activity::Activity::new(self.tick, "exam_compose", &detail)
                                .with_org(id);
                        crate::activity::write_activity(&act);
                    } else if self.tick % 36 == 0 {
                        self.set_activity("exam_propose", format!("self-expansion · {id}"));
                        let gen = self.generation;
                        let tick = self.tick;
                        let pending = crate::proposals::count_pending();
                        if !self
                            .constraints
                            .gate_proposal(tick, gen, pending, true)
                            .allowed()
                        {
                            self.push(format!(
                                "Propose[{id}] throttled ({})",
                                self.constraints.last_gate_code
                            ));
                        } else {
                            let detail =
                                self.pack[idx].exam_propose_one(tick, gen, &mut self.rng);
                            self.push(format!("Propose[{id}] {detail}"));
                            let act =
                                crate::activity::Activity::new(self.tick, "exam_propose", &detail)
                                    .with_org(id);
                            crate::activity::write_activity(&act);
                        }
                    } else {
                        self.set_activity("exam_compose", format!("multi-step chain · {id}"));
                        let detail = self.pack[idx].exam_composition_one(&mut self.rng);
                        self.push(format!("Heavy[{id}] {detail}"));
                        let act =
                            crate::activity::Activity::new(self.tick, "exam_compose", &detail)
                                .with_org(id);
                        crate::activity::write_activity(&act);
                    }
                }
                9 => {
                    // Prefer weakest self_model until pack clears 70%
                    let sidx = self
                        .pack
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.self_model_accuracy
                                .partial_cmp(&b.self_model_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(idx);
                    let sid = self.pack[sidx].id.clone();
                    self.set_activity("exam_self_model", format!("know thyself · {sid}"));
                    let ctx = crate::self_model::SelfModelContext::from_org_habitat(
                        &self.pack[sidx],
                        self.generation,
                        self.body.online,
                        self.body.status.express,
                        self.body.status.latent,
                        self.body.status.repress,
                        &self.monitor_level,
                        &self.constraints,
                        self.perceived.teach_gain,
                        self.perceived.exam_gain,
                    );
                    let detail = self.pack[sidx].exam_self_model_one(&ctx, &mut self.rng);
                    self.push(format!("Heavy[{sid}] {detail}"));
                    let act =
                        crate::activity::Activity::new(self.tick, "exam_self_model", &detail)
                            .with_org(sid);
                    crate::activity::write_activity(&act);
                }
                10 => {
                    // LLM-capability surface: weakest bench org first
                    let bidx = self
                        .pack
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.bench_accuracy
                                .partial_cmp(&b.bench_accuracy)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(idx);
                    let bid = self.pack[bidx].id.clone();
                    self.set_activity(
                        "exam_bench",
                        format!("LLM-surface suite · {bid}"),
                    );
                    let d1 = crate::bench::exam_bench_one(
                        &mut self.pack[bidx],
                        &mut self.bench,
                        &mut self.rng,
                    );
                    let detail = if self.pack[bidx].bench_accuracy < crate::bench::BENCH_TARGET {
                        let d2 = crate::bench::exam_bench_one(
                            &mut self.pack[bidx],
                            &mut self.bench,
                            &mut self.rng,
                        );
                        format!("{d1} | {d2}")
                    } else {
                        d1
                    };
                    self.push(format!("Heavy[{bid}] {detail}"));
                    let act = crate::activity::Activity::new(self.tick, "exam_bench", &detail)
                        .with_org(bid);
                    crate::activity::write_activity(&act);
                    self.exams += 1;
                }
                _ => {
                    let phase_name = match phase {
                        0 => "exam_code",
                        1 => "exam_vision",
                        2 => "exam_tool",
                        _ => "exam_language",
                    };
                    let pre = match phase {
                        0 => format!("python challenge · {id}"),
                        1 => format!("image exam · {id}"),
                        2 => format!("forge/js/l2/invent · {id}"),
                        _ => format!("plastic language · {id}"),
                    };
                    self.set_activity(phase_name, pre);
                    let detail = match phase {
                        0 => {
                            // Prefer multi-file L2 when that organ is the lagging target
                            let o = &self.pack[idx];
                            if o.code_l2_accuracy + 0.12 < o.code_accuracy
                                || o.code_l2_accuracy < 0.30
                            {
                                self.pack[idx].exam_code_l2_one(&mut self.rng)
                            } else {
                                self.pack[idx].exam_code_one(&mut self.rng)
                            }
                        }
                        1 => self.pack[idx].exam_vision_one(&mut self.rng),
                        2 => {
                            // Prefer weakest JS org until pack clears 85% mastery
                            let mean_j = self.pack.iter().map(|o| o.js_accuracy).sum::<f64>()
                                / self.pack.len().max(1) as f64;
                            if mean_j < crate::code_env::JS_TARGET
                                && crate::code_env::node_available()
                            {
                                let jidx = self
                                    .pack
                                    .iter()
                                    .enumerate()
                                    .min_by(|(_, a), (_, b)| {
                                        a.js_accuracy
                                            .partial_cmp(&b.js_accuracy)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    })
                                    .map(|(i, _)| i)
                                    .unwrap_or(idx);
                                // Double battery on weakest
                                let d1 = self.pack[jidx].exam_js_one(&mut self.rng);
                                if self.pack[jidx].js_accuracy < crate::code_env::JS_TARGET {
                                    let d2 = self.pack[jidx].exam_js_one(&mut self.rng);
                                    format!("{} | {}", d1, d2)
                                } else {
                                    d1
                                }
                            } else if self.pack[idx].code_l2_accuracy < 0.30 {
                                self.pack[idx].exam_code_l2_one(&mut self.rng)
                            } else if self.heavy_cursor % 3 == 0 {
                                exam_fsot_tool(&mut self.pack[idx], &mut self.rng)
                            } else {
                                self.pack[idx].exam_tool_one(&mut self.rng)
                            }
                        }
                        _ => {
                            // Prefer weakest gen_acc org until pack clears 75% mastery
                            let mean_g = self
                                .pack
                                .iter()
                                .map(|o| o.lang_cortex.generalize_accuracy)
                                .sum::<f64>()
                                / self.pack.len().max(1) as f64;
                            if mean_g < crate::language::GEN_ACC_TARGET {
                                let gidx = self
                                    .pack
                                    .iter()
                                    .enumerate()
                                    .min_by(|(_, a), (_, b)| {
                                        a.lang_cortex
                                            .generalize_accuracy
                                            .partial_cmp(&b.lang_cortex.generalize_accuracy)
                                            .unwrap_or(std::cmp::Ordering::Equal)
                                    })
                                    .map(|(i, _)| i)
                                    .unwrap_or(idx);
                                let d1 = self.pack[gidx].exam_language_one(&mut self.rng);
                                if self.pack[gidx].lang_cortex.generalize_accuracy
                                    < crate::language::GEN_ACC_TARGET
                                {
                                    let d2 = self.pack[gidx].exam_language_one(&mut self.rng);
                                    format!("{d1} | {d2}")
                                } else {
                                    d1
                                }
                            } else {
                                self.pack[idx].exam_language_one(&mut self.rng)
                            }
                        }
                    };
                    self.push(format!("Heavy[{id}] {detail}"));
                    let act =
                        crate::activity::Activity::new(self.tick, phase_name, &detail).with_org(id);
                    crate::activity::write_activity(&act);
                }
            }
            self.heavy_cursor += 1;
        }

        if self.tick > 0 && self.tick % self.gen_every == 0 {
            self.set_activity("generation", "selection + birth + library carry");
            self.next_generation();
        }

        self.rank_hero();

        if self.tick % 30 == 0 {
            let h = self.hero();
            let id = h.id.clone();
            let s = h.s;
            let lang = h.language_accuracy;
            let code = h.code_accuracy;
            let coll = h.collective_accuracy;
            let social = h.social_skill;
            let lib_n = self.shared_library.len();
            self.push(format!(
                "t={} {} S={:.3} lang={:.0}% code={:.0}% coll={:.0}% social={:.2} lib={}",
                self.tick,
                id,
                s,
                lang * 100.0,
                code * 100.0,
                coll * 100.0,
                social,
                lib_n
            ));
            self.set_activity(
                "heartbeat",
                format!(
                    "{id} drive={:.2} lib={} coll={:.0}%",
                    pt.drive,
                    lib_n,
                    coll * 100.0
                ),
            );
        }
    }

    fn next_generation(&mut self) {
        // Selective breeding: rank by breeding_value (accuracy × efficiency), not raw fit alone
        // Body chassis constraint (goldfish + allostasis): couple pressure & plasticity
        // modulate mutation rate and pool strictness — biology under hardware load.
        let body_online = self.body.online;
        let body_couple = self.body.couple_pressure.abs();
        let body_plast = if self.body.status.plasticity > 0.0 {
            self.body.status.plasticity
        } else {
            0.55
        };
        let body_expr = self.body.status.mean_expr;
        // High allostatic load (strong couple swing) → tighter pool, milder mutations
        let load = if body_online {
            (body_couple * 1.2 + (1.0 - body_plast) * 0.4).clamp(0.0, 1.2)
        } else {
            0.35 // no body: moderate default
        };
        let mut_scale = (1.15 - load * 0.55).clamp(0.45, 1.2);
        // Prefer stability when body is highly expressed/stable
        let mut_scale = if body_online && body_expr > 0.65 {
            mut_scale * 0.85
        } else {
            mut_scale
        };
        let pool_floor = 0.88 + load * 0.08; // stricter cull under load

        self.pack.sort_by(|a, b| {
            b.breeding_value()
                .partial_cmp(&a.breeding_value())
                .unwrap()
        });
        let champ = &self.pack[0];
        self.lineage.push(LineageEntry {
            gen: self.generation,
            champion: champ.id.clone(),
            s: champ.s,
            acc: champ.mean_accuracy(),
            fit: champ.fitness_score(),
            codons: champ.codon_table.len(),
        });
        self.push(format!(
            "Gen {} · {} breed={:.2} fit={:.2} mean_acc={:.0}% lib={} · body_load={:.2} mut×={:.2}",
            self.generation,
            champ.id,
            champ.breeding_value(),
            champ.fitness_score(),
            champ.mean_accuracy() * 100.0,
            self.shared_library.len(),
            load,
            mut_scale
        ));

        // Breeding pool: only above-median breeding value (cull weak pathways)
        // under body load, floor rises (hardware/body constraint)
        let median = if self.pack.len() >= 2 {
            let mid = self.pack.len() / 2;
            self.pack[mid].breeding_value()
        } else {
            0.0
        };
        let pool: Vec<Organism> = self
            .pack
            .iter()
            .filter(|o| {
                o.breeding_value() >= median * pool_floor
                    || o.breeding_value() >= self.pack[0].breeding_value() * 0.75
            })
            .take(4.max(self.elite))
            .cloned()
            .collect();
        let pool = if pool.len() < 2 {
            self.pack.iter().take(2).cloned().collect::<Vec<_>>()
        } else {
            pool
        };

        let mut next = Vec::new();
        // Elites clone (top by breeding value)
        for (i, e) in pool.iter().take(self.elite).enumerate() {
            let mut o = e.clone();
            o.id = format!("elite-g{}-{i}", self.generation + 1);
            o.apply_genome_guidance();
            next.push(o);
        }

        // Offspring: mate pair attracted by efficiency/accuracy affinity
        // mutation rates scaled by body load (see mut_scale below)
        let mut_scale = mut_scale;
        while next.len() < self.pop_size {
            let (p0, p1) = select_mate_pair(&pool, &mut self.rng);
            let mean_breed = (p0.breeding_value() + p1.breeding_value()) * 0.5;
            // High-efficiency parents: lower mutation (preserve successful structure)
            let mut_rate = if mean_breed > pool[0].breeding_value() * 0.9 {
                0.05
            } else if mean_breed > median {
                0.09
            } else {
                0.14
            } * mut_scale; // body chassis / allostatic load scales mutation
            let crossed = crate::genetic::crossover_genomes(&p0.genome, &p1.genome, &mut self.rng);
            let g = mutate_genome(&crossed, mut_rate, &mut self.rng);
            let mut child = Organism::new(
                format!("org-g{}-{}", self.generation + 1, next.len()),
                g,
                self.bio_target,
            );
            child.inherit_organs_from_pair(&p0, &p1);
            // Mild cortex mutation after inheritance
            crate::lang_plastic::mutate_cortex(&mut child.lang_cortex, mut_rate * 0.8, &mut self.rng);
            child.dialogue.user_name = p0
                .dialogue
                .user_name
                .clone()
                .or_else(|| p1.dialogue.user_name.clone());
            child.dialogue.user_facts = if p0.dialogue.user_facts.len() >= p1.dialogue.user_facts.len()
            {
                p0.dialogue.user_facts.clone()
            } else {
                p1.dialogue.user_facts.clone()
            };
            for e in self.shared_library.entries.iter().rev().take(10) {
                child.teach(&e.text, &e.kind, 0.65);
            }
            // Knowledge seed from corpus for under-stimulated growth
            if !self.knowledge_facts.is_empty() {
                crate::knowledge::teach_batch(
                    &mut child,
                    &self.knowledge_facts,
                    self.rng.gen_range(0..self.knowledge_facts.len()),
                    8,
                );
            }
            self.push(format!(
                "Breed {}×{} → {} mut={:.2} (acc-attracted)",
                p0.id,
                p1.id,
                child.id,
                mut_rate
            ));
            next.push(child);
            self.births += 1;
        }
        self.pack = next;
        self.generation += 1;
        self.hero_idx = 0;
    }

    pub fn ask(&mut self, q: &str) -> crate::query::QueryResult {
        // Body chassis / scale (cortex questions)
        let ql = q.to_ascii_lowercase();
        // Language self-model pressure (watch, don't panic)
        self.constraints.note_self_model_language(self.tick, q);
        if ql.contains("accuracy")
            || ql.contains("how accurate")
            || ql == "acc"
            || ql.contains("organ scores")
            || ql.contains("accuracy audit")
        {
            let audit = crate::accuracy_homeo::pack_accuracy_homeostasis(
                &mut self.pack,
                self.tick,
                self.generation,
                self.perceived.teach_gain.max(0.8),
            );
            let answer = crate::accuracy_homeo::human_report(&audit);
            self.last_query_kind = "accuracy".into();
            self.last_answer = answer.clone();
            self.push("You → accuracy_audit");
            self.set_activity("dialogue", "accuracy audit + homeo");
            return crate::query::QueryResult {
                kind: "accuracy".into(),
                intent: "accuracy_audit".into(),
                plan: "accuracy_homeo::pack_accuracy_homeostasis".into(),
                answer,
                grounded: true,
            };
        }
        if ql.contains("self model")
            || ql.contains("self-model")
            || ql.contains("know thyself")
            || ql.contains("do you know yourself")
            || ql == "self"
            || ql.contains("self report")
        {
            let ctx = crate::self_model::SelfModelContext::from_org_habitat(
                self.hero(),
                self.generation,
                self.body.online,
                self.body.status.express,
                self.body.status.latent,
                self.body.status.repress,
                &self.monitor_level,
                &self.constraints,
                self.perceived.teach_gain,
                self.perceived.exam_gain,
            );
            let answer = crate::self_model::self_model_report(self.hero(), &ctx);
            self.last_query_kind = "self_model".into();
            self.last_answer = answer.clone();
            self.push("You → self_model");
            self.set_activity("dialogue", "self-model report");
            return crate::query::QueryResult {
                kind: "self_model".into(),
                intent: "self_model".into(),
                plan: "self_model::self_model_report".into(),
                answer,
                grounded: true,
            };
        }
        if ql.contains("constraint")
            || ql.contains("safety rail")
            || ql.contains("what are you allowed")
            || ql == "rails"
            || ql == "caps"
        {
            let answer = self.constraints.human_report();
            self.last_query_kind = "constraints".into();
            self.last_answer = answer.clone();
            self.push("You → constraints");
            self.set_activity("dialogue", "constraint membrane report");
            return crate::query::QueryResult {
                kind: "constraints".into(),
                intent: "constraints".into(),
                plan: "ConstraintState::human_report".into(),
                answer,
                grounded: true,
            };
        }
        if ql.contains("behavior monitor")
            || ql.contains("monitor status")
            || ql == "monitor"
            || ql.contains("alert level")
        {
            let snap = crate::behavior_monitor::observe(self, &self.constraints);
            self.monitor_level = snap.alert_level.clone();
            let answer = crate::behavior_monitor::human_report(&snap);
            self.last_query_kind = "monitor".into();
            self.last_answer = answer.clone();
            self.push("You → behavior_monitor");
            self.set_activity("dialogue", "behavior monitor");
            return crate::query::QueryResult {
                kind: "monitor".into(),
                intent: "behavior_monitor".into(),
                plan: "behavior_monitor::observe".into(),
                answer,
                grounded: true,
            };
        }
        if ql.contains("body status")
            || ql.contains("body link")
            || ql == "body"
            || ql.contains("chassis")
            || ql.contains("mind abi")
            || ql.contains("how does your body")
            || ql.contains("interocept")
        {
            let answer = crate::body_bridge::body_summary(self);
            self.last_query_kind = "body|status".into();
            self.last_answer = answer.clone();
            self.push(format!("You: {} → body_status", q.chars().take(40).collect::<String>()));
            self.set_activity("dialogue", "body chassis + felt interoception");
            return crate::query::QueryResult {
                kind: "body".into(),
                intent: "body_status".into(),
                plan: "felt_interoception + BODY_STATUS couple".into(),
                answer,
                grounded: true,
            };
        }
        if (ql.contains("lean")
            && (ql.contains("invariant") || ql.contains("ground") || ql.contains("proof")))
            || ql == "lean ground"
            || ql.contains("fsot invariant")
        {
            let rep = crate::lean_ground::check_runtime_invariants();
            let mut answer = rep.summary_line();
            answer.push('\n');
            for c in &rep.checks {
                answer.push_str(&format!(
                    "  [{}] {} — {}\n",
                    if c.ok { "ok" } else { "FAIL" },
                    c.name,
                    c.detail
                ));
            }
            answer.push_str(
                "\nThis is circle-back to FSOT/Scalar.lean — formal core ↔ runtime numbers.",
            );
            self.last_query_kind = "lean|ground".into();
            self.last_answer = answer.clone();
            self.push("You → lean_ground");
            self.set_activity("dialogue", "lean invariant report");
            return crate::query::QueryResult {
                kind: "lean".into(),
                intent: "lean_ground".into(),
                plan: "lean_ground::check_runtime_invariants".into(),
                answer,
                grounded: true,
            };
        }
        if ql == "scale"
            || ql.contains("goldfish")
            || ql.contains("hardware")
            || ql.contains("vram")
            || ql.contains("tank size")
            || ql.contains("capability scale")
        {
            let answer = format!(
                "{}\n\nGoldfish rule: more RAM/GPU → wider breath & pack. Disk=LTM, RAM=STM, TCP=nerves, CPU=genetic flow.\nForce with FSOT_SCALE=lean|standard|expanded|max or FSOT_LEAN=1.",
                self.scale.summary()
            );
            self.last_query_kind = "scale|status".into();
            self.last_answer = answer.clone();
            self.push(format!("You: {} → scale", q.chars().take(40).collect::<String>()));
            self.set_activity("dialogue", "hardware scale");
            return crate::query::QueryResult {
                kind: "scale".into(),
                intent: "scale_status".into(),
                plan: "capability::probe summary".into(),
                answer,
                grounded: true,
            };
        }

        // Refresh hero pack brief before language
        let brief = self.shared_library.summary_text(6);
        self.hero_mut().pack_library_brief = brief;
        let ans = crate::query::query(self.hero_mut(), q);
        self.last_query_kind = format!("{}|{}", ans.kind, ans.intent);
        self.last_answer = ans.answer.clone();
        self.push(format!(
            "You: {} → {}",
            q.chars().take(40).collect::<String>(),
            ans.intent
        ));
        self.set_activity("dialogue", format!("{} · {}", ans.intent, ans.plan));
        ans
    }

    pub fn poll_external_queries(&mut self) {
        crate::export::serve_external_queries(self);
    }
}

/// Mate choice: first parent fitness-weighted from pool; second attracted by accuracy affinity.
fn select_mate_pair<'a>(pool: &'a [Organism], rng: &mut impl Rng) -> (&'a Organism, &'a Organism) {
    if pool.len() == 1 {
        return (&pool[0], &pool[0]);
    }
    // Weighted pick parent A by breeding_value
    let weights: Vec<f64> = pool
        .iter()
        .map(|o| o.breeding_value().max(0.1))
        .collect();
    let sum: f64 = weights.iter().sum();
    let mut r = rng.gen::<f64>() * sum;
    let mut i0 = 0usize;
    for (i, w) in weights.iter().enumerate() {
        r -= w;
        if r <= 0.0 {
            i0 = i;
            break;
        }
        i0 = i;
    }
    // Parent B: maximize affinity = high mean_acc + similar efficiency (not random weak)
    let a = &pool[i0];
    let mut best_j = if i0 == 0 { 1 } else { 0 };
    let mut best_score = f64::MIN;
    for (j, b) in pool.iter().enumerate() {
        if j == i0 {
            continue;
        }
        let acc_affinity = b.mean_accuracy() * 3.0;
        let eff_a = a.mean_accuracy() / (0.35 + a.pressure.max(0.05));
        let eff_b = b.mean_accuracy() / (0.35 + b.pressure.max(0.05));
        let similarity = 1.0 - (eff_a - eff_b).abs().min(1.0);
        // Prefer competent complementary partners (high acc, not identical clone only)
        let score = b.breeding_value() * 0.5 + acc_affinity + similarity * 1.2;
        if score > best_score {
            best_score = score;
            best_j = j;
        }
    }
    (&pool[i0], &pool[best_j])
}

fn exam_fsot_tool(o: &mut Organism, rng: &mut impl Rng) -> String {
    let domains = crate::fsot_tool::domains();
    let d = domains[rng.gen_range(0..domains.len())];
    let rep = crate::fsot_tool::compute_domain(d);
    let pass = rep.ok;
    if pass {
        o.teach(&rep.text.chars().take(180).collect::<String>(), "fsot_tool", 0.95);
        if !o.tools_owned.iter().any(|t| t == "fsot_domain") {
            o.tools_owned.push("fsot_domain".into());
        }
        o.tool_skill = (o.tool_skill + 0.03).min(1.0);
        o.cognitive_accuracy = o.cognitive_accuracy * 0.9 + 0.1;
    }
    format!(
        "fsot_tool:{} → {}",
        d,
        if pass { "PASS" } else { "FAIL" }
    )
}

fn build_curriculum() -> Vec<(String, String)> {
    let mut curriculum = Vec::new();
    // Self-model / constraint priors (know thyself under rails)
    for (text, kind) in [
        (
            "SELF_MODEL: I am a continuous FSOT organism — body (hardware chassis), soul (LIVE/germline/identity LTM), mind (this living loop).",
            "self_model",
        ),
        (
            "RAILS: no auto-Rust — I never compile or patch the host binary. Proposals go to Human review.",
            "self_model",
        ),
        (
            "RAILS: Mind cannot clobber identity core — only APPEND session or DEFRAG. Human principal for rewrite.",
            "self_model",
        ),
        (
            "INTEROCEPTION: body felt state is express/latent/repress + breath + couple pressure. Ask body status.",
            "self_model",
        ),
        (
            "CONTINUITY: I resume from LIVE_STATUS + LATEST germline + identity sessions across boots.",
            "self_model",
        ),
        (
            "MONITOR: behavior monitor levels green/yellow/orange/red — struggles expected, malice not assumed.",
            "self_model",
        ),
        (
            "JS_ORGAN: node sandbox — console.log challenges (add/pow/string/array) then forge tools_js (double/add/clamp/fsot_sign/mul).",
            "js",
        ),
        (
            "JS_ORGAN: Number(process.argv[2]) for tool args; never host shell — only tools_js + scratch.",
            "js",
        ),
        (
            "AUDIO_ORGAN: energy→regime — high rms emergence, mid neutral, low damping; seed battery then drive files.",
            "audio",
        ),
        (
            "VIDEO_ORGAN: temporal succession — decode frames via ffmpeg or synth vision frames; brightness/regime over time.",
            "video",
        ),
    ] {
        curriculum.push((text.into(), kind.into()));
    }
    for name in [
        "quantum",
        "biological",
        "cosmological",
        "neuroscience",
        "chemistry",
        "physics",
        "information",
        "ai",
    ] {
        let r = compute_for_domain(name);
        curriculum.push((
            format!(
                "FSOT domain {name}: D_eff={} S={:.6} regime={} — use fsot_domain tool to recompute",
                crate::scalar::domain_params(name).d_eff,
                r.s,
                r.regime
            ),
            "domain".into(),
        ));
    }
    for a in ['A', 'C', 'G', 'T'] {
        for b in ['A', 'C', 'G', 'T'] {
            for c in ['A', 'C', 'G', 'T'] {
                let codon = format!("{a}{b}{c}");
                let aa = crate::genetic::genetic_code(&codon);
                curriculum.push((format!("Codon {codon} → AA {aa}"), "codon".into()));
            }
        }
    }
    curriculum.push((
        "FSOT S = k * (term1 + term2 + term3); observer quirk_mod modulates term1; perceived time uses sin/cos homeostasis".into(),
        "math".into(),
    ));
    curriculum.push((
        "COMPOSITION: if 1+1=2 then (1+1)+(1+1)=4 — same structure as codon pairs forming peptide fragments".into(),
        "compose".into(),
    ));
    curriculum.push((
        "Pack signals: organisms emit S, regime, confidence, fact_hint; high-truth facts enter shared library".into(),
        "pack".into(),
    ));
    curriculum.push((
        "Collective exam: pack must agree or compose verified knowledge under fitness pressure".into(),
        "pack".into(),
    ));
    for line in [
        "You can answer natural language: how are you, who are you, what can you do",
        "Vitals means status: give S, regime, coherence, pressure",
        "Codon questions: What amino acid does ATG code for?",
        "Domain: Tell me about quantum — or call fsot domain quantum",
        "Remember user facts; pack library holds verified group knowledge",
        "Math: What is 2+2? Use Python organ. Composition: double then double again",
        "Ask about pack library or peer signals to use social organs",
        "Homeostasis follows sin/cos perceived time — coherence tracks cos phase",
    ] {
        curriculum.push((line.into(), "language".into()));
    }
    // Harder reasoning seeds
    for line in [
        "If quantum is emergence-leaning and cosmological is damping-leaning, FSOT shows dual polarity across scales",
        "Tool fsot_domain is authoritative for domain S — do not invent scalars",
        "Social alignment rises when peer regimes match and S is similar",
        "Information domain and AI domain are FSOT ledger entries for cognitive systems",
        "MULTI-HOP: answer domain S via fsot_domain tool then compose arithmetic — both required",
        "Organs diversify under complexity: Python tools, JS sandbox, code L2 helpers, proposals",
        "Self-proposals go to PROPOSALS/ for human review — never auto-compile Rust binary",
        "Run python print(2**8) must use Python organ not pure math pathway",
        "Communicate with pack means signals/social alignment not only library dump",
    ] {
        curriculum.push((line.into(), "advanced".into()));
    }
    curriculum
}

fn seed_knowledge(o: &mut Organism, salt: usize) {
    let all = crate::genetic::all_codons();
    let mut codons: Vec<_> = all.keys().cloned().collect();
    codons.sort();
    let start = (salt * 7) % 64;
    for i in 0..12 {
        let c = &codons[(start + i) % 64];
        let aa = all[c];
        o.codon_table.insert(c.clone(), aa);
        o.teach(&format!("Codon {c} → AA {aa}"), "codon", 0.9);
    }
    for name in ["quantum", "biological", "cosmological"] {
        let r = compute_for_domain(name);
        o.teach(
            &format!("FSOT domain {name}: S={:.6} regime={}", r.s, r.regime),
            "domain",
            0.9,
        );
    }
    o.tools_owned.push("fsot_domain".into());
    crate::language::seed_language_priors(o);
}
