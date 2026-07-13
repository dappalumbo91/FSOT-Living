//! Pack communication, shared verified library, collective consistency exams
//!
//! Horizontal culture (signals) + group information farming under FSOT pressure.

use crate::fsot_tool;
use crate::genetic::genetic_code;
use crate::homeostasis::PerceivedTime;
use crate::organism::Organism;
use rand::Rng;
use serde::Serialize;

/// Chemical-analogue broadcast from one organism.
#[derive(Clone, Debug, Serialize)]
pub struct PackSignal {
    pub from_id: String,
    pub s: f64,
    pub regime: String,
    pub confidence: f64,
    pub fact_hint: String,
    pub fact_kind: String,
    pub truth_score: f64,
}

/// Verified shared knowledge entry (pack library).
#[derive(Clone, Debug, Serialize)]
pub struct LibraryEntry {
    pub text: String,
    pub kind: String,
    pub trust: f64,
    pub sources: Vec<String>,
    pub verified: bool,
    pub tick: u64,
}

#[derive(Clone, Debug, Default)]
pub struct SharedLibrary {
    pub entries: Vec<LibraryEntry>,
    pub publishes: u64,
    pub rejects: u64,
    pub collective_pass: u64,
    pub collective_fail: u64,
}

impl SharedLibrary {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn publish(
        &mut self,
        text: impl Into<String>,
        kind: impl Into<String>,
        source: impl Into<String>,
        trust: f64,
        verified: bool,
        tick: u64,
    ) {
        let text = text.into();
        let kind = kind.into();
        let source = source.into();
        // merge near-duplicates
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| e.text == text || (e.kind == kind && e.text.contains(&text.chars().take(40).collect::<String>())))
        {
            e.trust = (e.trust + trust * 0.5).min(3.0);
            if !e.sources.contains(&source) {
                e.sources.push(source);
            }
            e.verified = e.verified || verified;
            e.tick = tick;
            self.publishes += 1;
            return;
        }
        if !verified && trust < 0.55 {
            self.rejects += 1;
            return;
        }
        self.entries.push(LibraryEntry {
            text: text.chars().take(220).collect(),
            kind,
            trust,
            sources: vec![source],
            verified,
            tick,
        });
        self.publishes += 1;
        if self.entries.len() > 160 {
            self.entries
                .sort_by(|a, b| b.trust.partial_cmp(&a.trust).unwrap());
            self.entries.truncate(160);
        }
    }

    pub fn query(&self, topic: &str, k: usize) -> Vec<&LibraryEntry> {
        let tl = topic.to_lowercase();
        let mut scored: Vec<(f64, &LibraryEntry)> = self
            .entries
            .iter()
            .map(|e| {
                let el = e.text.to_lowercase();
                let mut sc = e.trust;
                if e.verified {
                    sc += 0.5;
                }
                for w in tl.split_whitespace() {
                    if w.len() > 2 && el.contains(w) {
                        sc += 0.35;
                    }
                }
                (sc, e)
            })
            .filter(|(s, _)| *s > 0.4)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(k).map(|(_, e)| e).collect()
    }

    pub fn summary_text(&self, limit: usize) -> String {
        let mut lines = Vec::new();
        let mut v: Vec<_> = self.entries.iter().collect();
        v.sort_by(|a, b| b.trust.partial_cmp(&a.trust).unwrap());
        for e in v.into_iter().take(limit) {
            lines.push(format!(
                "[{}{} trust={:.2} src={}] {}",
                e.kind,
                if e.verified { "✓" } else { "?" },
                e.trust,
                e.sources.len(),
                e.text.chars().take(100).collect::<String>()
            ));
        }
        if lines.is_empty() {
            "(pack library empty)".into()
        } else {
            lines.join("\n")
        }
    }
}

/// Emit signal from organism (chemical analogue).
pub fn emit_signal(o: &Organism) -> PackSignal {
    let (hint, kind, truth) = best_fact(o);
    let conf = (o.coherence * 0.4
        + o.language_accuracy * 0.2
        + o.cognitive_accuracy * 0.2
        + o.integration * 0.2)
        .clamp(0.05, 1.0);
    PackSignal {
        from_id: o.id.clone(),
        s: o.s,
        regime: o.regime.clone(),
        confidence: conf,
        fact_hint: hint,
        fact_kind: kind,
        truth_score: truth,
    }
}

fn best_fact(o: &Organism) -> (String, String, f64) {
    if let Some(c) = o.genome.first() {
        let aa = genetic_code(c);
        return (format!("Codon {c} → AA {aa}"), "codon".into(), 1.0);
    }
    if let Some(t) = o.long_term.last() {
        return (t.chars().take(120).collect(), "ltm".into(), 0.6);
    }
    (
        format!("live S={:.4} {}", o.s, o.regime),
        "state".into(),
        0.5,
    )
}

/// Couple pack via signals under homeostatic couple_gain.
/// Returns mean social alignment per organism (same order as pack).
pub fn couple_pack(
    pack: &mut [Organism],
    signals: &[PackSignal],
    pt: &PerceivedTime,
    library: &mut SharedLibrary,
    tick: u64,
) -> Vec<f64> {
    let n = pack.len().max(1) as f64;
    let mut alignments = vec![0.0; pack.len()];

    for i in 0..pack.len() {
        let me_s = pack[i].s;
        let me_reg = pack[i].regime.clone();
        let me_id = pack[i].id.clone();
        let mut align = 0.0;
        let mut wsum = 0.0;
        let mut received = 0u32;

        for sig in signals {
            if sig.from_id == me_id {
                continue;
            }
            let reg_match = if sig.regime == me_reg { 1.0 } else { 0.35 };
            let s_sim = 1.0 - (sig.s - me_s).abs().min(1.0);
            let w = sig.confidence * pt.couple_gain * (0.5 + 0.5 * s_sim) * reg_match;
            align += w * s_sim * reg_match;
            wsum += w;
            received += 1;

            // High-truth signals → candidate library publish
            if sig.truth_score >= 0.9 && sig.confidence > 0.45 {
                library.publish(
                    sig.fact_hint.clone(),
                    sig.fact_kind.clone(),
                    sig.from_id.clone(),
                    sig.confidence * sig.truth_score,
                    true,
                    tick,
                );
            }
        }

        let a = if wsum > 1e-6 { align / wsum } else { 0.3 };
        alignments[i] = a;
        pack[i].social_alignment = a;
        pack[i].signals_received = pack[i].signals_received.saturating_add(received);
        pack[i].last_signal_summary = format!(
            "aligned={:.2} peers={} couple_gain={:.2}",
            a,
            received,
            pt.couple_gain
        );

        // Homeostatic social pull on reward/pressure
        pack[i].reward = (pack[i].reward * 0.95 + 0.3 + a * 0.15 * pt.couple_gain).clamp(0.1, 0.92);
        if a < 0.25 {
            pack[i].pressure = (pack[i].pressure + 0.015 * pt.exam_gain).min(1.5);
        }

        // Soft teach from best peer fact
        if let Some(best) = signals
            .iter()
            .filter(|s| s.from_id != me_id && s.truth_score > 0.85)
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
        {
            if a > 0.4 {
                pack[i].teach(
                    &format!("PEER[{}]: {}", best.from_id, best.fact_hint),
                    "peer",
                    0.45 * pt.teach_gain * best.confidence,
                );
            }
        }

        // Social skill tracks successful alignment
        pack[i].social_skill = (pack[i].social_skill * 0.98 + a * 0.02).clamp(0.05, 1.0);
        let _ = n;
    }
    alignments
}

/// Collective consistency exam — pack farms composed knowledge.
/// Example: know ATG→M and domain quantum S; compose; verify with tools.
pub fn exam_collective(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    rng: &mut impl Rng,
    tick: u64,
) -> String {
    let kind = rng.gen_range(0..6u8);
    match kind {
        0 => exam_compose_arithmetic(pack, library, rng, tick),
        1 => exam_compose_codon(pack, library, rng, tick),
        2 => exam_compose_domain_dual(pack, library, tick),
        3 => exam_agree_regime(pack, library, tick),
        4 => exam_multihop_tool(pack, library, rng, tick),
        _ => exam_consistency_library(pack, library, tick),
    }
}

/// Multi-hop: compute domain via FSOT tool + arithmetic composition (tool-required).
fn exam_multihop_tool(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    rng: &mut impl Rng,
    tick: u64,
) -> String {
    let dom = if rng.gen_bool(0.5) {
        "quantum"
    } else {
        "cosmological"
    };
    let rep = fsot_tool::compute_domain(dom);
    let a = rng.gen_range(1..5i64);
    let code = format!("print({a}+{a})\n");
    let py = crate::code_env::run_python(&code, 3);
    let math_ok = py.ok && py.stdout.trim() == (a + a).to_string();
    let tool_ok = rep.ok && rep.text.contains("S=");
    let pass = math_ok && tool_ok;
    let text = format!(
        "MULTI-HOP: fsot_domain({dom}) S={:.4} {} AND {a}+{a}={} [tools required]",
        rep.s,
        rep.regime,
        a + a
    );
    if pass {
        library.publish(text.clone(), "multihop", "pack+tools", 1.4, true, tick);
        for o in pack.iter_mut() {
            o.teach(&text, "multihop", 0.95);
            o.composition_accuracy = o.composition_accuracy * 0.75 + 0.25;
            if !o.tools_owned.iter().any(|t| t == "fsot_domain") {
                o.tools_owned.push("fsot_domain".into());
            }
        }
    } else {
        for o in pack.iter_mut() {
            o.composition_accuracy = o.composition_accuracy * 0.8;
            o.pressure = (o.pressure + 0.02).min(1.5);
        }
    }
    credit(pack, pass, library);
    format!(
        "collective:multihop {dom}+arith → {}",
        if pass { "PASS" } else { "FAIL" }
    )
}

/// Consistency: library codon entries must match genetic truth.
fn exam_consistency_library(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    tick: u64,
) -> String {
    let mut checked = 0u32;
    let mut ok_n = 0u32;
    for e in library.entries.iter().filter(|e| e.kind == "codon" || e.text.contains("Codon")) {
        checked += 1;
        if let Some(rest) = e.text.split("Codon ").nth(1) {
            let codon: String = rest.chars().take(3).collect();
            if codon.len() == 3 {
                let truth = genetic_code(&codon);
                if e.text.contains(truth) || e.text.contains(&format!("AA {truth}")) {
                    ok_n += 1;
                }
            }
        }
        if checked >= 5 {
            break;
        }
    }
    let pass = checked == 0 || ok_n * 2 >= checked;
    if pass {
        library.publish(
            format!("CONSISTENCY: {ok_n}/{checked} codon library entries match genetic code"),
            "consistency",
            "pack",
            1.0,
            true,
            tick,
        );
    }
    credit(pack, pass, library);
    format!("collective:consistency {ok_n}/{checked} → {}", if pass { "PASS" } else { "FAIL" })
}

fn credit(pack: &mut [Organism], pass: bool, library: &mut SharedLibrary) {
    if pass {
        library.collective_pass += 1;
    } else {
        library.collective_fail += 1;
    }
    for o in pack.iter_mut() {
        let sample = if pass { 1.0 } else { 0.0 };
        // Fast recovery when collective is under-stimulated (birth at 0)
        let lr = if o.collective_accuracy < 0.35 {
            0.40
        } else if o.collective_accuracy < 0.55 {
            0.30
        } else {
            0.22
        };
        o.collective_accuracy =
            (o.collective_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
        if pass {
            o.social_skill = (o.social_skill + 0.025).min(1.0);
            // Composition rides on collective multi-agent reasoning
            let clr = if o.composition_accuracy < 0.45 {
                0.28
            } else {
                0.14
            };
            o.composition_accuracy =
                (o.composition_accuracy * (1.0 - clr) + clr * sample).clamp(0.0, 1.0);
        } else {
            o.pressure = (o.pressure + 0.015).min(1.5);
        }
    }
}

/// Bump composition organ with adaptive learning rate.
fn bump_composition(o: &mut Organism, pass: bool) {
    let sample = if pass { 1.0 } else { 0.0 };
    let lr = if o.composition_accuracy < 0.40 {
        0.36
    } else if o.composition_accuracy < 0.55 {
        0.26
    } else {
        0.16
    };
    o.composition_accuracy =
        (o.composition_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);
}

/// 1+1=2 and 2+2=4 compositional reasoning via Python organ of first capable org.
fn exam_compose_arithmetic(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    rng: &mut impl Rng,
    tick: u64,
) -> String {
    let a = rng.gen_range(1..6i64);
    // Fact1: a+a = 2a; Fact2: composition (a+a)+(a+a)=4a
    let step1 = a + a;
    let step2 = step1 + step1;
    let code = format!("print({a}+{a})\nprint({step1}+{step1})\n");
    let r = crate::code_env::run_python(&code, 4);
    let out = r.stdout.replace('\r', "");
    let lines: Vec<_> = out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let pass = r.ok
        && lines.len() >= 2
        && lines[0] == step1.to_string()
        && lines[1] == step2.to_string();

    let text = format!(
        "COLLECTIVE arithmetic: {a}+{a}={step1}; ({a}+{a})+({a}+{a})={step2} ⇒ composition holds"
    );
    if pass {
        library.publish(text.clone(), "compose_math", "pack", 1.2, true, tick);
        // teach all
        for o in pack.iter_mut() {
            o.teach(&text, "compose", 0.95);
            o.code_accuracy = o.code_accuracy * 0.9 + 0.1;
            bump_composition(o, true);
        }
    } else {
        for o in pack.iter_mut() {
            bump_composition(o, false);
        }
    }
    credit(pack, pass, library);
    format!(
        "collective:arith {a}→{step1}→{step2} → {}",
        if pass { "PASS" } else { "FAIL" }
    )
}

fn exam_compose_codon(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    rng: &mut impl Rng,
    tick: u64,
) -> String {
    let codons = ["ATG", "TTT", "AAA", "TGG", "GAA"];
    let c1 = codons[rng.gen_range(0..codons.len())];
    let c2 = codons[rng.gen_range(0..codons.len())];
    let aa1 = genetic_code(c1);
    let aa2 = genetic_code(c2);
    // Pack "knows" if majority has correct table entries
    let mut votes_ok = 0;
    for o in pack.iter() {
        let ok1 = o.codon_table.get(c1).copied().unwrap_or('?') == aa1
            || genetic_code(c1) == aa1;
        let ok2 = o.codon_table.get(c2).copied().unwrap_or('?') == aa2;
        if ok1 && ok2 {
            votes_ok += 1;
        }
    }
    let pass = votes_ok >= (pack.len() + 1) / 2;
    let text = format!(
        "COLLECTIVE codon compose: {c1}→{aa1} and {c2}→{aa2}; peptide fragment {aa1}{aa2}"
    );
    if pass {
        library.publish(text.clone(), "compose_codon", "pack", 1.1, true, tick);
        for o in pack.iter_mut() {
            o.codon_table.insert(c1.into(), aa1);
            o.codon_table.insert(c2.into(), aa2);
            o.teach(&text, "compose", 0.9);
            bump_composition(o, true);
        }
    } else {
        for o in pack.iter_mut() {
            bump_composition(o, false);
        }
    }
    credit(pack, pass, library);
    format!(
        "collective:codon {c1}/{c2} votes={votes_ok}/{} → {}",
        pack.len(),
        if pass { "PASS" } else { "FAIL" }
    )
}

fn exam_compose_domain_dual(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    tick: u64,
) -> String {
    let report = fsot_tool::compose_sign_claim("quantum", "cosmological");
    let pass = report.ok && report.text.contains("COMPOSITION");
    if pass {
        library.publish(
            format!(
                "COLLECTIVE FSOT: quantum S={:.3} vs cosmological — {}",
                fsot_tool::compute_domain("quantum").s,
                report.regime
            ),
            "compose_domain",
            "fsot_tool",
            1.3,
            true,
            tick,
        );
        for o in pack.iter_mut() {
            o.teach(&report.text.chars().take(200).collect::<String>(), "fsot_tool", 0.9);
            // Ensure tool known
            if !o.tools_owned.iter().any(|t| t == "fsot_domain") {
                o.tools_owned.push("fsot_domain".into());
            }
            o.tool_skill = (o.tool_skill + 0.02).min(1.0);
            bump_composition(o, true);
        }
    } else {
        for o in pack.iter_mut() {
            bump_composition(o, false);
        }
    }
    credit(pack, pass, library);
    format!(
        "collective:fsot_dual → {}",
        if pass { "PASS" } else { "FAIL" }
    )
}

fn exam_agree_regime(
    pack: &mut [Organism],
    library: &mut SharedLibrary,
    tick: u64,
) -> String {
    // Majority regime vote — social consensus under FSOT live state
    let mut counts = std::collections::HashMap::new();
    for o in pack.iter() {
        *counts.entry(o.regime.clone()).or_insert(0) += 1;
    }
    let (maj, n) = counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .unwrap_or(("neutral".into(), 0));
    let pass = n >= 2;
    let text = format!("COLLECTIVE regime consensus: {maj} ({n}/{} orgs)", pack.len());
    if pass {
        library.publish(text.clone(), "consensus", "pack", 0.8, true, tick);
    }
    credit(pack, pass, library);
    format!(
        "collective:regime {maj} → {}",
        if pass { "PASS" } else { "FAIL" }
    )
}
