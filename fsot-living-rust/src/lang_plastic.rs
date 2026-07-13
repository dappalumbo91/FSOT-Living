//! Adaptive language cortex — biological plasticity, not LLM weights
//!
//! Model (developmental + evolutionary):
//! 1. Sensory feature detectors on the utterance (like cortical receptive fields)
//! 2. Plastic pathways: feature → action (like adaptive synapses / routes)
//! 3. Hebbian + exam reward: use it + succeed → myelinate; fail → weaken / explore
//! 4. Compositional speech: ordered utterance primitives recombined (syntax evolution)
//! 5. Inherited cortex at birth with mutation (vertical culture + variation)
//!
//! No pretrained transformer. Generalization = pathway coverage under pressure.

use rand::Rng;
use serde::Serialize;

pub const FEAT_DIM: usize = 32;

/// Motor / cognitive action a pathway can drive (organ call).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LangAct {
    Feel,
    Status,
    Identity,
    Help,
    Capabilities,
    Codon,
    Domain,
    Genome,
    Fitness,
    Organs,
    Tools,
    Math,
    Python,
    Vision,
    Forge,
    Remember,
    Recall,
    Fsot,
    FsotTool,
    PackLib,
    PackSig,
    /// Causal self-model: why regime / how pressure
    Causal,
    Unknown,
}

/// Utterance primitives — biological "speech gestures" composed into answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Primitive {
    Address,
    StateS,
    Regime,
    PressureMood,
    OrganSummary,
    FactBody,
    CausalLink,
    PackNote,
    OfferNext,
    GroundFooter,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pathway {
    pub id: u32,
    pub weights: Vec<f32>,
    pub strength: f64,
    pub act: LangAct,
    pub syntax: Vec<Primitive>,
    pub uses: u64,
    pub successes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LangCortex {
    pub pathways: Vec<Pathway>,
    /// Plasticity rate (modulated by homeostasis / pressure)
    pub plasticity: f64,
    pub next_id: u32,
    pub generalize_accuracy: f64,
    pub last_activation: f64,
    pub last_acts: Vec<String>,
    pub pathway_count: usize,
}

impl Default for LangCortex {
    fn default() -> Self {
        innate_cortex()
    }
}

impl LangCortex {
    pub fn refresh_meta(&mut self) {
        self.pathway_count = self.pathways.len();
    }
}

/// Innate priors — like evolved speech circuits present at birth, then plastic.
pub fn innate_cortex() -> LangCortex {
    let mut c = LangCortex {
        pathways: Vec::new(),
        plasticity: 0.12,
        next_id: 1,
        generalize_accuracy: 0.0,
        last_activation: 0.0,
        last_acts: Vec::new(),
        pathway_count: 0,
    };
    // Feature index map documented in extract_features
    seed(
        &mut c,
        LangAct::Feel,
        &[(1, 1.2), (3, 1.4), (2, 0.4)],
        vec![
            Primitive::Address,
            Primitive::Regime,
            Primitive::PressureMood,
            Primitive::StateS,
            Primitive::OfferNext,
        ],
        1.0,
    );
    seed(
        &mut c,
        LangAct::Status,
        &[(3, 1.5), (2, 0.6), (0, 0.3)],
        vec![
            Primitive::Address,
            Primitive::StateS,
            Primitive::Regime,
            Primitive::OrganSummary,
            Primitive::GroundFooter,
        ],
        1.0,
    );
    seed(
        &mut c,
        LangAct::Identity,
        &[(4, 1.6), (2, 0.5), (16, 0.4)],
        vec![Primitive::Address, Primitive::FactBody, Primitive::StateS],
        0.95,
    );
    seed(
        &mut c,
        LangAct::Codon,
        &[(5, 1.8), (6, 1.2), (0, 0.3)],
        vec![Primitive::FactBody, Primitive::GroundFooter],
        1.1,
    );
    seed(
        &mut c,
        LangAct::Domain,
        &[(7, 1.5), (8, 1.0), (9, 1.0), (16, 0.6)],
        vec![Primitive::FactBody, Primitive::StateS, Primitive::GroundFooter],
        1.0,
    );
    seed(
        &mut c,
        LangAct::Math,
        &[(10, 1.8), (0, 0.4)],
        vec![Primitive::FactBody, Primitive::GroundFooter],
        1.0,
    );
    seed(
        &mut c,
        LangAct::Remember,
        &[(11, 1.8)],
        vec![Primitive::FactBody, Primitive::OfferNext],
        1.0,
    );
    seed(
        &mut c,
        LangAct::Recall,
        &[(12, 1.5), (11, 0.3)],
        vec![Primitive::FactBody, Primitive::PackNote],
        0.9,
    );
    seed(
        &mut c,
        LangAct::Fsot,
        &[(16, 1.6), (13, 0.3)],
        vec![Primitive::FactBody, Primitive::StateS, Primitive::CausalLink],
        1.0,
    );
    seed(
        &mut c,
        LangAct::FsotTool,
        &[(16, 1.2), (7, 0.8)],
        vec![Primitive::FactBody, Primitive::GroundFooter],
        0.95,
    );
    seed(
        &mut c,
        LangAct::PackLib,
        &[(15, 2.2), (27, 0.4), (12, 0.3)], // library / show knowledge
        vec![Primitive::PackNote, Primitive::FactBody],
        1.2,
    );
    seed(
        &mut c,
        LangAct::PackSig,
        &[(29, 2.0), (2, 0.5), (14, 0.5)], // communicate / signal only — no library feature
        vec![Primitive::PackNote, Primitive::CausalLink],
        1.1,
    );
    seed(
        &mut c,
        LangAct::Python,
        &[(21, 2.0), (10, 0.2)], // python strongly preferred over bare math when py cue
        vec![Primitive::FactBody],
        1.15,
    );
    seed(
        &mut c,
        LangAct::Causal,
        &[(13, 1.8), (14, 1.0), (3, 0.5), (16, 0.4)],
        vec![
            Primitive::CausalLink,
            Primitive::StateS,
            Primitive::Regime,
            Primitive::PressureMood,
            Primitive::OfferNext,
        ],
        1.05,
    );
    seed(
        &mut c,
        LangAct::Genome,
        &[(17, 1.7)],
        vec![Primitive::FactBody],
        0.9,
    );
    seed(
        &mut c,
        LangAct::Fitness,
        &[(18, 1.7)],
        vec![Primitive::OrganSummary, Primitive::StateS],
        0.9,
    );
    seed(
        &mut c,
        LangAct::Organs,
        &[(30, 1.9), (25, 0.3), (19, 0.3)],
        vec![Primitive::OrganSummary, Primitive::OfferNext],
        1.05,
    );
    seed(
        &mut c,
        LangAct::Capabilities,
        &[(25, 1.6), (24, 0.5)],
        vec![Primitive::FactBody, Primitive::OfferNext],
        0.9,
    );
    seed(
        &mut c,
        LangAct::Help,
        &[(24, 1.7)],
        vec![Primitive::FactBody, Primitive::OfferNext],
        0.9,
    );
    seed(
        &mut c,
        LangAct::Tools,
        &[(19, 1.5)],
        vec![Primitive::FactBody],
        0.85,
    );
    seed(
        &mut c,
        LangAct::Vision,
        &[(20, 1.6)],
        vec![Primitive::FactBody],
        0.85,
    );

    seed(
        &mut c,
        LangAct::Forge,
        &[(19, 1.2)],
        vec![Primitive::FactBody],
        0.8,
    );
    seed(
        &mut c,
        LangAct::Unknown,
        &[(23, 1.4), (31, 0.2)],
        vec![Primitive::Address, Primitive::OfferNext, Primitive::PackNote],
        0.7,
    );
    // Weak exploratory pathways (variation for evolution)
    for _ in 0..4 {
        let act = LangAct::Recall;
        let mut w = vec![0.0f32; FEAT_DIM];
        for i in 0..FEAT_DIM {
            w[i] = 0.05;
        }
        w[12] = 0.4;
        w[31] = 0.1;
        let id = c.next_id;
        c.next_id += 1;
        c.pathways.push(Pathway {
            id,
            weights: w,
            strength: 0.25,
            act,
            syntax: vec![Primitive::FactBody, Primitive::OfferNext],
            uses: 0,
            successes: 0,
        });
    }
    c.refresh_meta();
    c
}

fn seed(
    c: &mut LangCortex,
    act: LangAct,
    feats: &[(usize, f32)],
    syntax: Vec<Primitive>,
    strength: f64,
) {
    let mut w = vec![0.0f32; FEAT_DIM];
    w[31] = 0.15; // bias
    for (i, v) in feats {
        if *i < FEAT_DIM {
            w[*i] = *v;
        }
    }
    let id = c.next_id;
    c.next_id += 1;
    c.pathways.push(Pathway {
        id,
        weights: w,
        strength,
        act,
        syntax,
        uses: 0,
        successes: 0,
    });
}

/// Sensory encoding of an utterance — fixed detectors, plastic readouts.
pub fn extract_features(q: &str) -> Vec<f32> {
    let mut f = vec![0.0f32; FEAT_DIM];
    let lower = q.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '+')
        .filter(|w| !w.is_empty())
        .collect();

    f[31] = 1.0; // bias
    f[28] = (words.len() as f32 / 24.0).min(1.0);
    if lower.contains('?') || words.iter().any(|w| matches!(*w, "what" | "why" | "how" | "who" | "where" | "when")) {
        f[0] = 1.0;
    }
    if words.iter().any(|w| matches!(*w, "hello" | "hi" | "hey" | "greetings")) {
        f[1] = 1.0;
    }
    if words.iter().any(|w| matches!(*w, "you" | "your" | "yourself" | "u")) {
        f[2] = 1.0;
    }
    if lower.contains("how are you")
        || lower.contains("feeling")
        || lower.contains("vital")
        || lower.contains("status")
        || lower.contains("rundown")
        || lower.contains("state")
        || lower.contains("scalar")
    {
        f[3] = 1.0;
    }
    if lower.contains("who are you")
        || lower.contains("what are you")
        || lower.contains("your name")
        || lower.contains("introduce")
    {
        f[4] = 1.0;
    }
    // codon token
    for w in q.split_whitespace() {
        let u = w
            .trim_matches(|c: char| !c.is_ascii_alphanumeric())
            .to_uppercase();
        if u.len() == 3 && u.chars().all(|c| matches!(c, 'A' | 'C' | 'G' | 'T')) {
            f[5] = 1.0;
            break;
        }
    }
    if lower.contains("amino") || lower.contains("codon") || lower.contains("translate") || lower.contains("codes for") {
        f[6] = 1.0;
    }
    if lower.contains("domain") || lower.contains("ledger") {
        f[7] = 1.0;
    }
    if lower.contains("quantum") {
        f[8] = 1.0;
    }
    if lower.contains("cosmo") {
        f[9] = 1.0;
    }
    if lower.contains("bio") {
        f[26] = 1.0;
    }
    // numbers / math
    if words.iter().any(|w| w.chars().any(|c| c.is_ascii_digit()))
        || lower.contains('+')
        || lower.contains("plus")
        || lower.contains("times")
        || lower.contains("multiply")
    {
        f[10] = 1.0;
    }
    if lower.contains("remember") || lower.contains("my name is") || lower.contains("note that") {
        f[11] = 1.0;
    }
    if lower.contains("what is my")
        || lower.contains("do you know")
        || lower.contains("what do you know")
        || lower.contains("recall")
        || lower.contains("what have you")
    {
        f[12] = 1.0;
    }
    if lower.contains("why") || lower.contains("because") || lower.contains("cause") || lower.contains("reason") {
        f[13] = 1.0;
    }
    if lower.starts_with("how ") || lower.contains("how does") || lower.contains("how do ") {
        f[14] = 1.0;
    }
    let wants_library = lower.contains("library")
        || lower.contains("collective knowledge")
        || lower.contains("shared knowledge")
        || lower.contains("what has the pack verified")
        || lower.contains("pack library")
        || (lower.contains("show") && lower.contains("pack") && !lower.contains("communicate"));
    let wants_signals = lower.contains("communicate")
        || lower.contains("signal")
        || lower.contains("peer signal")
        || lower.contains("talk to your pack")
        || lower.contains("do you talk")
        || (lower.contains("with your pack") && !lower.contains("library"));
    if wants_library {
        f[15] = 1.0; // library
        f[29] = 0.0; // suppress signal path
    } else if wants_signals {
        f[29] = 1.0; // social signal / communicate
        f[15] = 0.0;
    } else if lower.contains("pack") {
        f[15] = 0.5; // ambiguous pack mention → mild library lean
    }
    if lower.contains("fsot") || lower.contains("term1") || lower.contains("framework") {
        f[16] = 1.0;
    }
    if lower.contains("genome") || lower.contains("genes") {
        f[17] = 1.0;
    }
    if lower.contains("fit") || lower.contains("fitness") || lower.contains("evolutionarily") {
        f[18] = 1.0;
    }
    if lower.contains("tool") || lower.contains("forge") {
        f[19] = 1.0;
    }
    if lower.contains("see") || lower.contains("vision") || lower.contains("image") || lower.contains("look at") {
        f[20] = 1.0;
    }
    if lower.contains("python")
        || lower.contains("py:")
        || lower.contains("run python")
        || lower.contains("print(")
        || lower.contains("**")
        || lower.contains("node")
        || lower.contains("javascript")
    {
        f[21] = 1.0;
        // demote pure-math grab when code organ is requested
        if lower.contains("python") || lower.contains("print") || lower.contains("**") {
            f[10] *= 0.25;
        }
    } else if lower.contains("code") && !lower.contains("codon") {
        f[21] = 0.8;
    }
    if lower.contains(" and ") || lower.contains(" also ") || lower.contains(" as well ") || lower.contains(',') {
        f[22] = 1.0;
    }
    let junk = ["blorple", "ziggurat", "xyzzy", "qwerty", "asdf", "banana99", "zzz"];
    if junk.iter().any(|j| lower.contains(j)) {
        f[23] = 1.0;
    }
    if lower.contains("help") || lower.contains("what can i ask") {
        f[24] = 1.0;
    }
    if lower == "organs"
        || lower == "skills"
        || lower.contains("organ status")
        || lower.contains("your organs")
    {
        f[30] = 1.0;
    }
    if lower.contains("what can you") || lower.contains("capabilit") || lower.contains("multimodal") {
        f[25] = 1.0;
    }
    if lower.contains("compose") || lower.contains("same as") || lower.contains("therefore") || lower.contains("if ") && lower.contains(" then") {
        f[27] = 1.0;
    }
    if lower.contains('!') {
        f[29] = 0.5;
    }
    f
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[derive(Clone, Debug)]
pub struct Activation {
    pub pathway_id: u32,
    pub act: LangAct,
    pub score: f64,
    pub syntax: Vec<Primitive>,
}

/// Compete pathways (winner-take-most). Multi-intent if compound feature high.
pub fn activate(cortex: &LangCortex, feats: &[f32], compound_threshold: f64) -> Vec<Activation> {
    let mut scored: Vec<Activation> = cortex
        .pathways
        .iter()
        .map(|p| {
            let raw = dot(feats, &p.weights) as f64;
            let score = raw * (0.35 + 0.65 * p.strength);
            Activation {
                pathway_id: p.id,
                act: p.act,
                score,
                syntax: p.syntax.clone(),
            }
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    if scored.is_empty() {
        return vec![Activation {
            pathway_id: 0,
            act: LangAct::Unknown,
            score: 0.0,
            syntax: vec![Primitive::OfferNext],
        }];
    }

    let mut winners = vec![scored[0].clone()];
    // Compound questions: second distinct action if strong enough
    let compound = feats.get(22).copied().unwrap_or(0.0) as f64;
    if compound > 0.5 && scored.len() > 1 {
        for s in scored.iter().skip(1).take(4) {
            if s.act != winners[0].act && s.score > compound_threshold * winners[0].score {
                winners.push(s.clone());
                if winners.len() >= 2 {
                    break;
                }
            }
        }
    }
    // Suppress Unknown if junk low and something else scored
    if winners[0].act == LangAct::Unknown && winners[0].score < 0.8 {
        if let Some(alt) = scored.iter().find(|s| s.act != LangAct::Unknown && s.score > 0.35) {
            winners[0] = alt.clone();
        }
    }
    // Junk high → force Unknown over Domain hijack
    let junk = feats.get(23).copied().unwrap_or(0.0);
    if junk > 0.5 {
        winners = vec![Activation {
            pathway_id: 0,
            act: LangAct::Unknown,
            score: 1.0,
            syntax: vec![Primitive::Address, Primitive::OfferNext],
        }];
    }
    winners
}

/// Hebbian + reward plasticity after an outcome (exam or grounded dialogue).
pub fn plasticize(
    cortex: &mut LangCortex,
    feats: &[f32],
    winner_ids: &[u32],
    success: bool,
    homeo_plasticity: f64,
) {
    let lr = (cortex.plasticity * (0.5 + homeo_plasticity)).clamp(0.02, 0.35);
    for p in cortex.pathways.iter_mut() {
        if winner_ids.contains(&p.id) {
            p.uses += 1;
            if success {
                p.successes += 1;
                p.strength = (p.strength + lr * 0.15).min(2.0);
                // Hebbian: strengthen co-active features
                for i in 0..FEAT_DIM.min(p.weights.len()) {
                    if feats[i] > 0.2 {
                        p.weights[i] = (p.weights[i] + lr as f32 * feats[i] * 0.35).min(2.5);
                    }
                }
            } else {
                p.strength = (p.strength - lr * 0.12).max(0.08);
                for i in 0..FEAT_DIM.min(p.weights.len()) {
                    if feats[i] > 0.2 {
                        p.weights[i] = (p.weights[i] - lr as f32 * feats[i] * 0.2).max(-0.5);
                    }
                }
            }
        }
    }
    let sample = if success { 1.0 } else { 0.0 };
    // Fast recovery when gen_acc is under mastery bar (aligned with GEN_ACC_TARGET)
    let lr = if cortex.generalize_accuracy < 0.40 {
        0.40
    } else if cortex.generalize_accuracy < 0.55 {
        0.32
    } else if cortex.generalize_accuracy < 0.75 {
        0.24
    } else {
        0.12
    };
    cortex.generalize_accuracy =
        (cortex.generalize_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
}

/// Evolutionary mutation of cortex (birth / high pressure).
pub fn mutate_cortex(cortex: &mut LangCortex, rate: f64, rng: &mut impl Rng) {
    for p in cortex.pathways.iter_mut() {
        if rng.gen::<f64>() < rate {
            let i = rng.gen_range(0..FEAT_DIM.min(p.weights.len()));
            p.weights[i] = (p.weights[i] + (rng.gen::<f32>() - 0.5) * 0.4).clamp(-1.0, 2.5);
        }
        if rng.gen::<f64>() < rate * 0.5 {
            p.strength = (p.strength + (rng.gen::<f64>() - 0.45) * 0.1).clamp(0.08, 2.0);
        }
    }
    // Neurogenesis under variation: add exploratory pathway
    if rng.gen::<f64>() < rate * 0.8 && cortex.pathways.len() < 48 {
        let acts = [
            LangAct::Causal,
            LangAct::Recall,
            LangAct::Status,
            LangAct::Feel,
            LangAct::Domain,
            LangAct::Codon,
        ];
        let act = acts[rng.gen_range(0..acts.len())];
        let mut w = vec![0.0f32; FEAT_DIM];
        for i in 0..FEAT_DIM {
            w[i] = (rng.gen::<f32>() - 0.5) * 0.3;
        }
        w[31] = 0.2;
        let id = cortex.next_id;
        cortex.next_id += 1;
        cortex.pathways.push(Pathway {
            id,
            weights: w,
            strength: 0.3,
            act,
            syntax: vec![
                Primitive::FactBody,
                Primitive::CausalLink,
                Primitive::OfferNext,
            ],
            uses: 0,
            successes: 0,
        });
    }
    // Prune dead pathways
    if cortex.pathways.len() > 20 {
        cortex.pathways.retain(|p| p.strength > 0.12 || p.uses < 3);
    }
    cortex.refresh_meta();
}

/// Cultural inheritance: clone parent cortex with mild mutation.
pub fn inherit_cortex(parent: &LangCortex, rng: &mut impl Rng) -> LangCortex {
    let mut child = parent.clone();
    child.generalize_accuracy *= 0.9;
    child.last_acts.clear();
    mutate_cortex(&mut child, 0.12, rng);
    child
}

/// Map plastic act → language Intent-compatible kind string for execute bridge.
pub fn act_name(a: LangAct) -> &'static str {
    match a {
        LangAct::Feel => "feeling",
        LangAct::Status => "status",
        LangAct::Identity => "identity",
        LangAct::Help => "help",
        LangAct::Capabilities => "capabilities",
        LangAct::Codon => "codon",
        LangAct::Domain => "domain",
        LangAct::Genome => "genome",
        LangAct::Fitness => "fitness",
        LangAct::Organs => "organs",
        LangAct::Tools => "tools",
        LangAct::Math => "math",
        LangAct::Python => "python",
        LangAct::Vision => "vision",
        LangAct::Forge => "forge",
        LangAct::Remember => "remember",
        LangAct::Recall => "recall",
        LangAct::Fsot => "fsot",
        LangAct::FsotTool => "fsot_tool",
        LangAct::PackLib => "pack_library",
        LangAct::PackSig => "pack_signals",
        LangAct::Causal => "causal",
        LangAct::Unknown => "unknown",
    }
}

/// Generalization exam prompts (held-out paraphrases + causal).
pub fn generalization_prompts() -> &'static [(&'static str, LangAct, &'static [&'static str])] {
    &[
        (
            "Could you give me a sense of how you're doing?",
            LangAct::Feel,
            &["s=", "regime", "emergence", "damping", "neutral"],
        ),
        (
            "Why are you in your current regime?",
            LangAct::Causal,
            &["because", "s=", "regime", "pressure", "drive", "emergence", "damping"],
        ),
        (
            "Explain the reason for your current FSOT regime in plain language.",
            LangAct::Causal,
            &["s=", "regime", "threshold", "pressure", "drive", "because"],
        ),
        (
            "In plain words, what does the codon ATG stand for?",
            LangAct::Codon,
            &["m", "atg", "amino"],
        ),
        (
            "I'm curious about quantum in the FSOT ledger — what's S?",
            LangAct::Domain,
            &["s=", "quantum"],
        ),
        (
            "If you had to introduce yourself briefly, who are you?",
            LangAct::Identity,
            &["fsot", "organism"],
        ),
        (
            "How evolutionarily successful do you feel right now?",
            LangAct::Fitness,
            &["fitness", "fit"],
        ),
        (
            "What's sitting in the pack's shared knowledge?",
            LangAct::PackLib,
            &["pack", "library", "collective", "trust", "empty"],
        ),
        (
            "Show me the pack library",
            LangAct::PackLib,
            &["library", "pack", "trust", "empty", "codon"],
        ),
        (
            "Add three and nine for me would you?",
            LangAct::Math,
            &["12"],
        ),
        (
            "What is the golden ratio approximately?",
            LangAct::Recall,
            &["1.618", "phi", "golden"],
        ),
        (
            "How can you help me as an assistant?",
            LangAct::Capabilities,
            &["fsot", "ground", "tool", "assist"],
        ),
        (
            "In a few words, what state are you in right now?",
            LangAct::Feel,
            &["s=", "regime", "emergence", "damping", "neutral", "pressure"],
        ),
        (
            "Walk me through why your S puts you in this regime.",
            LangAct::Causal,
            &["because", "s=", "regime", "pressure", "emergence", "damping"],
        ),
        (
            "What amino acid does the start codon ATG encode?",
            LangAct::Codon,
            &["m", "atg", "amino", "methionine"],
        ),
        (
            "What's the FSOT S value in the cosmological domain?",
            LangAct::Domain,
            &["s=", "cosmo", "damping", "regime"],
        ),
        (
            "Sum fourteen and eight for me please.",
            LangAct::Math,
            &["22"],
        ),
        (
            "What tools or skills do you currently have available?",
            LangAct::Capabilities,
            &["tool", "organ", "code", "fsot", "assist", "python", "js"],
        ),
        (
            "Tell me your fitness in evolutionary terms.",
            LangAct::Fitness,
            &["fitness", "fit"],
        ),
        (
            "Who or what are you, stripped of fluff?",
            LangAct::Identity,
            &["fsot", "organism", "living"],
        ),
    ]
}
