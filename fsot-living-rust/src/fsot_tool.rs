//! FSOT framework as a first-class callable tool (verified domain ledger path)
//!
//! Organisms (and language organ) call into this instead of inventing S values.
//! Grounded in scalar.rs domain ledger — same engine as Lean-ported compute.

use crate::scalar::{compute_for_domain, domain_params, constants};

#[derive(Clone, Debug)]
pub struct FsotToolReport {
    pub ok: bool,
    pub domain: String,
    pub text: String,
    pub s: f64,
    pub regime: String,
}

/// List domains the tool can compute.
pub fn domains() -> &'static [&'static str] {
    &[
        "quantum",
        "biological",
        "cosmological",
        "neuroscience",
        "chemistry",
        "physics",
        "information",
        "ai",
    ]
}

/// Run FSOT domain compute tool.
pub fn compute_domain(name: &str) -> FsotToolReport {
    let key = normalize_domain(name);
    let r = compute_for_domain(&key);
    let d = domain_params(&key);
    FsotToolReport {
        ok: true,
        domain: key.clone(),
        s: r.s,
        regime: r.regime.to_string(),
        text: format!(
            "FSOT_TOOL domain={key}\n\
             S={:.6} raw_S={:.6} regime={}\n\
             D_eff={} δψ={} observed={}\n\
             T1={:.5} T2={:.5} T3={:.3e}\n\
             k={:.6} (FSOT 2.1 scalar engine)",
            r.s,
            r.raw_s,
            r.regime,
            d.d_eff,
            d.delta_psi,
            d.observed,
            r.term1,
            r.term2,
            r.term3,
            constants::k()
        ),
    }
}

/// Explain framework constants (no domain).
pub fn explain_framework() -> FsotToolReport {
    FsotToolReport {
        ok: true,
        domain: "framework".into(),
        s: 0.0,
        regime: "meta".into(),
        text: format!(
            "FSOT 2.1: S = k · (T1 + T2 + T3)\n\
             k={:.6}  φ={}  perceived_param_base uses γ_E/e\n\
             Domains: {}\n\
             Regimes: emergence (S>0.02), neutral, damping (S<-0.02)\n\
             Call: fsot domain <name> | use as tool under exam pressure",
            constants::k(),
            constants::PHI,
            domains().join(", ")
        ),
    }
}

/// Consistency check: if A and B domain signs, report compositional claim.
pub fn compose_sign_claim(dom_a: &str, dom_b: &str) -> FsotToolReport {
    let a = compute_domain(dom_a);
    let b = compute_domain(dom_b);
    let same_sign = (a.s > 0.02 && b.s > 0.02) || (a.s < -0.02 && b.s < -0.02);
    let claim = if same_sign {
        format!(
            "COMPOSITION: {0} and {1} share regime family (both {2}). Combined narrative: aligned polarity.",
            a.domain,
            b.domain,
            if a.s > 0.0 { "constructive/emergence-leaning" } else { "damping-leaning" }
        )
    } else {
        format!(
            "COMPOSITION: {0} (S={1:.3}) and {2} (S={3:.3}) differ in polarity — dual-aspect FSOT structure.",
            a.domain, a.s, b.domain, b.s
        )
    };
    FsotToolReport {
        ok: true,
        domain: format!("{}+{}", a.domain, b.domain),
        s: a.s + b.s,
        regime: if same_sign { "aligned" } else { "dual" }.into(),
        text: format!("{}\n---\n{}\n---\n{}", a.text, b.text, claim),
    }
}

pub fn normalize_domain(name: &str) -> String {
    let n = name.trim().to_lowercase();
    if n.contains("cosmo") {
        "cosmological".into()
    } else if n.contains("quantum") || n == "q" {
        "quantum".into()
    } else if n.contains("bio") {
        "biological".into()
    } else if n.contains("neuro") {
        "neuroscience".into()
    } else if n.contains("chem") {
        "chemistry".into()
    } else if n.contains("phys") {
        "physics".into()
    } else if n.contains("info") {
        "information".into()
    } else if n == "ai" || n.contains("artificial") {
        "ai".into()
    } else if domains().contains(&n.as_str()) {
        n
    } else {
        "biological".into()
    }
}

/// Exam: does organism report correct sign/regime for a domain via tool?
pub fn exam_domain_sign(domain: &str, answer: &str) -> bool {
    let r = compute_domain(domain);
    let al = answer.to_lowercase();
    let regime_ok = al.contains(&r.regime);
    let sign_ok = if r.s > 0.02 {
        al.contains("emergence") || al.contains("positive") || al.contains('+')
    } else if r.s < -0.02 {
        al.contains("damping") || al.contains("negative") || al.contains('-')
    } else {
        al.contains("neutral")
    };
    regime_ok || (sign_ok && al.contains('s'))
}
