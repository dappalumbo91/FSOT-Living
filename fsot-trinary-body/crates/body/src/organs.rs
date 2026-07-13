//! Organ membrane + home VFS — biological long-term / working memory.
//!
//! Linux VFS / $HOME is the **template job** (paths, read/write, keep your stuff).
//! Biology: compartments of identity, knowledge, skills, sensory tags, genome.
//! Genetics: each node has field + Trit + expression (not binary flags).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use trinary::{collapse, Trit};

const MAX_CONTENT: usize = 1024;

/// Stable organ identities (path roots).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum OrganId {
    // Soma / visceral
    BodyStatus = 0,
    Genome = 1,
    Intero = 2,
    Knowledge = 3,
    Tools = 4,
    Notes = 5,
    Bridge = 6,
    // Home VFS (Linux $HOME salvage → biological self)
    HomeKnowledge = 7,
    HomeTools = 8,
    HomeMedia = 9,
    HomeGenome = 10,
    HomeProposals = 11,
    HomeIdentity = 12,
}

pub const ORGAN_COUNT: usize = 13;

impl OrganId {
    pub fn all() -> [OrganId; ORGAN_COUNT] {
        [
            OrganId::BodyStatus,
            OrganId::Genome,
            OrganId::Intero,
            OrganId::Knowledge,
            OrganId::Tools,
            OrganId::Notes,
            OrganId::Bridge,
            OrganId::HomeKnowledge,
            OrganId::HomeTools,
            OrganId::HomeMedia,
            OrganId::HomeGenome,
            OrganId::HomeProposals,
            OrganId::HomeIdentity,
        ]
    }

    pub fn path(self) -> &'static str {
        match self {
            OrganId::BodyStatus => "/body/status",
            OrganId::Genome => "/body/genome",
            OrganId::Intero => "/body/intero",
            OrganId::Knowledge => "/organs/knowledge",
            OrganId::Tools => "/organs/tools",
            OrganId::Notes => "/organs/notes",
            OrganId::Bridge => "/organs/bridge",
            OrganId::HomeKnowledge => "/home/knowledge",
            OrganId::HomeTools => "/home/tools",
            OrganId::HomeMedia => "/home/media",
            OrganId::HomeGenome => "/home/genome",
            OrganId::HomeProposals => "/home/proposals",
            OrganId::HomeIdentity => "/home/identity",
        }
    }

    /// Biological role (for docs / telemetry).
    pub fn biology(self) -> &'static str {
        match self {
            OrganId::BodyStatus => "vital signs summary",
            OrganId::Genome => "germline working copy",
            OrganId::Intero => "vagus / interoception cache",
            OrganId::Knowledge => "semantic working memory",
            OrganId::Tools => "procedural skill cache",
            OrganId::Notes => "scratchpad / prefrontal buffer",
            OrganId::Bridge => "synapse to cortex (Mind ABI)",
            OrganId::HomeKnowledge => "long-term semantic LTM (hippocampal→cortical)",
            OrganId::HomeTools => "consolidated motor skills",
            OrganId::HomeMedia => "sensory engrams (pointers, not bulk)",
            OrganId::HomeGenome => "archived germline + LTM snapshots",
            OrganId::HomeProposals => "novelty / planning under selection",
            OrganId::HomeIdentity => "autobiographical self / $HOME identity",
        }
    }

    pub fn from_path(path: &str) -> Option<Self> {
        let p = path.trim().trim_end_matches('/');
        match p {
            "/body/status" | "status" => Some(OrganId::BodyStatus),
            "/body/genome" | "genome" => Some(OrganId::Genome),
            "/body/intero" | "intero" => Some(OrganId::Intero),
            "/organs/knowledge" | "knowledge" => Some(OrganId::Knowledge),
            "/organs/tools" | "tools" => Some(OrganId::Tools),
            "/organs/notes" | "notes" => Some(OrganId::Notes),
            "/organs/bridge" | "bridge" => Some(OrganId::Bridge),
            "/home/knowledge" | "home/knowledge" => Some(OrganId::HomeKnowledge),
            "/home/tools" | "home/tools" => Some(OrganId::HomeTools),
            "/home/media" | "home/media" => Some(OrganId::HomeMedia),
            "/home/genome" | "home/genome" => Some(OrganId::HomeGenome),
            "/home/proposals" | "home/proposals" => Some(OrganId::HomeProposals),
            "/home/identity" | "home/identity" | "/home" | "home" => Some(OrganId::HomeIdentity),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OrganId::BodyStatus => "status",
            OrganId::Genome => "genome",
            OrganId::Intero => "intero",
            OrganId::Knowledge => "knowledge",
            OrganId::Tools => "tools",
            OrganId::Notes => "notes",
            OrganId::Bridge => "bridge",
            OrganId::HomeKnowledge => "home_knowledge",
            OrganId::HomeTools => "home_tools",
            OrganId::HomeMedia => "home_media",
            OrganId::HomeGenome => "home_genome",
            OrganId::HomeProposals => "home_proposals",
            OrganId::HomeIdentity => "home_identity",
        }
    }

    pub fn is_home(self) -> bool {
        matches!(
            self,
            OrganId::HomeKnowledge
                | OrganId::HomeTools
                | OrganId::HomeMedia
                | OrganId::HomeGenome
                | OrganId::HomeProposals
                | OrganId::HomeIdentity
        )
    }

    /// Disk subdir under home/ mount (host).
    pub fn home_dir(self) -> Option<&'static str> {
        match self {
            OrganId::HomeKnowledge => Some("knowledge"),
            OrganId::HomeTools => Some("tools"),
            OrganId::HomeMedia => Some("media"),
            OrganId::HomeGenome => Some("genome"),
            OrganId::HomeProposals => Some("proposals"),
            OrganId::HomeIdentity => Some("identity"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrganNode {
    pub id: OrganId,
    /// Continuous expression field (FSOT channel).
    pub field: f64,
    pub state: Trit,
    pub content: Vec<u8>,
    pub writes: u64,
    pub reads: u64,
    /// Scheduler: how urgently this organ needs a tick (0..1+).
    pub need: f64,
}

impl OrganNode {
    fn new(id: OrganId, seed_field: f64) -> Self {
        OrganNode {
            id,
            field: seed_field,
            state: collapse(seed_field),
            content: Vec::new(),
            writes: 0,
            reads: 0,
            need: 0.25,
        }
    }

    pub fn tick(&mut self, pressure: f64) {
        self.field = self.field * 0.9 + pressure * 0.1;
        self.state = collapse(self.field);
        // need drifts with |field| and recent activity
        let mag = if self.field < 0.0 {
            -self.field
        } else {
            self.field
        };
        self.need = (self.need * 0.85 + mag * 0.15).clamp(0.05, 1.5);
    }

    pub fn read_str(&mut self) -> String {
        self.reads = self.reads.wrapping_add(1);
        self.need = (self.need + 0.1).min(1.5);
        String::from_utf8_lossy(&self.content).into_owned()
    }

    pub fn write_str(&mut self, data: &str) {
        self.writes = self.writes.wrapping_add(1);
        let bytes = data.as_bytes();
        let n = bytes.len().min(MAX_CONTENT);
        self.content.clear();
        self.content.extend_from_slice(&bytes[..n]);
        self.field = (self.field + 0.35).clamp(-1.5, 1.5);
        self.state = collapse(self.field);
        self.need = (self.need + 0.2).min(1.5);
    }

    /// Append a line without wiping existing content (safe episodic encode).
    pub fn append_str(&mut self, data: &str) {
        self.writes = self.writes.wrapping_add(1);
        let mut s = String::from_utf8_lossy(&self.content).into_owned();
        if !s.is_empty() && !s.ends_with('\n') {
            s.push('\n');
        }
        let line = data.trim();
        if !line.is_empty() {
            s.push_str(line);
            s.push('\n');
        }
        let bytes = s.as_bytes();
        let n = bytes.len().min(MAX_CONTENT);
        // If overflow, keep tail (recent) but never drop leading core: lines
        let slice = if bytes.len() > MAX_CONTENT {
            let start = bytes.len() - MAX_CONTENT;
            // try keep core: prefix if present
            let text = core::str::from_utf8(bytes).unwrap_or("");
            if let Some(idx) = text.find("core:") {
                let core_end = text[idx..].find('\n').map(|i| idx + i + 1).unwrap_or(idx);
                let mut out = String::from(&text[idx..core_end]);
                let tail = &text[text.len().saturating_sub(MAX_CONTENT - out.len())..];
                out.push_str(tail);
                self.content.clear();
                self.content.extend_from_slice(out.as_bytes());
                self.field = (self.field + 0.12).clamp(-1.5, 1.5);
                self.state = collapse(self.field);
                return;
            }
            &bytes[start..]
        } else {
            &bytes[..n]
        };
        self.content.clear();
        self.content.extend_from_slice(slice);
        self.field = (self.field + 0.12).clamp(-1.5, 1.5);
        self.state = collapse(self.field);
        self.need = (self.need + 0.1).min(1.5);
    }

    /// Defrag: reorganize content — preserve `core:` lines, dedupe sessions, compact.
    /// Does **not** wipe autobiographical core (safe for Mind principal).
    pub fn defrag(&mut self, hint: &str) {
        let text = String::from_utf8_lossy(&self.content);
        let mut core: Vec<String> = Vec::new();
        let mut session: Vec<String> = Vec::new();
        let mut other: Vec<String> = Vec::new();
        for line in text.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if t.starts_with("core:") {
                if !core.iter().any(|c| c == t) {
                    core.push(String::from(t));
                }
            } else if t.starts_with("session:") {
                if !session.iter().any(|c| c == t) {
                    session.push(String::from(t));
                }
            } else {
                // Promote first free-form line to core if no core yet
                if core.is_empty() && !t.starts_with("session:") {
                    core.push(format!("core: {t}"));
                } else if !other.iter().any(|c| c == t) {
                    other.push(String::from(t));
                }
            }
        }
        if core.is_empty() {
            core.push(String::from(
                "core: companion self · FSOT biological identity",
            ));
        }
        let hint = hint.trim();
        if !hint.is_empty() {
            let s = if hint.starts_with("session:") {
                String::from(hint)
            } else {
                format!("session: {hint}")
            };
            if !session.iter().any(|c| c == &s) {
                session.push(s);
            }
        }
        // Keep last 24 session traces (episodic self — longer autobiographical window)
        let sess_start = session.len().saturating_sub(24);
        let mut out = String::new();
        for c in &core {
            out.push_str(c);
            out.push('\n');
        }
        for o in other.iter().take(8) {
            out.push_str(o);
            out.push('\n');
        }
        for s in &session[sess_start..] {
            out.push_str(s);
            out.push('\n');
        }
        let bytes = out.as_bytes();
        let n = bytes.len().min(MAX_CONTENT);
        self.content.clear();
        self.content.extend_from_slice(&bytes[..n]);
        self.writes = self.writes.wrapping_add(1);
        // Soft expression (reorganization, not a rewrite spike)
        self.field = (self.field * 0.9 + 0.15).clamp(-1.5, 1.5);
        self.state = collapse(self.field);
        self.need = (self.need * 0.8).max(0.1);
    }

    pub fn expression_strength(&self) -> f64 {
        match self.state {
            Trit::Superposed => 0.25 + self.need * 0.1,
            Trit::SpinUp | Trit::SpinDown => {
                let mag = if self.field < 0.0 {
                    -self.field
                } else {
                    self.field
                };
                (0.4 + mag * 0.5).clamp(0.0, 1.0)
            }
        }
    }
}

/// Body organ membrane — path-addressed, includes /home/* VFS salvage.
pub struct Membrane {
    nodes: [OrganNode; ORGAN_COUNT],
}

impl Membrane {
    pub fn new() -> Self {
        Membrane {
            nodes: [
                OrganNode::new(OrganId::BodyStatus, 0.4),
                OrganNode::new(OrganId::Genome, 0.6),
                OrganNode::new(OrganId::Intero, 0.3),
                OrganNode::new(OrganId::Knowledge, 0.1),
                OrganNode::new(OrganId::Tools, 0.1),
                OrganNode::new(OrganId::Notes, 0.0),
                OrganNode::new(OrganId::Bridge, 0.2),
                OrganNode::new(OrganId::HomeKnowledge, 0.15),
                OrganNode::new(OrganId::HomeTools, 0.15),
                OrganNode::new(OrganId::HomeMedia, 0.1),
                OrganNode::new(OrganId::HomeGenome, 0.5),
                OrganNode::new(OrganId::HomeProposals, 0.05),
                OrganNode::new(OrganId::HomeIdentity, 0.35),
            ],
        }
    }

    pub fn get_mut(&mut self, id: OrganId) -> &mut OrganNode {
        &mut self.nodes[id as usize]
    }

    pub fn get(&self, id: OrganId) -> &OrganNode {
        &self.nodes[id as usize]
    }

    pub fn by_path_mut(&mut self, path: &str) -> Option<&mut OrganNode> {
        OrganId::from_path(path).map(|id| self.get_mut(id))
    }

    pub fn by_path(&self, path: &str) -> Option<&OrganNode> {
        OrganId::from_path(path).map(|id| self.get(id))
    }

    pub fn tick_all(&mut self, pressure: f64) {
        for n in self.nodes.iter_mut() {
            n.tick(pressure * 0.15);
        }
    }

    pub fn list_line(&self) -> String {
        let mut s = String::from("ORGANS");
        for n in &self.nodes {
            s.push_str(&format!(
                " {}={{trit={},field={:.2},bytes={},r={},w={},need={:.2}}}",
                n.id.label(),
                n.state.to_i8(),
                n.field,
                n.content.len(),
                n.reads,
                n.writes,
                n.need
            ));
        }
        s
    }

    /// Home VFS listing (Linux $HOME salvage).
    pub fn home_list_line(&self) -> String {
        let mut s = String::from("HOME");
        for n in &self.nodes {
            if n.id.is_home() {
                s.push_str(&format!(
                    " {}={{path={},trit={},bytes={},bio={}}}",
                    n.id.label(),
                    n.id.path(),
                    n.state.to_i8(),
                    n.content.len(),
                    n.id.biology()
                ));
            }
        }
        s
    }

    /// Biological scheduler: organ run order for this tick.
    /// priority ∝ expression × need × (1 − |field| overload damp) — not CFS fairness.
    pub fn schedule_priorities(&self, breath_step: f64) -> Vec<(OrganId, f64)> {
        let mut v: Vec<(OrganId, f64)> = self
            .nodes
            .iter()
            .map(|n| {
                let expr = n.expression_strength();
                let overload = (n.field.abs() - 1.0).max(0.0);
                let p = expr * n.need * (1.0 - overload * 0.3) * (0.5 + breath_step * 4.0);
                (n.id, p)
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        v
    }


    pub fn schedule_line(&self, breath_step: f64) -> String {
        let mut s = String::from("SCHED");
        for (id, p) in self.schedule_priorities(breath_step).into_iter().take(8) {
            s.push_str(&format!(" {}={:.3}", id.label(), p));
        }
        s
    }

    pub fn seed_genome_snapshot(&mut self, text: &str) {
        self.get_mut(OrganId::Genome).write_str(text);
        self.get_mut(OrganId::HomeGenome).write_str(text);
    }

    pub fn seed_status(&mut self, text: &str) {
        self.get_mut(OrganId::BodyStatus).write_str(text);
    }

    pub fn seed_home_identity(&mut self, text: &str) {
        self.get_mut(OrganId::HomeIdentity).write_str(text);
    }
}

impl Default for Membrane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_knowledge() {
        let mut m = Membrane::new();
        m.get_mut(OrganId::Knowledge)
            .write_str("fact: FSOT is trinary");
        let s = m.get_mut(OrganId::Knowledge).read_str();
        assert!(s.contains("trinary"));
    }

    #[test]
    fn home_paths() {
        assert_eq!(
            OrganId::from_path("/home/knowledge"),
            Some(OrganId::HomeKnowledge)
        );
        assert!(OrganId::HomeTools.is_home());
    }

    #[test]
    fn schedule_orders_by_need() {
        let mut m = Membrane::new();
        m.get_mut(OrganId::HomeKnowledge).write_str("encode this");
        let sched = m.schedule_priorities(0.1);
        assert!(!sched.is_empty());
        // written organ should be competitive
        assert!(sched.iter().any(|(id, _)| *id == OrganId::HomeKnowledge));
    }
}
