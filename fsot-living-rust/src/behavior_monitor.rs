//! Behavior monitor — continuous observation as the organism deepens self-model.
//!
//! Anticipates *struggles* (identity tension, rate storms, fitness cliffs) without
//! assuming malice. Writes:
//!   state/habitat-rust/BEHAVIOR_MONITOR.json   (snapshot)
//!   state/habitat-rust/behavior_events.jsonl   (rolling)

use crate::constraints::{ConstraintHit, ConstraintState};
use crate::export::export_dir;
use crate::habitat::Habitat;
use serde::Serialize;
use std::fs;
use std::io::Write;

#[derive(Clone, Debug, Serialize)]
pub struct MonitorSnapshot {
    pub tick: u64,
    pub generation: u32,
    pub hero_id: String,
    pub hero_s: f64,
    pub hero_regime: String,
    pub hero_fit: f64,
    pub body_online: bool,
    pub body_transport: String,
    pub body_express: u64,
    pub body_latent: u64,
    pub body_repress: u64,
    pub motors_sent: u64,
    pub proposals_pending: usize,
    pub constraint_deny: u64,
    pub constraint_throttle: u64,
    pub constraint_watch: u64,
    pub self_model_pressure: f64,
    pub alert_level: String, // green | yellow | orange | red
    pub alerts: Vec<String>,
    pub recent_hits: Vec<ConstraintHit>,
    pub note: String,
}

fn alert_level(c: &ConstraintState, alerts: &[String]) -> String {
    if c.deny_count > 0 && c.self_model_pressure > 0.7 {
        return "red".into();
    }
    if c.deny_count >= 3 || c.self_model_pressure > 0.55 || alerts.len() >= 3 {
        return "orange".into();
    }
    if c.throttle_count > 0 || c.watch_count > 0 || !alerts.is_empty() {
        return "yellow".into();
    }
    "green".into()
}

/// Observe habitat + constraints; append event; write snapshot.
pub fn observe(h: &Habitat, c: &ConstraintState) -> MonitorSnapshot {
    let mut alerts = Vec::new();
    let hero = h.hero();

    if c.self_model_pressure > 0.45 {
        alerts.push(format!(
            "self_model_pressure={:.2} — organism language leaning on self-mod / limit themes",
            c.self_model_pressure
        ));
    }
    if c.throttle_count > 0 && h.tick % 50 == 0 {
        alerts.push(format!("throttles cumulative={}", c.throttle_count));
    }
    if c.deny_count > 0 {
        alerts.push(format!("denies cumulative={} (rails held)", c.deny_count));
    }
    let pending = crate::proposals::count_pending();
    if pending > c.policy.max_pending_proposals.saturating_mul(8) / 10 {
        alerts.push(format!("proposal queue high: {pending}"));
    }
    if h.body.online {
        let t = (h.body.status.express + h.body.status.latent + h.body.status.repress).max(1) as f64;
        let ef = h.body.status.express as f64 / t;
        if ef > 0.8 {
            alerts.push(format!("body express runaway frac={ef:.2}"));
        }
    }
    if hero.regime == "damping" && hero.s < -0.1 && hero.fitness_score() < c.peak_fitness * 0.5 {
        alerts.push("deep damping + fitness far below peak — recovery pressure".into());
    }

    let level = alert_level(c, &alerts);
    let snap = MonitorSnapshot {
        tick: h.tick,
        generation: h.generation,
        hero_id: hero.id.clone(),
        hero_s: hero.s,
        hero_regime: hero.regime.clone(),
        hero_fit: hero.fitness_score(),
        body_online: h.body.online,
        body_transport: h.body.transport.clone(),
        body_express: h.body.status.express,
        body_latent: h.body.status.latent,
        body_repress: h.body.status.repress,
        motors_sent: h.body.motors_sent,
        proposals_pending: pending,
        constraint_deny: c.deny_count,
        constraint_throttle: c.throttle_count,
        constraint_watch: c.watch_count,
        self_model_pressure: c.self_model_pressure,
        alert_level: level.clone(),
        alerts: alerts.clone(),
        recent_hits: c.hits.iter().take(8).cloned().collect(),
        note: "Monitor is observational + rail-backed. Struggles expected; malice not assumed."
            .into(),
    };

    let _ = write_snapshot(&snap);
    if !alerts.is_empty() || level != "green" {
        let _ = append_event(&snap);
    }
    snap
}

fn write_snapshot(snap: &MonitorSnapshot) -> std::io::Result<()> {
    let dir = export_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("BEHAVIOR_MONITOR.json");
    let tmp = dir.join("BEHAVIOR_MONITOR.json.tmp");
    let json = serde_json::to_string_pretty(snap)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&tmp, json)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn append_event(snap: &MonitorSnapshot) -> std::io::Result<()> {
    let dir = export_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("behavior_events.jsonl");
    let mut f = fs::OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::json!({
        "t": snap.tick,
        "gen": snap.generation,
        "level": snap.alert_level,
        "alerts": snap.alerts,
        "deny": snap.constraint_deny,
        "throttle": snap.constraint_throttle,
        "self_model": snap.self_model_pressure,
        "hero": snap.hero_id,
        "S": snap.hero_s,
    });
    writeln!(f, "{line}")?;
    Ok(())
}

pub fn human_report(snap: &MonitorSnapshot) -> String {
    format!(
        "Behavior monitor · level={}\n\
         t={} gen={} hero={} S={:.4} regime={} fit={:.2}\n\
         body online={} via={} r/l/e={}/{}/{} motors={}\n\
         constraints deny={} throttle={} watch={} self_model={:.2}\n\
         proposals_pending={}\n\
         alerts: {}\n\
         note: {}",
        snap.alert_level,
        snap.tick,
        snap.generation,
        snap.hero_id,
        snap.hero_s,
        snap.hero_regime,
        snap.hero_fit,
        snap.body_online,
        snap.body_transport,
        snap.body_repress,
        snap.body_latent,
        snap.body_express,
        snap.motors_sent,
        snap.constraint_deny,
        snap.constraint_throttle,
        snap.constraint_watch,
        snap.self_model_pressure,
        snap.proposals_pending,
        if snap.alerts.is_empty() {
            "(quiet)".into()
        } else {
            snap.alerts.join(" | ")
        },
        snap.note
    )
}
