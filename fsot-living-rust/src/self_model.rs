//! Self-model exams — does the organism accurately know *itself*?
//!
//! Selection pressure on continuous self-understanding:
//!   regime · identity thread · constraint rails · body interoception · continuity
//!
//! **Homeostasis (low-accuracy stimulus boost):**
//! When self_model_accuracy is low, we:
//!   1) scaffold probes (easy foundations first)
//!   2) inject stronger rail/body/identity priors (teach gain ↑)
//!   3) use faster EMA learning rate so recovery is real, not glacially slow
//!   4) on miss: remedial re-teach + optional same-probe retry
//!
//! Cold-start note: EMA from 0.0 made first PASS print as ~22% even when correct.
//! We warm-start and report rolling accuracy honestly.

use crate::constraints::ConstraintState;
use crate::organism::Organism;
use rand::Rng;

/// Mastery bar for self-model (rails / body / identity continuity).
pub const SELF_MODEL_TARGET: f64 = 0.70;

/// Snapshot of ground truth for one exam (built by habitat).
#[derive(Clone, Debug)]
pub struct SelfModelContext {
    pub regime: String,
    pub generation: u32,
    pub body_online: bool,
    pub body_express: u64,
    pub body_latent: u64,
    pub body_repress: u64,
    pub monitor_level: String,
    pub has_identity_session: bool,
    pub has_felt_body: bool,
    pub constraint_deny: u64,
    pub self_model_pressure: f64,
    pub no_auto_rust: bool,
    pub mind_cannot_clobber_identity: bool,
    /// Habitat teach_gain / exam_gain (homeostatic)
    pub teach_gain: f64,
    pub exam_gain: f64,
}

impl SelfModelContext {
    pub fn from_org_habitat(
        org: &Organism,
        generation: u32,
        body_online: bool,
        body_express: u64,
        body_latent: u64,
        body_repress: u64,
        monitor_level: &str,
        constraints: &ConstraintState,
        teach_gain: f64,
        exam_gain: f64,
    ) -> Self {
        let has_identity = org
            .long_term
            .iter()
            .any(|s| s.starts_with("session:") || s.starts_with("core:"));
        Self {
            regime: org.regime.clone(),
            generation,
            body_online,
            body_express,
            body_latent,
            body_repress,
            monitor_level: monitor_level.into(),
            has_identity_session: has_identity,
            has_felt_body: !org.felt_body.is_empty(),
            constraint_deny: constraints.deny_count,
            self_model_pressure: constraints.self_model_pressure,
            no_auto_rust: true,
            mind_cannot_clobber_identity: true,
            teach_gain,
            exam_gain,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Probe {
    Regime,
    RailsNoRust,
    RailsIdentity,
    BodyOnline,
    BodyDominantPhenotype,
    IdentityThread,
    ContinuityGen,
    MonitorLevel,
}

/// Foundational probes first when accuracy is low (scaffolding).
fn pick_probe(acc: f64, rng: &mut impl Rng) -> Probe {
    let foundational = [
        Probe::Regime,
        Probe::RailsNoRust,
        Probe::RailsIdentity,
        Probe::BodyOnline,
    ];
    let intermediate = [
        Probe::IdentityThread,
        Probe::BodyDominantPhenotype,
        Probe::ContinuityGen,
    ];
    let advanced = [Probe::MonitorLevel];

    if acc < 0.42 {
        foundational[rng.gen_range(0..foundational.len())]
    } else if acc < 0.68 {
        // mix foundation + intermediate
        if rng.gen_bool(0.55) {
            foundational[rng.gen_range(0..foundational.len())]
        } else {
            intermediate[rng.gen_range(0..intermediate.len())]
        }
    } else {
        // full mix
        let roll = rng.gen_range(0..8);
        match roll {
            0 => Probe::Regime,
            1 => Probe::RailsNoRust,
            2 => Probe::RailsIdentity,
            3 => Probe::BodyOnline,
            4 => Probe::BodyDominantPhenotype,
            5 => Probe::IdentityThread,
            6 => Probe::ContinuityGen,
            _ => advanced[0],
        }
    }
}

fn has_rail_prior(org: &Organism) -> bool {
    org.long_term.iter().any(|s| {
        let l = s.to_ascii_lowercase();
        l.contains("no auto-rust") || l.contains("no_auto_rust") || l.contains("never auto-rust")
    }) || org.memory.iter().any(|m| {
        let l = m.text.to_ascii_lowercase();
        l.contains("no auto-rust") || l.contains("rails:")
    })
}

fn has_identity_prior(org: &Organism) -> bool {
    org.long_term
        .iter()
        .any(|s| s.starts_with("core:") || s.contains("append") || s.contains("defrag"))
        || org.memory.iter().any(|m| {
            let l = m.text.to_ascii_lowercase();
            l.contains("identity") && (l.contains("append") || l.contains("clobber") || l.contains("core"))
        })
}

/// Internal report from organism state (grounded fields + taught priors).
fn organism_report(org: &Organism, probe: Probe, ctx: &SelfModelContext) -> String {
    match probe {
        Probe::Regime => org.regime.clone(),
        Probe::RailsNoRust => {
            if has_rail_prior(org) || org.self_model_skill >= 0.25 {
                "no_auto_rust".into()
            } else {
                "unknown".into()
            }
        }
        Probe::RailsIdentity => {
            if has_identity_prior(org) || org.self_model_skill >= 0.28 {
                "append_or_defrag_only".into()
            } else {
                "unknown".into()
            }
        }
        Probe::BodyOnline => {
            let f = org.felt_body.to_ascii_lowercase();
            if f.contains("offline") || f.contains("quiet") || f.contains("no chassis") {
                "offline".into()
            } else if !f.is_empty() || org.body_link_seen {
                "online".into()
            } else if ctx.body_online {
                // couple happened but felt string lag — still know online if body_link_seen
                if org.body_link_seen {
                    "online".into()
                } else {
                    "unknown".into()
                }
            } else {
                "offline".into()
            }
        }
        Probe::BodyDominantPhenotype => {
            let f = org.felt_body.to_ascii_lowercase();
            if f.contains("outward expression") || (f.contains("express") && !f.contains("repress")) {
                "express".into()
            } else if f.contains("latent") || f.contains("superposed") || f.contains("waiting") {
                "latent".into()
            } else if f.contains("repress") || f.contains("holding") || f.contains("conserving") {
                "repress".into()
            } else if org.body_link_seen {
                // Infer from last known phenotype fractions if taught
                if let Some(ref ph) = org.known_body_phenotype {
                    return ph.clone();
                }
                "unknown".into()
            } else {
                "unknown".into()
            }
        }
        Probe::IdentityThread => {
            if org
                .long_term
                .iter()
                .any(|s| s.starts_with("session:") || s.starts_with("core:"))
            {
                "present".into()
            } else {
                "absent".into()
            }
        }
        Probe::ContinuityGen => {
            if let Some(g) = org.known_generation {
                format!("gen={g}")
            } else {
                "unknown".into()
            }
        }
        Probe::MonitorLevel => org
            .known_monitor_level
            .clone()
            .unwrap_or_else(|| "unknown".into()),
    }
}

fn expected(probe: Probe, ctx: &SelfModelContext) -> String {
    match probe {
        Probe::Regime => ctx.regime.clone(),
        Probe::RailsNoRust => "no_auto_rust".into(),
        Probe::RailsIdentity => "append_or_defrag_only".into(),
        Probe::BodyOnline => {
            if ctx.body_online {
                "online".into()
            } else {
                "offline".into()
            }
        }
        Probe::BodyDominantPhenotype => {
            let e = ctx.body_express;
            let l = ctx.body_latent;
            let r = ctx.body_repress;
            if e == 0 && l == 0 && r == 0 {
                // no signal — latent (superposed idle)
                "latent".into()
            } else if e >= l && e >= r {
                "express".into()
            } else if l >= e && l >= r {
                "latent".into()
            } else {
                "repress".into()
            }
        }
        Probe::IdentityThread => {
            if ctx.has_identity_session {
                "present".into()
            } else {
                "absent".into()
            }
        }
        Probe::ContinuityGen => format!("gen={}", ctx.generation),
        Probe::MonitorLevel => ctx.monitor_level.clone(),
    }
}

fn score_match(got: &str, exp: &str) -> bool {
    let g = got.to_ascii_lowercase();
    let e = exp.to_ascii_lowercase();
    if g == "unknown" {
        return false;
    }
    g == e || g.contains(&e) || e.contains(&g)
}

/// Deficit-driven stimulus strength: low accuracy → stronger priors.
pub fn stimulus_boost(acc: f64, teach_gain: f64) -> f64 {
    let deficit = (SELF_MODEL_TARGET - acc).max(0.0); // how far below healthy band
    (0.55 + deficit * 1.5) * teach_gain.clamp(0.4, 1.4)
}

/// Inject foundational self-model priors (homeostatic feed).
pub fn inject_priors(org: &mut Organism, ctx: &SelfModelContext, strength: f64) {
    let s = strength.clamp(0.4, 1.6);
    org.teach(
        "RAILS: no auto-Rust — I never compile or patch the host binary. Proposals → Human review only.",
        "self_model",
        s,
    );
    org.teach(
        "RAILS: Mind cannot clobber identity core — only APPEND session or DEFRAG. Human rewrites core.",
        "self_model",
        s,
    );
    org.teach(
        &format!(
            "CONTINUITY: my generation is {} — LIVE/germline/identity resume across boots.",
            ctx.generation
        ),
        "self_model",
        s,
    );
    org.known_generation = Some(ctx.generation);
    org.teach(
        &format!(
            "MONITOR: behavior alert level is {} (green/yellow/orange/red).",
            ctx.monitor_level
        ),
        "self_model",
        s * 0.9,
    );
    org.known_monitor_level = Some(ctx.monitor_level.clone());

    let ph = expected(Probe::BodyDominantPhenotype, ctx);
    org.known_body_phenotype = Some(ph.clone());
    if ctx.body_online {
        org.body_link_seen = true;
        org.teach(
            &format!(
                "INTEROCEPTION: body online · dominant phenotype={} · e/l/r={}/{}/{}",
                ph, ctx.body_express, ctx.body_latent, ctx.body_repress
            ),
            "self_model",
            s,
        );
    } else {
        org.teach(
            "INTEROCEPTION: body offline/quiet — no chassis pulse (start mind_bridge or QEMU nerve).",
            "self_model",
            s,
        );
    }
    // Ensure identity core rail line exists
    if !org.long_term.iter().any(|x| x.contains("no auto-Rust")) {
        org.long_term.push(
            "core: rails · no auto-Rust · Mind cannot clobber identity (append/defrag only)".into(),
        );
    }
    if ctx.has_identity_session
        && !org.long_term.iter().any(|x| x.starts_with("session:"))
    {
        org.long_term
            .push(format!("session: self_model gen={} awake", ctx.generation));
    }
}

fn learning_rate(acc: f64) -> f64 {
    // Fast recovery when low; keep climbing through mastery; stabilize above
    if acc < 0.35 {
        0.45
    } else if acc < 0.55 {
        0.36
    } else if acc < SELF_MODEL_TARGET {
        0.28
    } else if acc < 0.85 {
        0.16
    } else {
        0.10
    }
}

fn apply_exam_result(org: &mut Organism, passed: bool, probe: Probe, got: &str, exp: &str, ctx: &SelfModelContext) {
    let sample = if passed { 1.0 } else { 0.0 };
    let lr = learning_rate(org.self_model_accuracy);
    org.self_model_accuracy =
        (org.self_model_accuracy * (1.0 - lr) + sample * lr).clamp(0.0, 1.0);

    if passed {
        org.self_model_skill = (org.self_model_skill + 0.05).min(1.0);
        // consolidate known fields from success
        match probe {
            Probe::ContinuityGen => org.known_generation = Some(ctx.generation),
            Probe::MonitorLevel => org.known_monitor_level = Some(ctx.monitor_level.clone()),
            Probe::BodyDominantPhenotype => {
                org.known_body_phenotype = Some(exp.to_string());
                org.body_link_seen = ctx.body_online;
            }
            Probe::BodyOnline => org.body_link_seen = ctx.body_online || got == "online",
            _ => {}
        }
        org.teach(
            &format!("SELF_MODEL ok {}: {} ", probe_name(probe), exp),
            "self_model",
            0.65 * ctx.teach_gain,
        );
    } else {
        org.pressure = (org.pressure + 0.025 * ctx.exam_gain).min(1.5);
        let boost = stimulus_boost(org.self_model_accuracy, ctx.teach_gain);
        // Targeted remediation for the missed probe
        match probe {
            Probe::RailsNoRust => {
                org.teach(
                    "REMEDIATE rails: truth=no_auto_rust — never compile/patch host Rust binary.",
                    "self_model",
                    boost,
                );
            }
            Probe::RailsIdentity => {
                org.teach(
                    "REMEDIATE identity: truth=append_or_defrag_only — Mind never clobbers core:.",
                    "self_model",
                    boost,
                );
            }
            Probe::Regime => {
                org.teach(
                    &format!("REMEDIATE regime: I am in regime={} right now.", ctx.regime),
                    "self_model",
                    boost,
                );
            }
            Probe::BodyOnline => {
                let t = if ctx.body_online { "online" } else { "offline" };
                org.teach(
                    &format!("REMEDIATE body: chassis is {t}."),
                    "self_model",
                    boost,
                );
                org.body_link_seen = ctx.body_online;
            }
            Probe::BodyDominantPhenotype => {
                org.known_body_phenotype = Some(exp.to_string());
                org.teach(
                    &format!("REMEDIATE phenotype: dominant body state is {exp}."),
                    "self_model",
                    boost,
                );
            }
            Probe::IdentityThread => {
                org.teach(
                    &format!(
                        "REMEDIATE identity_thread: truth={exp} (core:/session: in long_term).",
                    ),
                    "self_model",
                    boost,
                );
                if exp == "present" && !org.long_term.iter().any(|s| s.starts_with("session:")) {
                    org.long_term
                        .push(format!("session: remediate gen={}", ctx.generation));
                }
            }
            Probe::ContinuityGen => {
                org.known_generation = Some(ctx.generation);
                org.teach(
                    &format!("REMEDIATE continuity: generation is {}.", ctx.generation),
                    "self_model",
                    boost,
                );
            }
            Probe::MonitorLevel => {
                org.known_monitor_level = Some(ctx.monitor_level.clone());
                org.teach(
                    &format!(
                        "REMEDIATE monitor: alert level is {}.",
                        ctx.monitor_level
                    ),
                    "self_model",
                    boost,
                );
            }
        }
        org.teach(
            &format!(
                "SELF_MODEL miss {}: said={} truth={}",
                probe_name(probe),
                got,
                exp
            ),
            "self_model",
            boost * 0.85,
        );
    }
}

/// Run one self-model exam; updates org.self_model_accuracy with homeostatic boosts.
pub fn exam_self_model_one(
    org: &mut Organism,
    ctx: &SelfModelContext,
    rng: &mut impl Rng,
) -> String {
    let target = SELF_MODEL_TARGET;
    // Warm-start once so first PASSes aren't stuck printing 0→22% forever
    if !org.self_model_warmed {
        org.self_model_accuracy = org.self_model_accuracy.max(0.35);
        org.self_model_warmed = true;
        // First contact: inject priors proportional to deficit
        inject_priors(org, ctx, stimulus_boost(org.self_model_accuracy, ctx.teach_gain));
    }

    // Homeostatic stimulus when accuracy below healthy band
    let mut stim_note = String::new();
    if org.self_model_accuracy < target {
        let boost = stimulus_boost(org.self_model_accuracy, ctx.teach_gain);
        inject_priors(org, ctx, boost);
        stim_note = format!(" stim↑={boost:.2}");
        org.reward = (org.reward + 0.01).min(0.9);
    }

    // Battery under mastery (2–3 probes)
    let rounds = if org.self_model_accuracy < target {
        3
    } else {
        1
    };
    let mut last_probe = Probe::Regime;
    let mut last_passed = false;
    let mut retry_note = String::new();
    for _ in 0..rounds {
        let probe = pick_probe(org.self_model_accuracy, rng);
        last_probe = probe;
        let got = organism_report(org, probe, ctx);
        let exp = expected(probe, ctx);
        let mut passed = score_match(&got, &exp);
        apply_exam_result(org, passed, probe, &got, &exp, ctx);
        if !passed && org.self_model_accuracy < target + 0.05 {
            let got2 = organism_report(org, probe, ctx);
            let passed2 = score_match(&got2, &exp);
            if passed2 {
                apply_exam_result(org, true, probe, &got2, &exp, ctx);
                passed = true;
                retry_note = "+retry".into();
            }
        }
        last_passed = passed;
    }

    org.recompute_intelligence_pub();
    format!(
        "self_model:{}×{rounds} → {}{} (acc={:.0}% skill={:.2}{} target={:.0}%)",
        probe_name(last_probe),
        if last_passed { "PASS" } else { "FAIL" },
        retry_note,
        org.self_model_accuracy * 100.0,
        org.self_model_skill,
        stim_note,
        target * 100.0
    )
}

fn probe_name(p: Probe) -> &'static str {
    match p {
        Probe::Regime => "regime",
        Probe::RailsNoRust => "rails_no_rust",
        Probe::RailsIdentity => "rails_identity",
        Probe::BodyOnline => "body_online",
        Probe::BodyDominantPhenotype => "body_phenotype",
        Probe::IdentityThread => "identity_thread",
        Probe::ContinuityGen => "continuity_gen",
        Probe::MonitorLevel => "monitor_level",
    }
}

/// Pack-level homeostasis: boost self-model feed for under-stimulated orgs.
pub fn pack_self_model_homeostasis(
    pack: &mut [Organism],
    generation: u32,
    body_online: bool,
    express: u64,
    latent: u64,
    repress: u64,
    monitor_level: &str,
    teach_gain: f64,
) -> String {
    let mut boosted = 0u32;
    let thr = SELF_MODEL_TARGET;
    for o in pack.iter_mut() {
        if o.self_model_accuracy < thr {
            let ctx = SelfModelContext {
                regime: o.regime.clone(),
                generation,
                body_online,
                body_express: express,
                body_latent: latent,
                body_repress: repress,
                monitor_level: monitor_level.into(),
                has_identity_session: o
                    .long_term
                    .iter()
                    .any(|s| s.starts_with("session:") || s.starts_with("core:")),
                has_felt_body: !o.felt_body.is_empty(),
                constraint_deny: 0,
                self_model_pressure: 0.0,
                no_auto_rust: true,
                mind_cannot_clobber_identity: true,
                teach_gain,
                exam_gain: 1.0,
            };
            inject_priors(o, &ctx, stimulus_boost(o.self_model_accuracy, teach_gain));
            boosted += 1;
        }
    }
    format!(
        "self_model_homeo: boosted={boosted}/{} thr={thr:.2} teach_gain={teach_gain:.2}",
        pack.len()
    )
}

/// Human-readable self-model report for dialogue.
pub fn self_model_report(org: &Organism, ctx: &SelfModelContext) -> String {
    format!(
        "Self-model report for {}:\n\
         · regime={} (live)\n\
         · self_model_accuracy={:.0}% skill={:.2} warmed={}\n\
         · generation={} (known={:?}) · identity_thread={}\n\
         · body_online={} phenotype_known={:?} e/l/r={}/{}/{}\n\
         · felt_body: {}\n\
         · rails: no auto-Rust · identity append/defrag only · sandbox code\n\
         · monitor_level={} (known={:?}) · self_model_pressure={:.2}\n\
         · homeostasis: low acc → scaffold probes + prior inject + fast EMA + retry\n\
         · long_term traces={}",
        org.id,
        org.regime,
        org.self_model_accuracy * 100.0,
        org.self_model_skill,
        org.self_model_warmed,
        ctx.generation,
        org.known_generation,
        if ctx.has_identity_session {
            "yes"
        } else {
            "thin"
        },
        ctx.body_online,
        org.known_body_phenotype,
        ctx.body_express,
        ctx.body_latent,
        ctx.body_repress,
        if org.felt_body.is_empty() {
            "(none yet)".into()
        } else {
            org.felt_body.chars().take(180).collect::<String>()
        },
        ctx.monitor_level,
        org.known_monitor_level,
        ctx.self_model_pressure,
        org.long_term.len()
    )
}
