//! Memory tiers — disk LTM vs RAM working set.
//!
//! | Tier | Substrate | Contents |
//! |------|-----------|----------|
//! | Working | RAM | loci fields, expression, motor_bias, intero ring head |
//! | Nerves | TCP/serial | live MIND/MOTOR (not stored here) |
//! | Long-term | Disk lines | consolidated snapshot after SLEEP |
//!
//! Consolidation ≈ sleep: plastic rest + serialize durable state.
//! Wake ≈ load disk → rehydrate RAM (loci can stay plastic, not frozen bits).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use genetic::{GeneticFabric, Locus};
use trinary::{collapse, Trit};

use crate::organs::OrganId;
use crate::soma::Soma;

const MAGIC: &str = "FSOT_SOMA_V1";

/// Disk-durable long-term snapshot (still trinary — not binary genes).
#[derive(Clone, Debug, Default)]
pub struct SomaSnapshot {
    pub tick: u64,
    pub breath_phase: f64,
    pub motor_bias: f64,
    pub breath_count: u64,
    pub focus: usize,
    pub genome_fp: u64,
    pub codons: Vec<String>,
    pub loci: Vec<LocusSnap>,
    pub organs: Vec<OrganSnap>,
    pub scale_tier: String,
}

#[derive(Clone, Debug)]
pub struct LocusSnap {
    pub field: f64,
    pub state: i8,
    pub expression: f64,
    pub plasticity: f64,
}

#[derive(Clone, Debug)]
pub struct OrganSnap {
    pub id: String,
    pub field: f64,
    pub state: i8,
    pub content: String,
}

impl SomaSnapshot {
    /// Encode to line-oriented text for hard-drive LTM.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        s.push_str(MAGIC);
        s.push('\n');
        s.push_str(&format!("tick={}\n", self.tick));
        s.push_str(&format!("breath_phase={:.6}\n", self.breath_phase));
        s.push_str(&format!("motor_bias={:.6}\n", self.motor_bias));
        s.push_str(&format!("breath_count={}\n", self.breath_count));
        s.push_str(&format!("focus={}\n", self.focus));
        s.push_str(&format!("genome_fp=0x{:016x}\n", self.genome_fp));
        s.push_str(&format!("scale_tier={}\n", self.scale_tier));
        s.push_str(&format!("codons={}\n", self.codons.join(",")));
        for (i, l) in self.loci.iter().enumerate() {
            s.push_str(&format!(
                "locus={} field={:.6} state={} expr={:.4} plast={:.4}\n",
                i, l.field, l.state, l.expression, l.plasticity
            ));
        }
        for o in &self.organs {
            // Escape newlines in content
            let content = o.content.replace('\n', "\\n").replace('\r', "");
            s.push_str(&format!(
                "organ={} field={:.4} state={} content={}\n",
                o.id, o.field, o.state, content
            ));
        }
        s.push_str("END\n");
        s
    }

    pub fn from_text(text: &str) -> Option<Self> {
        let mut snap = SomaSnapshot::default();
        let mut saw_magic = false;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == MAGIC {
                saw_magic = true;
                continue;
            }
            if line == "END" {
                break;
            }
            if !saw_magic {
                continue;
            }
            if let Some(v) = line.strip_prefix("tick=") {
                snap.tick = v.parse().unwrap_or(0);
            } else if let Some(v) = line.strip_prefix("breath_phase=") {
                snap.breath_phase = v.parse().unwrap_or(0.0);
            } else if let Some(v) = line.strip_prefix("motor_bias=") {
                snap.motor_bias = v.parse().unwrap_or(0.0);
            } else if let Some(v) = line.strip_prefix("breath_count=") {
                snap.breath_count = v.parse().unwrap_or(0);
            } else if let Some(v) = line.strip_prefix("focus=") {
                snap.focus = v.parse().unwrap_or(0);
            } else if let Some(v) = line.strip_prefix("genome_fp=") {
                let v = v.trim_start_matches("0x");
                snap.genome_fp = u64::from_str_radix(v, 16).unwrap_or(0);
            } else if let Some(v) = line.strip_prefix("scale_tier=") {
                snap.scale_tier = String::from(v);
            } else if let Some(v) = line.strip_prefix("codons=") {
                snap.codons = v
                    .split(',')
                    .filter(|c| !c.is_empty())
                    .map(String::from)
                    .collect();
            } else if line.starts_with("locus=") {
                // locus=i field=.. state=.. expr=.. plast=..
                let mut field = 0.0;
                let mut state = 0i8;
                let mut expression = 0.5;
                let mut plasticity = 0.5;
                for part in line.split_whitespace() {
                    if let Some(v) = part.strip_prefix("field=") {
                        field = v.parse().unwrap_or(0.0);
                    } else if let Some(v) = part.strip_prefix("state=") {
                        state = v.parse().unwrap_or(0);
                    } else if let Some(v) = part.strip_prefix("expr=") {
                        expression = v.parse().unwrap_or(0.5);
                    } else if let Some(v) = part.strip_prefix("plast=") {
                        plasticity = v.parse().unwrap_or(0.5);
                    }
                }
                snap.loci.push(LocusSnap {
                    field,
                    state,
                    expression,
                    plasticity,
                });
            } else if line.starts_with("organ=") {
                let mut id = String::new();
                let mut field = 0.0;
                let mut state = 0i8;
                let mut content = String::new();
                // content may contain spaces — split once after content=
                if let Some(rest) = line.strip_prefix("organ=") {
                    let parts: Vec<&str> = rest.splitn(2, " content=").collect();
                    let head = parts[0];
                    if parts.len() > 1 {
                        content = parts[1].replace("\\n", "\n");
                    }
                    for part in head.split_whitespace() {
                        if !part.contains('=') && id.is_empty() {
                            id = String::from(part);
                        } else if let Some(v) = part.strip_prefix("field=") {
                            field = v.parse().unwrap_or(0.0);
                        } else if let Some(v) = part.strip_prefix("state=") {
                            state = v.parse().unwrap_or(0);
                        } else if id.is_empty() {
                            id = String::from(part);
                        }
                    }
                    // organ=knowledge field=0.1 state=1
                    if let Some(first) = head.split_whitespace().next() {
                        id = String::from(first);
                    }
                }
                snap.organs.push(OrganSnap {
                    id,
                    field,
                    state,
                    content,
                });
            }
        }
        if saw_magic {
            Some(snap)
        } else {
            None
        }
    }
}

impl Soma {
    /// Capture working set → durable snapshot (still in RAM until host writes disk).
    pub fn capture_snapshot(&self) -> SomaSnapshot {
        let loci = self
            .fabric
            .loci
            .iter()
            .map(|l| LocusSnap {
                field: l.field,
                state: l.state.to_i8(),
                expression: l.expression,
                plasticity: l.plasticity,
            })
            .collect();
        let mut organs = Vec::new();
        for id in OrganId::all() {
            let n = self.membrane.get(id);
            organs.push(OrganSnap {
                id: String::from(id.label()),
                field: n.field,
                state: n.state.to_i8(),
                content: String::from_utf8_lossy(&n.content).into_owned(),
            });
        }
        SomaSnapshot {
            tick: self.tick,
            breath_phase: self.breath_phase,
            motor_bias: self.motor_bias,
            breath_count: self.intero.breath_count,
            focus: self.intero.focus,
            genome_fp: self.fabric.fingerprint(),
            codons: self.fabric.codons.clone(),
            loci,
            organs,
            scale_tier: self.scale.tier.as_str().into(),
        }
    }

    /// Sleep: plastic rest → **FSOT dream weave** → LTM snapshot.
    ///
    /// Dream is not waking DEFRAG. It re-clusters memory by domain/sign/guidance
    /// connectivity (emergence↔damping bridges, genome↔bio links, etc.).
    /// SAVE (elsewhere) is a cold checkpoint without dream.
    pub fn sleep_consolidate(&mut self) -> SomaSnapshot {
        self.fabric.rest_all();
        self.intero.breathe();
        self.motor_bias *= 0.4;

        // Dream phase — FSOT self-organization of connective patterns
        let g = self.fabric.guidance();
        let dream = crate::dream::run_dream(&mut self.membrane, &g, self.tick);
        self.outbox.push(format!(
            "BODY event=DREAM t={} traces={} clusters={} bridges={} bio=fsot_connective_weave",
            self.tick, dream.traces, dream.clusters, dream.bridges
        ));
        // First line of journal for compact telemetry
        if let Some(line) = dream.journal.lines().next() {
            self.outbox.push(format!("BODY dream_journal {}", line));
        }

        let snap = self.capture_snapshot();
        self.consolidations = self.consolidations.wrapping_add(1);
        self.last_consolidate_tick = self.tick;
        self.outbox.push(format!(
            "BODY event=SLEEP_CONSOLIDATE tick={} loci={} organs={} tier={} dream_clusters={}",
            snap.tick,
            snap.loci.len(),
            snap.organs.len(),
            snap.scale_tier,
            dream.clusters
        ));
        snap
    }

    /// Wake: rehydrate working set from disk LTM (trinary, not frozen binary).
    pub fn wake_from_snapshot(&mut self, snap: &SomaSnapshot) -> bool {
        if !snap.codons.is_empty() {
            let refs: Vec<&str> = snap.codons.iter().map(|s| s.as_str()).collect();
            self.fabric = GeneticFabric::from_codons(&refs);
        }
        let n = self.fabric.loci.len().min(snap.loci.len());
        for i in 0..n {
            let s = &snap.loci[i];
            self.fabric.loci[i] = Locus {
                field: s.field,
                state: Trit::from_i8(s.state),
                expression: s.expression.clamp(0.0, 1.0),
                plasticity: s.plasticity.clamp(0.0, 1.0),
                last_collapse_tick: snap.tick,
            };
        }
        self.tick = snap.tick;
        self.breath_phase = snap.breath_phase;
        self.motor_bias = snap.motor_bias;
        self.intero.breath_count = snap.breath_count;
        self.intero.attend(snap.focus);

        for o in &snap.organs {
            let path = match o.id.as_str() {
                "status" => "/body/status",
                "genome" => "/body/genome",
                "intero" => "/body/intero",
                "knowledge" => "/organs/knowledge",
                "tools" => "/organs/tools",
                "notes" => "/organs/notes",
                "bridge" => "/organs/bridge",
                "home_knowledge" => "/home/knowledge",
                "home_tools" => "/home/tools",
                "home_media" => "/home/media",
                "home_genome" => "/home/genome",
                "home_proposals" => "/home/proposals",
                "home_identity" => "/home/identity",
                other => other,
            };
            if let Some(node) = self.membrane.by_path_mut(path) {
                node.field = o.field;
                node.state = if o.state != 0 {
                    Trit::from_i8(o.state)
                } else {
                    collapse(o.field)
                };
                let bytes = o.content.as_bytes();
                let n = bytes.len().min(1024);
                node.content.clear();
                node.content.extend_from_slice(&bytes[..n]);
            }
        }
        self.outbox.push(format!(
            "BODY event=WAKE_FROM_LTM tick={} genome_fp=0x{:016x} loci={}",
            self.tick,
            self.fabric.fingerprint(),
            self.fabric.loci.len()
        ));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleProfile;
    use crate::SensorInputs;

    #[test]
    fn roundtrip_snapshot() {
        let mut soma = Soma::boot_with_scale(ScaleProfile::standard());
        for _ in 0..10 {
            let _ = soma.tick(SensorInputs::default());
        }
        let snap = soma.sleep_consolidate();
        let text = snap.to_text();
        assert!(text.contains(MAGIC));
        let loaded = SomaSnapshot::from_text(&text).expect("parse");
        let mut soma2 = Soma::boot_with_scale(ScaleProfile::standard());
        assert!(soma2.wake_from_snapshot(&loaded));
        assert_eq!(soma2.tick, loaded.tick);
        assert_eq!(soma2.fabric.loci.len(), loaded.loci.len());
    }
}
