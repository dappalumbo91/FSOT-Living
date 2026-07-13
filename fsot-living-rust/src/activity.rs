//! Live activity feed — what the habitat is doing (freeze diagnosis + agent review)

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize)]
pub struct Activity {
    pub tick: u64,
    pub phase: String,
    pub detail: String,
    pub organism_id: Option<String>,
    pub unix_ms: u128,
}

impl Activity {
    pub fn new(tick: u64, phase: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            tick,
            phase: phase.into(),
            detail: detail.into(),
            organism_id: None,
            unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
        }
    }

    pub fn with_org(mut self, id: impl Into<String>) -> Self {
        self.organism_id = Some(id.into());
        self
    }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("state")
        .join("habitat-rust")
}

pub fn write_activity(act: &Activity) {
    let d = dir();
    let _ = fs::create_dir_all(&d);
    if let Ok(json) = serde_json::to_string_pretty(act) {
        let _ = fs::write(d.join("ACTIVITY.json"), json);
    }
    if let Ok(line) = serde_json::to_string(act) {
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(d.join("activity.jsonl"))
        {
            let _ = writeln!(f, "{line}");
            // keep log from growing forever — truncate if huge
        }
    }
}
