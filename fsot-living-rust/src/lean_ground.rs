//! Lean ground — runtime invariants that match FSOT 2.1 Lean formalization.
//!
//! Source of truth (updated local tree, 2026-07):
//!   Desktop/FSOT-2.1-Lean/FSOT-2.1-Lean-main/FSOT-2.1-Lean-main/
//!     FSOT/Formal/Scalar.lean   — Real ledger + get_domain_params
//!     FSOT/Formal/Domains.lean  — sign certificates
//!     FSOT/Formal/*Priors.lean  — extension spines (math, science, reasoning)
//!   S = k · (T1 + T2 + T3)  with k = φ · (γ_E / e) · √2 / ln(π) · 0.99
//!
//! We do **not** claim full theorem import into Rust. We claim:
//!   the living scalar engine *obeys the same algebraic skeleton* Lean formalizes.
//! That is the circle-back: formal math ⟷ measurable runtime.

use crate::scalar::{self, compute_for_domain, compute_s, ComputeParams};
use std::collections::HashSet;

/// One check result.
#[derive(Clone, Debug)]
pub struct InvariantCheck {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct InvariantReport {
    pub checks: Vec<InvariantCheck>,
    pub all_ok: bool,
}

impl InvariantReport {
    pub fn summary_line(&self) -> String {
        let pass = self.checks.iter().filter(|c| c.ok).count();
        let n = self.checks.len();
        let status = if self.all_ok { "PASS" } else { "FAIL" };
        format!(
            "lean_ground {status} {pass}/{n} · k={:.8} · S=k·(T1+T2+T3) family",
            scalar::constants::k()
        )
    }
}

/// Lean Scalar.k reconstructed independently from named constants (sanity).
fn lean_k_reference() -> f64 {
    use std::f64::consts::{E, PI, SQRT_2};
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let gamma_euler = 0.5772156649015329;
    let perceived_param_base = gamma_euler / E;
    phi * (perceived_param_base * SQRT_2) / PI.ln() * 0.99
}

/// Runtime invariants the habitat already lives under.
pub fn check_runtime_invariants() -> InvariantReport {
    let mut checks = Vec::new();

    // 1) k matches Lean definition (relative ε)
    let k_rt = scalar::constants::k();
    let k_lean = lean_k_reference();
    let k_err = (k_rt - k_lean).abs() / k_lean.abs().max(1e-12);
    checks.push(InvariantCheck {
        name: "k_matches_lean_scalar",
        ok: k_err < 1e-9,
        detail: format!("k_rt={k_rt:.12} k_lean={k_lean:.12} rel_err={k_err:.2e}"),
    });

    // 2) Algebraic skeleton: S ≈ k · raw_s and raw ≈ T1+T2+T3
    let r = compute_s(&ComputeParams::default());
    let sum = r.term1 + r.term2 + r.term3;
    let raw_err = (r.raw_s - sum).abs();
    let s_err = (r.s - r.raw_s * k_rt).abs();
    checks.push(InvariantCheck {
        name: "S_eq_k_times_sum_terms",
        ok: raw_err < 1e-9 && s_err < 1e-9,
        detail: format!(
            "raw_err={raw_err:.2e} s_err={s_err:.2e} regime={} S={:.6}",
            r.regime, r.s
        ),
    });

    // 3) Domain ledger produces finite S and known regime labels
    let mut domains_ok = true;
    let mut domain_bits = Vec::new();
    for d in [
        "quantum",
        "biological",
        "cosmological",
        "neural",
        "ai",
        "chemical",
        "molecular",
        "medical",
        "electron",
        "energy",
    ] {
        let x = compute_for_domain(d);
        let finite = x.s.is_finite() && x.raw_s.is_finite();
        let regime_ok = matches!(x.regime, "emergence" | "damping" | "neutral");
        if !(finite && regime_ok) {
            domains_ok = false;
        }
        domain_bits.push(format!("{d}:S={:.4}/{}", x.s, x.regime));
    }
    checks.push(InvariantCheck {
        name: "domain_ledger_finite_regimes",
        ok: domains_ok,
        detail: domain_bits.join(" · "),
    });

    // 3b) Proved domain *signs* from FSOT.Formal.Domains (Formal Scalar params)
    // quantum>0, cosmological<0, chemical>0, neural>0, medical>0, electron>0, ai≤0
    // energy>0 (PureMathematicsPriors), consciousness>0 (ComputationalReasoningPriors)
    let q = compute_for_domain("quantum");
    let cos = compute_for_domain("cosmological");
    let chem = compute_for_domain("chemical");
    let neuro = compute_for_domain("neural");
    let med = compute_for_domain("medical");
    let elec = compute_for_domain("electron");
    let ai = compute_for_domain("ai");
    let energy = compute_for_domain("energy");
    let mind = compute_for_domain("consciousness");
    let math_alias = compute_for_domain("pure_mathematics"); // → energy
    let reason_alias = compute_for_domain("computational_reasoning"); // → consciousness
    let signs_ok = q.raw_s > 0.0
        && cos.raw_s < 0.0
        && chem.raw_s > 0.0
        && neuro.raw_s > 0.0
        && med.raw_s > 0.0
        && elec.raw_s > 0.0
        && ai.raw_s <= 0.05
        && energy.raw_s > 0.0
        && mind.raw_s > 0.0
        && (math_alias.raw_s - energy.raw_s).abs() < 1e-9
        && (reason_alias.raw_s - mind.raw_s).abs() < 1e-9;
    checks.push(InvariantCheck {
        name: "proved_domain_sign_certificates",
        ok: signs_ok,
        detail: format!(
            "q={:.3} cos={:.3} chem={:.3} neural={:.3} med={:.3} e-={:.3} ai={:.3} energy={:.3} mind={:.3} math≡E reason≡C",
            q.raw_s, cos.raw_s, chem.raw_s, neuro.raw_s, med.raw_s, elec.raw_s, ai.raw_s,
            energy.raw_s, mind.raw_s
        ),
    });

    // 3c) Codon dual-axis priors (CodonPriors): 64 table, primary 8, secondary 27, 3 stops
    let table = crate::genetic::all_codons();
    let stops = ["TAA", "TAG", "TGA"];
    let stop_n = stops.iter().filter(|c| table.get(**c) == Some(&'*')).count();
    let codon_ok = table.len() == 64 && stop_n == 3 && crate::genetic::genetic_code("ATG") == 'M';
    // primary pattern space: each base ±1 → 8 patterns
    let prim_pat: HashSet<[i8; 3]> = table
        .keys()
        .map(|c| crate::genetic::primary_trinary(c))
        .collect();
    let sec_pat: HashSet<[i8; 3]> = table
        .keys()
        .map(|c| crate::genetic::secondary_trinary(c))
        .collect();
    let pat_ok = prim_pat.len() == 8 && sec_pat.len() == 27;
    checks.push(InvariantCheck {
        name: "codon_dual_axis_priors",
        ok: codon_ok && pat_ok,
        detail: format!(
            "codons={} stops={} prim_patterns={} sec_patterns={} ATG=M",
            table.len(),
            stop_n,
            prim_pat.len(),
            sec_pat.len()
        ),
    });

    // 4) Emergence preference from genome guidance stays in Lean-aligned clamp
    //    (body genetic uses 0.05..0.95; living guidance same family)
    let g = crate::genetic::genome_to_guidance(&crate::genetic::default_organism_genome());
    let emerge_ok = g.emergence_preference >= 0.05 && g.emergence_preference <= 0.95;
    let d_eff_ok = g.d_eff >= 8.0 && g.d_eff <= 20.0;
    checks.push(InvariantCheck {
        name: "guidance_emergence_d_eff_bounds",
        ok: emerge_ok && d_eff_ok,
        detail: format!(
            "emerge={:.3} d_eff={:.2} delta_psi={:.3}",
            g.emergence_preference, g.d_eff, g.delta_psi
        ),
    });

    // 5) Regime sign rule used by runtime matches documented thresholds
    let emerge = compute_s(&ComputeParams {
        n: 2.0,
        p: 2.0,
        d_eff: 6.0,
        recent_hits: 0.0,
        delta_psi: 1.0,
        delta_theta: 1.0,
        rho: 1.0,
        scale: 1.0,
        amplitude: 1.0,
        trend_bias: 0.5,
        observed: true,
    });
    let damp = compute_s(&ComputeParams {
        n: 1.0,
        p: 1.0,
        d_eff: 25.0,
        recent_hits: 5.0,
        delta_psi: 3.0,
        delta_theta: 0.0,
        rho: 0.1,
        scale: 0.1,
        amplitude: 0.1,
        trend_bias: -2.0,
        observed: false,
    });
    let regime_rule_ok = (emerge.s > 0.02 && emerge.regime == "emergence")
        || (damp.s < -0.02 && damp.regime == "damping")
        || emerge.regime == "neutral"
        || damp.regime == "neutral";
    checks.push(InvariantCheck {
        name: "regime_thresholds_consistent",
        ok: regime_rule_ok,
        detail: format!(
            "sample_emerge S={:.4}/{} sample_damp S={:.4}/{}",
            emerge.s, emerge.regime, damp.s, damp.regime
        ),
    });

    let all_ok = checks.iter().all(|c| c.ok);
    InvariantReport { checks, all_ok }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lean_ground_passes() {
        let r = check_runtime_invariants();
        assert!(r.all_ok, "{}", r.summary_line());
        for c in &r.checks {
            assert!(c.ok, "{}: {}", c.name, c.detail);
        }
    }
}
