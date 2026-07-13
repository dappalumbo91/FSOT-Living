//! Dream state — FSOT self-organization during SLEEP (not waking DEFRAG).
//!
//! ## What this is
//!
//! Biological sleep consolidates memory by **replaying and re-wiring** traces
//! into patterns that already make sense to the organism. Here, "makes sense"
//! means **FSOT connective structure**:
//!
//! - Domain affinity (quantum / bio / cosmo / code / self …)
//! - Emergence vs damping sign (same-sign clusters cohere; bridges link opposites carefully)
//! - Coupling to live genome guidance (δψ, d_eff, emergence preference)
//! - Trinary expression: strong-express traces weight more in the dream weave
//!
//! ## What this is not
//!
//! - Not `MOTOR DEFRAG` (waking housekeeping / core-preserving compact)
//! - Not random shuffle
//! - Not forced every append — only during **SLEEP** dream phase
//!
//! ## Flow
//!
//! ```text
//! SLEEP → plastic rest → DREAM (FSOT weave) → LTM snapshot
//! SAVE  → checkpoint only (no dream)
//! DEFRAG → explicit waking reorganize (core-safe)
//! ```

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use genetic::GenomeGuidance;
use trinary::Trit;

use crate::organs::{Membrane, OrganId};

/// One memory trace scored for dream connectivity.
#[derive(Clone, Debug)]
struct Trace {
    raw: String,
    domain: DomainKind,
    /// Soft S-sign: +1 emergence-leaning, −1 damping-leaning, 0 neutral.
    sign: i8,
    weight: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum DomainKind {
    SelfId,
    Genome,
    Quantum,
    Bio,
    Cosmo,
    Code,
    Knowledge,
    Social,
    Other,
}

impl DomainKind {
    fn label(self) -> &'static str {
        match self {
            DomainKind::SelfId => "self",
            DomainKind::Genome => "genome",
            DomainKind::Quantum => "quantum",
            DomainKind::Bio => "bio",
            DomainKind::Cosmo => "cosmo",
            DomainKind::Code => "code",
            DomainKind::Knowledge => "knowledge",
            DomainKind::Social => "social",
            DomainKind::Other => "other",
        }
    }
}

fn classify_line(line: &str) -> (DomainKind, i8) {
    let l = line.to_ascii_lowercase();
    let domain = if l.contains("core:") || l.contains("identity") || l.contains("companion") {
        DomainKind::SelfId
    } else if l.contains("genome") || l.contains("codon") || l.contains("atg") || l.contains("aa=")
    {
        DomainKind::Genome
    } else if l.contains("quantum") || l.contains("planck") || l.contains("qubit") {
        DomainKind::Quantum
    } else if l.contains("cosmo") || l.contains("hubble") || l.contains("lambda") {
        DomainKind::Cosmo
    } else if l.contains("bio") || l.contains("protein") || l.contains("cell") || l.contains("gene")
    {
        DomainKind::Bio
    } else if l.contains("code")
        || l.contains("python")
        || l.contains("forge")
        || l.contains("tool")
        || l.contains("js_")
    {
        DomainKind::Code
    } else if l.contains("pack") || l.contains("social") || l.contains("collective") {
        DomainKind::Social
    } else if l.contains("fact") || l.contains("knowledge") || l.contains("session:") {
        DomainKind::Knowledge
    } else {
        DomainKind::Other
    };
    // Emergence-leaning (+) vs damping-leaning (−) soft tags
    let sign = if l.contains("emergence")
        || l.contains("growth")
        || l.contains("express")
        || l.contains("breed")
    {
        1
    } else if l.contains("damping")
        || l.contains("repress")
        || l.contains("stable")
        || l.contains("conserv")
    {
        -1
    } else {
        match domain {
            DomainKind::Quantum | DomainKind::Bio | DomainKind::Code => 1,
            DomainKind::Cosmo => -1,
            DomainKind::SelfId | DomainKind::Genome => 0,
            _ => 0,
        }
    };
    (domain, sign)
}

fn connectivity(a: &Trace, b: &Trace, g: &GenomeGuidance) -> f64 {
    let mut s = 0.0;
    // Same domain → strong link
    if a.domain == b.domain {
        s += 1.0;
    }
    // Same FSOT sign (emergence/damping family) → cohere
    if a.sign != 0 && a.sign == b.sign {
        s += 0.7;
    }
    // Opposite signs → weak bridge only (tension, not merge)
    if a.sign != 0 && b.sign != 0 && a.sign != b.sign {
        s += 0.15;
    }
    // Guidance coupling: higher emergence_preference favors +sign clusters
    if g.emergence_preference > 0.55 && (a.sign > 0 || b.sign > 0) {
        s += 0.25 * g.emergence_preference;
    }
    if g.delta_psi < 0.4 && (a.sign < 0 || b.sign < 0) {
        s += 0.2;
    }
    // Domain coupling soft boost for genome↔bio
    match (a.domain, b.domain) {
        (DomainKind::Genome, DomainKind::Bio) | (DomainKind::Bio, DomainKind::Genome) => {
            s += 0.4 * g.domain_coupling;
        }
        (DomainKind::SelfId, _) | (_, DomainKind::SelfId) => s += 0.2,
        _ => {}
    }
    s * (0.5 + 0.5 * a.weight) * (0.5 + 0.5 * b.weight)
}

/// Harvest traces from home + working organs for the dream weave.
fn harvest(membrane: &Membrane, expr_boost: f64) -> Vec<Trace> {
    let mut out = Vec::new();
    let ids = [
        OrganId::HomeIdentity,
        OrganId::HomeKnowledge,
        OrganId::HomeGenome,
        OrganId::HomeTools,
        OrganId::HomeProposals,
        OrganId::Knowledge,
        OrganId::Notes,
    ];
    for id in ids {
        let node = membrane.get(id);
        let text = core::str::from_utf8(&node.content).unwrap_or("");
        let organ_w = match node.state {
            Trit::SpinUp => 1.0,
            Trit::Superposed => 0.55,
            Trit::SpinDown => 0.35,
        } * (0.6 + 0.4 * expr_boost);
        for line in text.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            // Never dream-away core identity lines as free float — keep weight high
            let (domain, sign) = classify_line(t);
            let w = if t.starts_with("core:") {
                1.5 * organ_w
            } else {
                organ_w
            };
            out.push(Trace {
                raw: String::from(t),
                domain,
                sign,
                weight: w,
            });
        }
    }
    out
}

/// Cluster traces by greedy FSOT connectivity (simple, no_std-friendly).
fn weave_clusters(traces: &[Trace], g: &GenomeGuidance) -> Vec<Vec<usize>> {
    let n = traces.len();
    if n == 0 {
        return Vec::new();
    }
    let mut used = vec![false; n];
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    // Seed order: highest weight first
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| {
        traces[j]
            .weight
            .partial_cmp(&traces[i].weight)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    // Fix Ordering if needed - use Equal
    for &seed in &order {
        if used[seed] {
            continue;
        }
        let mut cluster = vec![seed];
        used[seed] = true;
        // Attach strongly connected free traces
        loop {
            let mut best: Option<(usize, f64)> = None;
            for i in 0..n {
                if used[i] {
                    continue;
                }
                let mut score = 0.0;
                for &c in &cluster {
                    score += connectivity(&traces[c], &traces[i], g);
                }
                score /= cluster.len() as f64;
                if score >= 0.55 {
                    if best.map(|(_, s)| score > s).unwrap_or(true) {
                        best = Some((i, score));
                    }
                }
            }
            if let Some((i, _)) = best {
                used[i] = true;
                cluster.push(i);
                if cluster.len() >= 12 {
                    break;
                }
            } else {
                break;
            }
        }
        clusters.push(cluster);
    }
    // Prefer clusters that match genome emergence bias first
    clusters.sort_by(|a, b| {
        let sa: f64 = a.iter().map(|&i| traces[i].sign as f64 * traces[i].weight).sum();
        let sb: f64 = b.iter().map(|&i| traces[i].sign as f64 * traces[i].weight).sum();
        let sa = sa * if g.emergence_preference > 0.5 { 1.0 } else { -0.5 };
        let sb = sb * if g.emergence_preference > 0.5 { 1.0 } else { -0.5 };
        sb.partial_cmp(&sa).unwrap_or(core::cmp::Ordering::Equal)
    });
    clusters
}

/// Result of one dream cycle (for telemetry + journal).
#[derive(Clone, Debug)]
pub struct DreamReport {
    pub clusters: usize,
    pub traces: usize,
    pub bridges: usize,
    pub journal: String,
}

/// Run FSOT dream weave over membrane LTM/working traces.
/// Rewrites home/knowledge with clustered pattern (identity core preserved via separate organ).
pub fn run_dream(membrane: &mut Membrane, guidance: &GenomeGuidance, tick: u64) -> DreamReport {
    let expr = membrane.get(OrganId::HomeKnowledge).expression_strength();
    let traces = harvest(membrane, expr);
    let n = traces.len();
    if n == 0 {
        return DreamReport {
            clusters: 0,
            traces: 0,
            bridges: 0,
            journal: format!("dream: t={tick} empty"),
        };
    }
    let clusters = weave_clusters(&traces, guidance);
    let mut bridges = 0usize;
    let mut journal = format!(
        "dream: t={tick} traces={n} clusters={} delta_psi={:.3} d_eff={:.1} emerge={:.2}\n",
        clusters.len(),
        guidance.delta_psi,
        guidance.d_eff,
        guidance.emergence_preference
    );

    // Build woven knowledge body: cluster headers + members (FSOT connectivity order)
    let mut woven = String::new();
    woven.push_str("# FSOT dream weave — connective patterns (not random defrag)\n");
    for (ci, cluster) in clusters.iter().enumerate() {
        let mut dom_counts: Vec<(DomainKind, usize)> = Vec::new();
        let mut sign_sum = 0i32;
        for &i in cluster {
            let d = traces[i].domain;
            if let Some(e) = dom_counts.iter_mut().find(|(k, _)| *k == d) {
                e.1 += 1;
            } else {
                dom_counts.push((d, 1));
            }
            sign_sum += traces[i].sign as i32;
        }
        dom_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let top = dom_counts
            .first()
            .map(|(d, _)| d.label())
            .unwrap_or("other");
        let polarity = if sign_sum > 0 {
            "emergence"
        } else if sign_sum < 0 {
            "damping"
        } else {
            "neutral"
        };
        woven.push_str(&format!(
            "cluster:{ci} domain={top} polarity={polarity} n={}\n",
            cluster.len()
        ));
        journal.push_str(&format!(
            "  cluster:{ci} {top}/{polarity} n={}\n",
            cluster.len()
        ));
        for &i in cluster {
            woven.push_str(&traces[i].raw);
            woven.push('\n');
        }
        // Bridge note if cluster contains opposite signs
        let has_pos = cluster.iter().any(|&i| traces[i].sign > 0);
        let has_neg = cluster.iter().any(|&i| traces[i].sign < 0);
        if has_pos && has_neg {
            bridges += 1;
            woven.push_str(&format!(
                "bridge: cluster:{ci} couples emergence↔damping under δψ={:.3}\n",
                guidance.delta_psi
            ));
        }
    }
    journal.push_str(&format!("bridges={bridges}\n"));

    // Apply weave to home/knowledge (semantic LTM) — identity organ untouched here
    membrane
        .get_mut(OrganId::HomeKnowledge)
        .write_str(&woven);

    // Soft session note on identity: dream happened (append, not rewrite).
    // Richer narrative → continuous subjective thread across sleep cycles.
    membrane.get_mut(OrganId::HomeIdentity).append_str(&format!(
        "session: dream t={tick} clusters={} bridges={} traces={} δψ={:.3} d_eff={:.1} emerge={:.2} aa={}",
        clusters.len(),
        bridges,
        n,
        guidance.delta_psi,
        guidance.d_eff,
        guidance.emergence_preference,
        if guidance.aa_chain.len() > 18 {
            &guidance.aa_chain[..18]
        } else {
            guidance.aa_chain.as_str()
        }
    ));

    // Notes get dream journal summary
    membrane.get_mut(OrganId::Notes).write_str(&journal);

    DreamReport {
        clusters: clusters.len(),
        traces: n,
        bridges,
        journal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organs::Membrane;

    #[test]
    fn dream_clusters_knowledge() {
        let mut m = Membrane::new();
        m.get_mut(OrganId::HomeKnowledge)
            .write_str("fact quantum emergence growth\nfact cosmo damping stable\nfact bio gene protein\n");
        m.get_mut(OrganId::HomeIdentity)
            .write_str("core: companion self\nsession: gen=1\n");
        let g = GenomeGuidance {
            delta_psi: 0.7,
            d_eff: 12.0,
            emergence_preference: 0.65,
            domain_coupling: 0.8,
            aa_chain: String::from("MPNG"),
            codon_count: 4,
            mean_primary: 0.1,
            mean_secondary: 0.0,
        };
        let r = run_dream(&mut m, &g, 42);
        assert!(r.traces >= 3);
        assert!(r.clusters >= 1);
        let kn = m.get_mut(OrganId::HomeKnowledge).read_str();
        assert!(kn.contains("cluster:") || kn.contains("dream"));
    }
}
