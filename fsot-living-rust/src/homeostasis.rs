//! Homeostasis loops driven by FSOT perceived-time (sine/cosine phase family)
//!
//! Aligns with FSOT 2.1 Lean: perceived parameters, phase variance, and
//! oscillatory coupling of vitality / pressure / coherence over habitat time.

use crate::scalar::constants::{
    consciousness_factor, new_perceived_param, phase_variance, psi_con, theta_s, PHI, PI,
};

/// Global perceived-time state for the habitat at tick t.
#[derive(Clone, Debug)]
pub struct PerceivedTime {
    /// Dimensionless habitat time (tick scaled)
    pub t: f64,
    /// Primary phase (sin family)
    pub phase_sin: f64,
    /// Quadrature phase (cos family)
    pub phase_cos: f64,
    /// Slow envelope (generation-scale)
    pub envelope: f64,
    /// FSOT-modulated perceived rate
    pub perceived_rate: f64,
    /// Composite homeostatic drive in [-1, 1]
    pub drive: f64,
    /// Coupling gain for pack signals
    pub couple_gain: f64,
    /// Feed / teach strength modulator
    pub teach_gain: f64,
    /// Exam difficulty / pressure modulator
    pub exam_gain: f64,
}

/// Compute perceived time + homeostatic gains at habitat tick.
///
/// Uses sin/cos of multiple incommensurate periods (27, 91, φ·40, π·55)
/// mixed with FSOT constants so the loop is not a plain clock sine.
pub fn perceived_time(tick: u64, generation: u32) -> PerceivedTime {
    let t = tick as f64;
    let g = generation as f64;

    // Base angular rates (perceived time)
    let w1 = 2.0 * PI / 27.0;
    let w2 = 2.0 * PI / 91.0;
    let w3 = 2.0 * PI / (PHI * 40.0);
    let w4 = 2.0 * PI / (PI * 55.0);

    let s1 = (w1 * t).sin();
    let c1 = (w1 * t).cos();
    let s2 = (w2 * t).sin();
    let c2 = (w2 * t).cos();
    let s3 = (w3 * t + theta_s()).sin();
    let c3 = (w3 * t + phase_variance()).cos();
    let s4 = (w4 * t + psi_con()).sin();

    let phase_sin = (0.45 * s1 + 0.30 * s2 + 0.15 * s3 + 0.10 * s4).clamp(-1.0, 1.0);
    let phase_cos = (0.45 * c1 + 0.30 * c2 + 0.25 * c3).clamp(-1.0, 1.0);

    // Slow envelope across generations (cultural timescale)
    let envelope = (0.5 + 0.5 * (g / 12.0 + t / 800.0).sin()).clamp(0.15, 1.0);

    let perceived_rate = new_perceived_param() * (1.0 + 0.15 * phase_cos) * consciousness_factor().max(0.01);

    // Drive: constructive when sin/cos align toward coherence
    let drive = (0.55 * phase_sin + 0.35 * phase_cos * phase_variance().cos()
        + 0.10 * (t * w3).sin() * psi_con())
        .clamp(-1.0, 1.0);

    let couple_gain = (0.55 + 0.35 * phase_cos + 0.10 * envelope).clamp(0.2, 1.15);
    let teach_gain = (0.70 + 0.25 * phase_sin + 0.15 * envelope).clamp(0.4, 1.25);
    let exam_gain = (0.75 + 0.20 * (-phase_sin) + 0.10 * (1.0 - envelope)).clamp(0.5, 1.3);

    PerceivedTime {
        t,
        phase_sin,
        phase_cos,
        envelope,
        perceived_rate,
        drive,
        couple_gain,
        teach_gain,
        exam_gain,
    }
}

/// Apply homeostatic adjustment to organism scalars (call after compute).
pub fn apply_to_organism(
    coherence: &mut f64,
    pressure: &mut f64,
    reward: &mut f64,
    vitality: &mut f64,
    integration: &mut f64,
    pt: &PerceivedTime,
    social_alignment: f64,
) {
    // Coherence tracks cos-phase (stability) + social alignment
    let coh_target = 0.55 + 0.25 * pt.phase_cos + 0.15 * social_alignment;
    *coherence = (*coherence * 0.88 + coh_target * 0.12).clamp(0.2, 1.0);

    // Pressure rises on negative drive / high exam_gain, eases on positive drive.
    // High exam_gain = allostatic exam load (deeper "chew" cycles downstream).
    let p_pull = ((-pt.drive) * 0.08 + (pt.exam_gain - 1.0) * 0.07).clamp(-0.12, 0.15);
    *pressure = (*pressure + p_pull).clamp(0.0, 1.5);

    // Reward follows constructive drive under perceived time
    let r_pull = pt.drive * 0.04 * pt.perceived_rate.max(0.01);
    *reward = (*reward * 0.96 + 0.35 + r_pull).clamp(0.1, 0.92);

    *vitality = (*vitality + pt.drive * 0.03).clamp(-0.55, 1.2);
    *integration = (*integration * 0.95 + (0.4 + 0.2 * pt.envelope + 0.2 * social_alignment) * 0.05)
        .clamp(0.1, 1.0);
}

