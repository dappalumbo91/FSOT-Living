//! FSOT 2.1 scalar engine — port of FSOT.Formal.Scalar (Real ledger) + Float Scalar.lean.
//! Authoritative Formal domain map: Desktop/FSOT-2.1-Lean/.../FSOT/Formal/Scalar.lean
//! Extension priors (PureMathematics D≈18→energy, ComputationalReasoning→consciousness)
//! live as domain aliases; Lean remains reference, math is baked into this runtime.

#[derive(Clone, Debug)]
pub struct ScalarResult {
    pub s: f64,
    pub raw_s: f64,
    pub term1: f64,
    pub term2: f64,
    pub term3: f64,
    pub quirk_mod: f64,
    pub regime: &'static str,
    pub d_eff: f64,
    pub delta_psi: f64,
    pub observed: bool,
}

pub mod constants {
    pub const PHI: f64 = 1.618033988749895;
    pub const E: f64 = std::f64::consts::E;
    pub const PI: f64 = std::f64::consts::PI;
    pub const SQRT2: f64 = std::f64::consts::SQRT_2;
    pub const LOG2: f64 = std::f64::consts::LN_2;
    pub const GAMMA_EULER: f64 = 0.5772156649015329;
    pub const CATALAN_G: f64 = 0.9159655941772190;

    pub fn alpha() -> f64 {
        PI.ln() / (E * PHI.powi(13))
    }
    pub fn psi_con() -> f64 {
        (E - 1.0) / E
    }
    pub fn eta_eff() -> f64 {
        1.0 / (PI - 1.0)
    }
    pub fn beta() -> f64 {
        1.0 / (PI.powf(PI) + (E - 1.0)).exp()
    }
    pub fn gamma() -> f64 {
        -LOG2 / PHI
    }
    pub fn omega() -> f64 {
        (PI / E).sin() * SQRT2
    }
    pub fn theta_s() -> f64 {
        (psi_con() * eta_eff()).sin()
    }
    pub fn poof_factor() -> f64 {
        (-(PI.ln() / E) / (eta_eff() * PHI.ln())).exp()
    }
    pub fn acoustic_bleed() -> f64 {
        (PI / E).sin() * PHI / SQRT2
    }
    pub fn phase_variance() -> f64 {
        -(theta_s() + PI).cos()
    }
    pub fn coherence_efficiency() -> f64 {
        (1.0 - poof_factor() * theta_s().sin())
            * (1.0 + 0.01 * CATALAN_G / (PI * PHI))
    }
    pub fn bleed_in_factor() -> f64 {
        coherence_efficiency() * (1.0 - theta_s().sin() / PHI)
    }
    pub fn acoustic_inflow() -> f64 {
        acoustic_bleed() * (1.0 + theta_s().cos() / PHI)
    }
    pub fn suction_factor() -> f64 {
        poof_factor() * (-(theta_s() - PI).cos())
    }
    pub fn chaos_factor() -> f64 {
        gamma() / omega()
    }
    pub fn perceived_param_base() -> f64 {
        GAMMA_EULER / E
    }
    pub fn new_perceived_param() -> f64 {
        perceived_param_base() * SQRT2
    }
    pub fn consciousness_factor() -> f64 {
        coherence_efficiency() * new_perceived_param()
    }
    pub fn k() -> f64 {
        PHI * (perceived_param_base() * SQRT2) / PI.ln() * 0.99
    }
}

use constants::*;

fn growth_term(n: f64, recent_hits: f64) -> f64 {
    (alpha() * (1.0 - recent_hits / n) * GAMMA_EULER / PHI).exp()
}

fn quirk_mod(observed: bool, delta_psi: f64) -> f64 {
    if observed {
        (consciousness_factor() * phase_variance()).exp() * (delta_psi + phase_variance()).cos()
    } else {
        1.0
    }
}

fn perceived_adjust(d_eff: f64) -> f64 {
    1.0 + new_perceived_param() * (d_eff / 25.0).ln()
}

fn term3(n: f64, p: f64, d_eff: f64, delta_psi: f64, delta_theta: f64) -> f64 {
    beta()
        * delta_psi.cos()
        * (n * p / d_eff.sqrt())
        * (1.0 + chaos_factor() * (d_eff - 25.0) / 25.0)
        * (1.0
            + poof_factor() * (theta_s() + PI).cos()
            + suction_factor() * theta_s().sin())
        * (1.0
            + acoustic_bleed() * delta_theta.sin().powi(2) / PHI
            + acoustic_inflow() * delta_theta.cos().powi(2) / PHI)
        * (1.0 + bleed_in_factor() * phase_variance())
}

#[derive(Clone, Debug)]
pub struct ComputeParams {
    pub n: f64,
    pub p: f64,
    pub d_eff: f64,
    pub recent_hits: f64,
    pub delta_psi: f64,
    pub delta_theta: f64,
    pub rho: f64,
    pub scale: f64,
    pub amplitude: f64,
    pub trend_bias: f64,
    pub observed: bool,
}

impl Default for ComputeParams {
    fn default() -> Self {
        Self {
            n: 1.0,
            p: 1.0,
            d_eff: 25.0,
            recent_hits: 0.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            rho: 1.0,
            scale: 1.0,
            amplitude: 1.0,
            trend_bias: 0.0,
            observed: false,
        }
    }
}

pub fn compute_s(opts: &ComputeParams) -> ScalarResult {
    let gt = growth_term(opts.n, opts.recent_hits);
    let term1_base = (opts.n * opts.p / opts.d_eff.sqrt())
        * ((psi_con() + opts.delta_psi) / eta_eff()).cos()
        * (-alpha() * opts.recent_hits / opts.n + opts.rho + bleed_in_factor() * opts.delta_psi)
            .exp()
        * (1.0 + gt * coherence_efficiency());
    let pa = perceived_adjust(opts.d_eff);
    let qm = quirk_mod(opts.observed, opts.delta_psi);
    let t1 = term1_base * pa * qm;
    let t2 = opts.scale * opts.amplitude + opts.trend_bias;
    let t3 = term3(opts.n, opts.p, opts.d_eff, opts.delta_psi, opts.delta_theta);
    let raw = t1 + t2 + t3;
    let s = raw * k();
    let regime = if s > 0.02 {
        "emergence"
    } else if s < -0.02 {
        "damping"
    } else {
        "neutral"
    };
    ScalarResult {
        s,
        raw_s: raw,
        term1: t1,
        term2: t2,
        term3: t3,
        quirk_mod: qm,
        regime,
        d_eff: opts.d_eff,
        delta_psi: opts.delta_psi,
        observed: opts.observed,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DomainParams {
    pub d_eff: f64,
    pub recent_hits: f64,
    pub delta_psi: f64,
    pub delta_theta: f64,
    pub observed: bool,
}

/// Domain priors from **FSOT.Formal.Scalar.get_domain_params** (proved sign certificates).
/// Source tree (updated): `Desktop/FSOT-2.1-Lean/.../FSOT/Formal/Scalar.lean`
/// Extension spines map onto ledger domains (not free grades):
///   PureMathematicsPriors D_eff=18 · energy_raw_S_positive
///   ComputationalReasoningPriors D_eff=12 · consciousness_raw_S_positive
pub fn domain_params(name: &str) -> DomainParams {
    let key = name.to_lowercase();
    let key = match key.as_str() {
        "cosmo" | "cosmology" => "cosmological",
        "bio" | "biology" | "botany" | "zoology" | "ecology" => "biological",
        "neuro" | "neuroscience" | "psychology" => "neural",
        "ai_tech" | "intelligence" | "computer" | "cs" | "computing" => "ai",
        "chem" | "chemistry" | "organic" => "chemical",
        "astro" | "astronomy" | "astrophysics" => "astronomical",
        "info" | "information" | "reasoning" | "computational_reasoning" => "consciousness",
        // Pure mathematics / competition math → energy family (Formal PureMathematicsPriors)
        "math" | "mathematics" | "pure_mathematics" | "pure_math" | "algebra"
        | "geometry" | "calculus" | "arithmetic" => "energy",
        "physics" | "mechanics" => "electron",
        "quantum_physics" | "particle_physics" => "quantum",
        "medicine" | "medical_science" | "clinical" | "health" => "medical",
        "engineering" | "energy_sys" | "electrical" | "mechanical" => "energy",
        "nuclear_physics" | "nuclear_energy" => "nuclear",
        "fusion_physics" => "fusion",
        "materials" | "condensed_matter" => "material",
        "cosmology_science" | "dark_matter" => "cosmological",
        other => other,
    };
    match key {
        // —— Formal ledger (proved / oracle-aligned) ——
        "quantum" => DomainParams {
            d_eff: 6.0,
            recent_hits: 0.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: true,
        },
        "particle" => DomainParams {
            d_eff: 7.0,
            recent_hits: 0.0,
            delta_psi: 0.85,
            delta_theta: 1.0,
            observed: true,
        },
        "higgs" => DomainParams {
            d_eff: 7.0,
            recent_hits: 1.0,
            delta_psi: 0.95,
            delta_theta: 1.0,
            observed: true,
        },
        "proton" => DomainParams {
            d_eff: 8.0,
            recent_hits: 0.0,
            delta_psi: 0.7,
            delta_theta: 1.0,
            observed: true,
        },
        "electron" => DomainParams {
            d_eff: 8.0,
            recent_hits: 0.0,
            delta_psi: 0.6,
            delta_theta: 1.0,
            observed: true,
        },
        // chemical_raw_S_positive (D=9, observed)
        "chemical" => DomainParams {
            d_eff: 9.0,
            recent_hits: 0.0,
            delta_psi: 0.5,
            delta_theta: 1.0,
            observed: true,
        },
        "molecular" => DomainParams {
            d_eff: 9.0,
            recent_hits: 0.0,
            delta_psi: 0.4,
            delta_theta: 1.0,
            observed: true,
        },
        "material" => DomainParams {
            d_eff: 10.0,
            recent_hits: 0.0,
            delta_psi: 0.5,
            delta_theta: 1.0,
            observed: true,
        },
        "biological" => DomainParams {
            d_eff: 12.0,
            recent_hits: 0.0,
            delta_psi: 0.08,
            delta_theta: 1.0,
            observed: false,
        },
        "cellular" => DomainParams {
            d_eff: 12.0,
            recent_hits: 0.0,
            delta_psi: 0.08,
            delta_theta: 1.0,
            observed: false,
        },
        // medical_raw_S_positive
        "medical" => DomainParams {
            d_eff: 13.0,
            recent_hits: 1.0,
            delta_psi: 0.35,
            delta_theta: 1.0,
            observed: true,
        },
        // neural_raw_S_positive
        "neural" => DomainParams {
            d_eff: 14.0,
            recent_hits: 1.0,
            delta_psi: 0.70,
            delta_theta: 1.0,
            observed: true,
        },
        "energy" => DomainParams {
            d_eff: 15.0,
            recent_hits: 1.0,
            delta_psi: 0.9,
            delta_theta: 1.0,
            observed: true,
        },
        "nuclear" => DomainParams {
            d_eff: 15.0,
            recent_hits: 1.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: true,
        },
        "fusion" => DomainParams {
            d_eff: 16.0,
            recent_hits: 1.0,
            delta_psi: 0.95,
            delta_theta: 1.0,
            observed: true,
        },
        // ai_raw_S_non_positive (D=11, unobserved)
        "ai" => DomainParams {
            d_eff: 11.0,
            recent_hits: 0.0,
            delta_psi: 0.50,
            delta_theta: 1.0,
            observed: false,
        },
        "consciousness" => DomainParams {
            d_eff: 16.0,
            recent_hits: 1.0,
            delta_psi: 1.15,
            delta_theta: 1.0,
            observed: true,
        },
        "perceived" => DomainParams {
            d_eff: 12.0,
            recent_hits: 0.0,
            delta_psi: 0.300302,
            delta_theta: 1.0,
            observed: true,
        },
        "neural_net" => DomainParams {
            d_eff: 12.0,
            recent_hits: 1.0,
            delta_psi: 0.8,
            delta_theta: 1.0,
            observed: true,
        },
        "observer" => DomainParams {
            d_eff: 14.0,
            recent_hits: 1.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: true,
        },
        "astronomical" => DomainParams {
            d_eff: 20.0,
            recent_hits: 1.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: true,
        },
        // cosmological_raw_S_negative
        "cosmological" => DomainParams {
            d_eff: 25.0,
            recent_hits: 0.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: false,
        },
        "galactic" => DomainParams {
            d_eff: 21.0,
            recent_hits: 1.0,
            delta_psi: 0.9,
            delta_theta: 1.0,
            observed: true,
        },
        "blackhole" => DomainParams {
            d_eff: 23.0,
            recent_hits: 2.0,
            delta_psi: 1.25,
            delta_theta: 1.0,
            observed: true,
        },
        "cmb" => DomainParams {
            d_eff: 24.0,
            recent_hits: 0.0,
            delta_psi: 0.8,
            delta_theta: 1.0,
            observed: false,
        },
        "dark_energy" => DomainParams {
            d_eff: 25.0,
            recent_hits: 0.0,
            delta_psi: 1.1,
            delta_theta: 1.0,
            observed: false,
        },
        // social / earth used by living suites
        "social" => DomainParams {
            d_eff: 21.0,
            recent_hits: 1.0,
            delta_psi: 0.8,
            delta_theta: 1.0,
            observed: true,
        },
        _ => DomainParams {
            d_eff: 25.0,
            recent_hits: 0.0,
            delta_psi: 1.0,
            delta_theta: 1.0,
            observed: false,
        },
    }
}

pub fn compute_for_domain(name: &str) -> ScalarResult {
    let d = domain_params(name);
    compute_s(&ComputeParams {
        d_eff: d.d_eff,
        recent_hits: d.recent_hits,
        delta_psi: d.delta_psi,
        delta_theta: d.delta_theta,
        observed: d.observed,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantum_positive() {
        let r = compute_for_domain("quantum");
        assert!(r.s > 0.0, "quantum S={}", r.s);
    }

    #[test]
    fn cosmological_negative() {
        let r = compute_for_domain("cosmological");
        assert!(r.s < 0.0, "cosmo S={}", r.s);
    }
}
