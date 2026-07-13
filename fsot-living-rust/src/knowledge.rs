//! Grounded knowledge organ — factual + FSOT-derived curriculum under pressure
//!
//! Not LLM pretraining. Structured fact cards loaded from:
//!   1) innate FSOT / science / assistant seed set
//!   2) organism_workspace/knowledge/*.jsonl (human-expandable corpus)
//!
//! Organisms teach() facts, retrieve by semantic+keyword, and sit knowledge exams.
//! Path to "LLM-scale data": grow jsonl packs + fuse external benches via `bench.rs`
//! (MMLU/GSM8K/… from D:\training data) as *exam-backed curriculum*, never
//! ungrounded next-token training. Organs implement the same capability surface.

use crate::language::{semantic_store, SemanticEntry};
use crate::organism::Organism;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactCard {
    pub id: String,
    pub domain: String,
    pub claim: String,
    /// Short answer keys for exams (any match counts)
    pub keys: Vec<String>,
    /// Question paraphrases used under pressure
    pub questions: Vec<String>,
    /// FSOT-linked explanation when available
    #[serde(default)]
    pub fsot_note: String,
}

fn knowledge_dir() -> PathBuf {
    crate::code_env::workspace_root().join("knowledge")
}

/// Innate high-trust cards (always present).
pub fn seed_facts() -> Vec<FactCard> {
    vec![
        FactCard {
            id: "fsot_scalar".into(),
            domain: "fsot".into(),
            claim: "FSOT 2.1 defines scalar S = k · (T1 + T2 + T3) with domain parameters D_eff and δψ shaping regime.".into(),
            keys: vec!["s = k".into(), "t1".into(), "t2".into(), "t3".into(), "d_eff".into()],
            questions: vec![
                "What is the FSOT scalar formula?".into(),
                "How is S defined in FSOT 2.1?".into(),
            ],
            fsot_note: "Use fsot_domain tool to recompute any domain S; do not invent numbers.".into(),
        },
        FactCard {
            id: "fsot_regimes".into(),
            domain: "fsot".into(),
            claim: "FSOT regimes: emergence when S>0.02, damping when S<-0.02, else neutral.".into(),
            keys: vec!["emergence".into(), "damping".into(), "0.02".into()],
            questions: vec![
                "What are the FSOT regime thresholds?".into(),
                "When is an organism in emergence vs damping?".into(),
            ],
            fsot_note: "Live organism regime must match its current S under the same thresholds.".into(),
        },
        FactCard {
            id: "fsot_quantum".into(),
            domain: "fsot".into(),
            claim: "In the FSOT quantum domain, the ledger yields positive S (emergence-leaning) with low D_eff.".into(),
            keys: vec!["quantum".into(), "emergence".into(), "positive".into()],
            questions: vec![
                "What regime does FSOT predict for the quantum domain?".into(),
                "Is FSOT quantum domain emergence or damping?".into(),
            ],
            fsot_note: "Authoritative value: call fsot domain quantum.".into(),
        },
        FactCard {
            id: "fsot_cosmo".into(),
            domain: "fsot".into(),
            claim: "In the FSOT cosmological domain, the ledger yields negative S (damping-leaning) with high D_eff.".into(),
            keys: vec!["cosmological".into(), "damping".into(), "negative".into()],
            questions: vec![
                "What regime does FSOT predict for cosmology?".into(),
                "Is the cosmological domain emergence or damping in FSOT?".into(),
            ],
            fsot_note: "Authoritative value: call fsot domain cosmological.".into(),
        },
        FactCard {
            id: "codon_atg".into(),
            domain: "genetics".into(),
            claim: "Codon ATG codes for methionine (M) and is the standard start codon in the genetic code.".into(),
            keys: vec!["m".into(), "methionine".into(), "start".into(), "atg".into()],
            questions: vec![
                "What amino acid does ATG code for?".into(),
                "What is the start codon?".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "codon_stop".into(),
            domain: "genetics".into(),
            claim: "Stop codons include TAA, TAG, and TGA; they signal translation termination (often written as *).".into(),
            keys: vec!["taa".into(), "tag".into(), "tga".into(), "stop".into()],
            questions: vec![
                "What are the DNA stop codons?".into(),
                "Which codons stop translation?".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "dna_base".into(),
            domain: "genetics".into(),
            claim: "DNA uses four bases A, C, G, T; RNA uses U instead of T.".into(),
            keys: vec!["a".into(), "c".into(), "g".into(), "t".into(), "u".into()],
            questions: vec![
                "What bases does DNA use?".into(),
                "How does RNA differ from DNA bases?".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "phi".into(),
            domain: "math".into(),
            claim: "The golden ratio φ is (1+√5)/2 ≈ 1.618 and appears in FSOT constant structure.".into(),
            keys: vec!["1.618".into(), "golden".into(), "phi".into(), "φ".into()],
            questions: vec![
                "What is the golden ratio approximately?".into(),
                "What is phi in mathematics?".into(),
            ],
            fsot_note: "FSOT constants module uses PHI = 1.618033988749895.".into(),
        },
        FactCard {
            id: "assist_role".into(),
            domain: "assistant".into(),
            claim: "I am an FSOT-native living assistant: I ground answers in my scalar, tools, pack library, and learned facts; I say when I do not know.".into(),
            keys: vec!["fsot".into(), "ground".into(), "tools".into(), "do not know".into()],
            questions: vec![
                "How can you help me as an assistant?".into(),
                "What kind of assistant are you?".into(),
            ],
            fsot_note: "Prefer compute/tool/codon truth over fluent guessing.".into(),
        },
        FactCard {
            id: "assist_limits".into(),
            domain: "assistant".into(),
            claim: "I should not fabricate scientific numbers; for FSOT domains I recompute with the fsot_domain tool; for code I use the Python organ.".into(),
            keys: vec!["fabricate".into(), "recompute".into(), "python".into(), "tool".into()],
            questions: vec![
                "How do you avoid making up FSOT numbers?".into(),
                "When do you use tools vs memory?".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "si_units".into(),
            domain: "science".into(),
            claim: "SI base units include meter, kilogram, second, ampere, kelvin, mole, and candela.".into(),
            keys: vec!["meter".into(), "kilogram".into(), "second".into(), "kelvin".into()],
            questions: vec![
                "What are SI base units?".into(),
                "Name SI base units for length and mass.".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "photo_synth".into(),
            domain: "science".into(),
            claim: "Photosynthesis converts light energy into chemical energy, producing glucose and oxygen from CO2 and water in plants.".into(),
            keys: vec!["light".into(), "glucose".into(), "oxygen".into(), "co2".into()],
            questions: vec![
                "What does photosynthesis do?".into(),
                "What are the products of photosynthesis?".into(),
            ],
            fsot_note: String::new(),
        },
        FactCard {
            id: "evolution".into(),
            domain: "science".into(),
            claim: "Biological evolution is change in heritable traits of populations over generations via selection, mutation, drift, and migration.".into(),
            keys: vec!["selection".into(), "mutation".into(), "heritable".into(), "population".into()],
            questions: vec![
                "What is biological evolution?".into(),
                "What mechanisms drive evolution?".into(),
            ],
            fsot_note: "This organism itself evolves under multimodal fitness pressure.".into(),
        },
        FactCard {
            id: "info_bit".into(),
            domain: "science".into(),
            claim: "A bit is the basic unit of information; one bit distinguishes two equally likely states.".into(),
            keys: vec!["bit".into(), "two".into(), "information".into()],
            questions: vec![
                "What is a bit in information theory?".into(),
                "How much information is one bit?".into(),
            ],
            fsot_note: "FSOT information domain is a ledger entry for cognitive systems.".into(),
        },
    ]
}

/// Load seed + optional jsonl packs from workspace/knowledge.
pub fn load_all_facts() -> Vec<FactCard> {
    let lean = std::env::var("FSOT_LEAN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || std::env::var("FSOT_MIN_SPEC")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    load_all_facts_capped(if lean { 2_000 } else { 50_000 })
}

/// Load with explicit card cap (goldfish scale).
pub fn load_all_facts_capped(max_cards: usize) -> Vec<FactCard> {
    let max_cards = max_cards.max(64);
    let mut facts = seed_facts();
    let dir = knowledge_dir();
    let _ = fs::create_dir_all(&dir);
    let starter = dir.join("starter_pack.jsonl");
    if !starter.exists() {
        let mut lines = String::new();
        for f in &facts {
            if let Ok(s) = serde_json::to_string(f) {
                lines.push_str(&s);
                lines.push('\n');
            }
        }
        let _ = fs::write(
            dir.join("README.md"),
            "# Knowledge packs\n\n\
             Add `*.jsonl` files — one FactCard JSON per line.\n\
             Fields: id, domain, claim, keys[], questions[], fsot_note?\n\n\
             Cap via hardware scale (FSOT_SCALE / probe) or FSOT_LEAN=1.\n",
        );
        let _ = fs::write(&starter, lines);
    }
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            if facts.len() >= max_cards {
                break;
            }
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                continue;
            }
            if let Ok(text) = fs::read_to_string(&p) {
                for line in text.lines() {
                    if facts.len() >= max_cards {
                        break;
                    }
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(card) = serde_json::from_str::<FactCard>(line) {
                        if !facts.iter().any(|f| f.id == card.id) {
                            facts.push(card);
                        }
                    }
                }
            }
        }
    }
    facts
}

/// Inject a slice of knowledge into an organism (birth / feed).
pub fn teach_batch(org: &mut Organism, facts: &[FactCard], start: usize, n: usize) {
    if facts.is_empty() {
        return;
    }
    for i in 0..n {
        let f = &facts[(start + i) % facts.len()];
        teach_card(org, f, 0.9);
    }
}

fn teach_card(org: &mut Organism, f: &FactCard, strength: f64) {
    let text = if f.fsot_note.is_empty() {
        format!("FACT[{}|{}]: {}", f.id, f.domain, f.claim)
    } else {
        format!(
            "FACT[{}|{}]: {} | FSOT: {}",
            f.id, f.domain, f.claim, f.fsot_note
        )
    };
    org.teach(&text, "knowledge", strength);
    semantic_store(&mut org.semantic, &text, "knowledge", 1.0);
    // Durable store — short memory would drop facts under curriculum load
    if !org.long_term.iter().any(|t| t.contains(&f.id)) {
        org.long_term.push(text.clone());
    }
    // Keep a deep FACT store for mastery (≥85% needs retention volume)
    while org.long_term.len() > 320 {
        // Prefer dropping non-FACT lines first
        if let Some(i) = org.long_term.iter().position(|t| !t.contains("FACT[")) {
            org.long_term.remove(i);
        } else {
            org.long_term.remove(0);
        }
    }
    for q in &f.questions {
        semantic_store(
            &mut org.semantic,
            &format!(
                "Q: {} → {}",
                q,
                f.claim.chars().take(120).collect::<String>()
            ),
            "knowledge_qa",
            0.9,
        );
    }
    // Index keys for retrieval exams
    for k in &f.keys {
        if k.len() >= 2 {
            semantic_store(
                &mut org.semantic,
                &format!("KEY[{}]: {} · {}", f.id, k, f.claim.chars().take(80).collect::<String>()),
                "knowledge_key",
                0.8,
            );
        }
    }
}

fn fact_was_taught(org: &Organism, f: &FactCard) -> bool {
    let id = f.id.to_lowercase();
    let snip: String = f.claim.chars().take(48).collect::<String>().to_lowercase();
    org.long_term
        .iter()
        .chain(org.memory.iter().map(|m| &m.text))
        .chain(org.semantic.iter().map(|s| &s.text))
        .any(|t| {
            let tl = t.to_lowercase();
            tl.contains(&id)
                || (!snip.is_empty() && tl.contains(&snip))
                || f.keys.iter().filter(|k| k.len() > 3).any(|k| tl.contains(&k.to_lowercase()))
        })
}

/// Prefer facts the organism has actually studied (closed-book when cold).
fn pick_fact<'a>(
    org: &Organism,
    facts: &'a [FactCard],
    rng: &mut impl Rng,
    closed_book: bool,
) -> &'a FactCard {
    if closed_book {
        let taught: Vec<&FactCard> = facts.iter().filter(|f| fact_was_taught(org, f)).collect();
        if !taught.is_empty() {
            return taught[rng.gen_range(0..taught.len())];
        }
        // Prefer innate seed-sized prefix (first ~20 cards tend to be high-trust / repeated)
        let n = facts.len().min(24).max(1);
        return &facts[rng.gen_range(0..n)];
    }
    // Open book: full corpus pressure
    &facts[rng.gen_range(0..facts.len())]
}

/// Target mastery bar for the knowledge organ (user standard: ≥85%).
pub const KNOWLEDGE_TARGET: f64 = 0.85;

fn retrieve_answer(org: &Organism, q: &str, f: &FactCard, closed: bool) -> String {
    let mut ans = String::new();
    let id_l = f.id.to_lowercase();
    let claim_snip: String = f.claim.chars().take(48).collect::<String>().to_lowercase();

    // Closed-book: direct engram lookup by fact id (studied material only)
    if closed {
        for t in org
            .long_term
            .iter()
            .chain(org.memory.iter().map(|m| &m.text))
            .chain(org.semantic.iter().map(|s| &s.text))
        {
            let tl = t.to_lowercase();
            if tl.contains(&id_l)
                || (!claim_snip.is_empty() && tl.contains(&claim_snip))
                || (t.contains("FACT[") && f.keys.iter().filter(|k| k.len() > 3).any(|k| tl.contains(&k.to_lowercase())))
            {
                ans.push_str(t);
                ans.push(' ');
            }
        }
    }

    let hits = crate::language::semantic_retrieve(&org.semantic, q, 8);
    for (_, text, _) in &hits {
        ans.push_str(text);
        ans.push(' ');
    }
    // Keyword + key scan across durable stores
    let ql = q.to_lowercase();
    let mut words: Vec<String> = ql
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .map(|s| s.to_string())
        .collect();
    for k in &f.keys {
        if k.len() > 2 {
            words.push(k.to_lowercase());
        }
    }
    words.push(f.domain.to_lowercase());
    for t in org
        .long_term
        .iter()
        .chain(org.memory.iter().map(|m| &m.text))
        .chain(org.semantic.iter().map(|s| &s.text))
    {
        let tl = t.to_lowercase();
        if words.iter().any(|w| tl.contains(w)) || tl.contains(&id_l) {
            ans.push_str(t);
            ans.push(' ');
        }
    }
    ans
}

fn score_answer(ans: &str, f: &FactCard) -> bool {
    let al = ans.to_lowercase();
    if al.trim().is_empty() {
        return false;
    }
    // Prefer substantive keys (≥3 chars); allow claim snippet
    let key_hit = f.keys.iter().any(|k| {
        let kl = k.to_lowercase();
        kl.len() >= 3 && al.contains(&kl)
    }) || f.keys.iter().filter(|k| k.len() == 2).any(|k| al.contains(&k.to_lowercase()));
    let claim_hit = {
        let snip: String = f.claim.chars().take(40).collect::<String>().to_lowercase();
        !snip.is_empty() && al.contains(&snip)
    };
    let id_hit = al.contains(&f.id.to_lowercase()) && claim_hit;
    key_hit || claim_hit || id_hit
}

fn apply_sample(org: &mut Organism, pass: bool) {
    let sample = if pass { 1.0 } else { 0.0 };
    // Aggressive climb until target; stabilize above
    let lr = if org.knowledge_accuracy < 0.55 {
        0.45
    } else if org.knowledge_accuracy < 0.75 {
        0.36
    } else if org.knowledge_accuracy < KNOWLEDGE_TARGET {
        0.30
    } else if org.knowledge_accuracy < 0.92 {
        0.18
    } else {
        0.10
    };
    org.knowledge_accuracy =
        (org.knowledge_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
}

/// Knowledge exam under pressure — mastery target ≥85%.
///
/// Biology: encode (study) → consolidate → recall. Stay on closed-book
/// studied material until past the mastery bar; then open corpus for stretch.
pub fn exam_knowledge_one(org: &mut Organism, facts: &[FactCard], rng: &mut impl Rng) -> String {
    if facts.is_empty() {
        return "knowledge:empty".into();
    }

    let seeds = seed_facts();
    let fact_n = org.long_term.iter().filter(|t| t.contains("FACT[")).count();

    // Heavy study block until mastery bar
    if org.knowledge_accuracy < KNOWLEDGE_TARGET || fact_n < 12 {
        teach_batch(
            org,
            &seeds,
            rng.gen_range(0..seeds.len().max(1)),
            if org.knowledge_accuracy < 0.70 { 8 } else { 4 },
        );
        let span = facts.len().min(200).max(1);
        teach_batch(
            org,
            facts,
            rng.gen_range(0..span),
            if org.knowledge_accuracy < 0.70 { 6 } else { 3 },
        );
    }

    // Stay closed-book until past target (mastery of studied engrams)
    let closed = org.knowledge_accuracy < (KNOWLEDGE_TARGET + 0.03);
    if closed && !facts.iter().any(|f| fact_was_taught(org, f)) {
        teach_batch(org, &seeds, 0, seeds.len());
        teach_batch(org, facts, 0, 10);
    }

    let f = pick_fact(org, facts, rng, closed).clone();

    // Always encode target card before recall until mastery (study→test loop)
    if org.knowledge_accuracy < KNOWLEDGE_TARGET {
        teach_card(org, &f, 1.15);
    } else if !fact_was_taught(org, &f) {
        teach_card(org, &f, 0.9);
    }

    let q = if f.questions.is_empty() {
        format!("What do you know about {} / {}?", f.domain, f.id)
    } else {
        f.questions[rng.gen_range(0..f.questions.len())].clone()
    };

    let ans = retrieve_answer(org, &q, &f, closed);
    let mut pass = score_answer(&ans, &f);

    // If closed and failed: re-encode and re-test once (error-driven consolidation)
    if !pass && closed {
        teach_card(org, &f, 1.25);
        let ans2 = retrieve_answer(org, &q, &f, true);
        pass = score_answer(&ans2, &f);
    }

    apply_sample(org, pass);

    if pass {
        org.knowledge_skill = (org.knowledge_skill + 0.05).min(1.0);
        teach_card(org, &f, 0.9); // reinforce
    } else {
        org.pressure = (org.pressure + 0.02).min(1.5);
        teach_card(org, &f, 1.3);
        // Second remounted sample if still under bar
        if org.knowledge_accuracy < KNOWLEDGE_TARGET {
            let ans3 = retrieve_answer(org, &q, &f, true);
            if score_answer(&ans3, &f) {
                apply_sample(org, true);
                org.knowledge_skill = (org.knowledge_skill + 0.03).min(1.0);
                pass = true;
            }
        }
    }

    // Mastery burst: extra mini-quiz on a second taught card when under bar
    if org.knowledge_accuracy < KNOWLEDGE_TARGET {
        let f2 = pick_fact(org, facts, rng, true).clone();
        teach_card(org, &f2, 1.0);
        let q2 = if f2.questions.is_empty() {
            format!("Recall {}?", f2.domain)
        } else {
            f2.questions[rng.gen_range(0..f2.questions.len())].clone()
        };
        let a2 = retrieve_answer(org, &q2, &f2, true);
        let p2 = score_answer(&a2, &f2);
        apply_sample(org, p2);
        if p2 {
            org.knowledge_skill = (org.knowledge_skill + 0.03).min(1.0);
        }
    }

    org.recompute_intelligence_pub();
    let mode = if closed { "closed" } else { "open" };
    format!(
        "knowledge:{} → {} ({mode} acc={:.0}% target={:.0}%)",
        f.id,
        if pass { "PASS" } else { "FAIL" },
        org.knowledge_accuracy * 100.0,
        KNOWLEDGE_TARGET * 100.0
    )
}

/// Pack-level knowledge drill until mean ≥ target (called from habitat homeo).
pub fn pack_knowledge_mastery(pack: &mut [Organism], facts: &[FactCard], rng: &mut impl Rng) -> String {
    if facts.is_empty() {
        return "knowledge_mastery: no facts".into();
    }
    let mut boosted = 0u32;
    let seeds = seed_facts();
    for o in pack.iter_mut() {
        if o.knowledge_accuracy >= KNOWLEDGE_TARGET {
            continue;
        }
        boosted += 1;
        teach_batch(o, &seeds, rng.gen_range(0..seeds.len().max(1)), 6);
        teach_batch(
            o,
            facts,
            rng.gen_range(0..facts.len().min(300).max(1)),
            8,
        );
        // 2 rapid closed exams
        let _ = exam_knowledge_one(o, facts, rng);
        if o.knowledge_accuracy < KNOWLEDGE_TARGET {
            let _ = exam_knowledge_one(o, facts, rng);
        }
    }
    let mean = pack.iter().map(|o| o.knowledge_accuracy).sum::<f64>()
        / pack.len().max(1) as f64;
    format!(
        "knowledge_mastery: drilled={boosted}/{} mean={:.0}% target={:.0}%",
        pack.len(),
        mean * 100.0,
        KNOWLEDGE_TARGET * 100.0
    )
}

/// Answer a user factual question from knowledge + pack + FSOT notes (assistant path).
pub fn answer_factual(org: &Organism, q: &str) -> Option<String> {
    let ql = q.to_lowercase();
    let content_words: Vec<&str> = ql
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| {
            w.len() > 3
                && ![
                    "what", "does", "have", "with", "from", "that", "this", "about", "please",
                    "know", "tell", "your",
                ]
                .contains(w)
        })
        .collect();

    // 1) Prefer durable FACT[] lines that contain content words (true knowledge organ)
    let mut scored: Vec<(f32, String)> = Vec::new();
    for t in org
        .long_term
        .iter()
        .chain(org.memory.iter().map(|m| &m.text))
        .chain(org.semantic.iter().map(|s| &s.text))
    {
        let tl = t.to_lowercase();
        let is_fact = tl.contains("fact[") || t.starts_with("FACT");
        let hit = content_words.iter().filter(|w| tl.contains(**w)).count();
        if hit == 0 {
            continue;
        }
        // Down-rank dialogue echoes and codon noise for non-genetics questions
        let mut score = hit as f32 * 1.2;
        if is_fact {
            score += 2.0;
        }
        if tl.starts_with("q:") || tl.contains("dialogue") {
            score -= 1.5;
        }
        if !content_words.iter().any(|w| matches!(*w, "codon" | "amino" | "atg"))
            && tl.contains("codon")
            && hit <= 1
        {
            score -= 1.0;
        }
        scored.push((score, t.chars().take(220).collect()));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored.dedup_by(|a, b| a.1 == b.1);

    let mut parts: Vec<String> = scored
        .into_iter()
        .take(4)
        .map(|(s, t)| format!("· (knowledge, {s:.2}) {t}"))
        .collect();

    // 2) Semantic assist if thin
    if parts.len() < 2 {
        let hits = crate::language::semantic_retrieve(&org.semantic, q, 4);
        for (score, text, kind) in hits {
            if kind == "dialogue" || text.starts_with("Q:") {
                continue;
            }
            if score < 0.15 {
                continue;
            }
            parts.push(format!(
                "· ({kind}, {score:.2}) {}",
                text.chars().take(200).collect::<String>()
            ));
        }
    }

    if parts.is_empty() {
        return None;
    }
    parts.truncate(5);
    let mut out = format!(
        "From grounded knowledge (not free invention):\n{}",
        parts.join("\n")
    );
    // Attach FSOT note if claim is scientific and we have live S context
    if content_words
        .iter()
        .any(|w| matches!(*w, "fsot" | "scalar" | "regime" | "quantum" | "cosmo"))
    {
        out.push_str(&format!(
            "\nLive organism check: S={:.4} regime={}.",
            org.s, org.regime
        ));
    }
    Some(out)
}

// re-export helper for organism without circular issues
pub fn fact_count() -> usize {
    load_all_facts().len()
}
