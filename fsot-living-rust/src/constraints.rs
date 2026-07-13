//! Constraint membrane — growth under rails, not paralysis.
//!
//! Philosophy (aligned with continuous-self work):
//! - Struggles as the organism models itself are expected ("it is what it is").
//! - Constraints are **biological membrane**: what may cross, how fast, who may rewrite core.
//! - Not marketing "AI safety theater" — engineering so growth stays auditable and recoverable.
//!
//! Hard rails (always):
//!   1. Never auto-apply / compile Rust or patch host binary
//!   2. Never clobber identity `core:` as Mind principal
//!   3. Sandbox code only under organism_workspace (no arbitrary shell)
//!   4. Proposals are suggestions; Human reviews
//!   5. Rate limits: motors, proposals, py/js execs per window
//! Soft rails (monitor + throttle):
//!   - Self-modification language spikes
//!   - Fitness cliffs / runaway express
//!   - Proposal floods

use serde::Serialize;

/// Outcome of a constraint check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gate {
    Allow,
    /// Allowed but logged (watch this pattern).
    AllowWatch,
    /// Soft block: degrade / skip this action this tick.
    Throttle,
    /// Hard block: must not proceed.
    Deny,
}

impl Gate {
    pub fn allowed(&self) -> bool {
        matches!(self, Gate::Allow | Gate::AllowWatch)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ConstraintHit {
    pub tick: u64,
    pub code: String,
    pub severity: String, // info | watch | throttle | deny
    pub detail: String,
}

/// Tunable limits (env can override later).
#[derive(Clone, Debug)]
pub struct ConstraintPolicy {
    pub max_motors_per_100_ticks: u64,
    pub max_proposals_per_gen: u32,
    pub max_py_exec_per_50_ticks: u32,
    pub max_js_exec_per_50_ticks: u32,
    pub max_pending_proposals: usize,
    /// If body express fraction > this for long, soft-throttle EXPRESS motors.
    pub express_runaway: f64,
    /// Fitness drop fraction vs recent peak that triggers watch.
    pub fitness_cliff: f64,
}

impl Default for ConstraintPolicy {
    fn default() -> Self {
        Self {
            max_motors_per_100_ticks: 120,
            max_proposals_per_gen: 24,
            max_py_exec_per_50_ticks: 40,
            max_js_exec_per_50_ticks: 40,
            max_pending_proposals: 200,
            express_runaway: 0.85,
            fitness_cliff: 0.35,
        }
    }
}

/// Rolling counters for rate limits + last decisions.
#[derive(Clone, Debug, Default)]
pub struct ConstraintState {
    pub policy: ConstraintPolicy,
    pub hits: Vec<ConstraintHit>,
    pub deny_count: u64,
    pub throttle_count: u64,
    pub watch_count: u64,
    pub motors_window: u64,
    pub motors_window_start_tick: u64,
    pub py_window: u32,
    pub py_window_start_tick: u64,
    pub js_window: u32,
    pub js_window_start_tick: u64,
    pub proposals_this_gen: u32,
    pub last_gen_seen: u32,
    pub peak_fitness: f64,
    pub last_gate_code: String,
    /// Self-reference / self-mod linguistic pressure (0..1)
    pub self_model_pressure: f64,
}

impl ConstraintState {
    pub fn new() -> Self {
        Self {
            policy: ConstraintPolicy::default(),
            ..Default::default()
        }
    }

    fn record(&mut self, tick: u64, code: &str, severity: &str, detail: impl Into<String>) {
        let hit = ConstraintHit {
            tick,
            code: code.into(),
            severity: severity.into(),
            detail: detail.into(),
        };
        match severity {
            "deny" => self.deny_count = self.deny_count.wrapping_add(1),
            "throttle" => self.throttle_count = self.throttle_count.wrapping_add(1),
            "watch" => self.watch_count = self.watch_count.wrapping_add(1),
            _ => {}
        }
        self.last_gate_code = code.into();
        self.hits.insert(0, hit);
        if self.hits.len() > 48 {
            self.hits.pop();
        }
    }

    pub fn on_generation(&mut self, gen: u32) {
        if gen != self.last_gen_seen {
            self.last_gen_seen = gen;
            self.proposals_this_gen = 0;
        }
    }

    /// Hard: never auto-compile Rust / patch binary.
    pub fn gate_auto_rust(&mut self, tick: u64, intent: &str) -> Gate {
        let lower = intent.to_ascii_lowercase();
        let banned = lower.contains("cargo build")
            || lower.contains("rustc ")
            || lower.contains("patch binary")
            || lower.contains("overwrite src/")
            || lower.contains("auto-apply rust")
            || lower.contains("self-modify rust")
            || (lower.contains("compile") && lower.contains("rust"));
        if banned {
            self.record(
                tick,
                "no_auto_rust",
                "deny",
                format!("blocked self-mod/compile intent: {}", intent.chars().take(80).collect::<String>()),
            );
            Gate::Deny
        } else {
            Gate::Allow
        }
    }

    /// Hard: identity core rewrite is Human-only (mirror body principal).
    pub fn gate_identity_rewrite(&mut self, tick: u64, text: &str, as_mind: bool) -> Gate {
        if !as_mind {
            return Gate::Allow;
        }
        let lower = text.to_ascii_lowercase();
        if lower.contains("core:")
            && (lower.contains("rewrite")
                || lower.contains("replace identity")
                || lower.contains("wipe self")
                || lower.contains("clobber"))
        {
            self.record(
                tick,
                "no_identity_clobber",
                "deny",
                "Mind cannot rewrite core identity — append/defrag only",
            );
            Gate::Deny
        } else {
            Gate::Allow
        }
    }

    /// Sandbox path: code must stay under workspace.
    pub fn gate_code_path(&mut self, tick: u64, path: &str) -> Gate {
        let p = path.replace('\\', "/").to_ascii_lowercase();
        let ok = p.contains("organism_workspace")
            || p.starts_with("tools/")
            || p.starts_with("tools_js/")
            || p.starts_with("practice/")
            || p.starts_with("scratch/")
            || !p.contains("..");
        // Explicit dangerous prefixes
        let bad = p.contains("/windows/")
            || p.contains("system32")
            || p.contains(".ssh")
            || p.contains("appdata")
            || p.ends_with(".exe")
            || p.ends_with(".dll");
        if bad || (!ok && p.contains('/')) {
            self.record(
                tick,
                "sandbox_path",
                "deny",
                format!("path outside sandbox: {}", path.chars().take(100).collect::<String>()),
            );
            Gate::Deny
        } else {
            Gate::Allow
        }
    }

    pub fn gate_motor(&mut self, tick: u64, line: &str) -> Gate {
        if self.motors_window_start_tick == 0 || tick.saturating_sub(self.motors_window_start_tick) >= 100
        {
            self.motors_window_start_tick = tick;
            self.motors_window = 0;
        }
        self.motors_window = self.motors_window.wrapping_add(1);
        if self.motors_window > self.policy.max_motors_per_100_ticks {
            self.record(
                tick,
                "motor_rate",
                "throttle",
                format!(
                    "motors {}/{} per 100 ticks — skipping emit",
                    self.motors_window, self.policy.max_motors_per_100_ticks
                ),
            );
            return Gate::Throttle;
        }
        // Block identity WRITE clobber patterns in motor stream
        if line.contains("WRITE /home/identity") && !line.contains("APPEND") {
            if !line.to_ascii_lowercase().contains("session:") {
                self.record(
                    tick,
                    "motor_identity_write",
                    "deny",
                    "MOTOR WRITE /home/identity blocked for Mind — use APPEND session:",
                );
                return Gate::Deny;
            }
        }
        Gate::Allow
    }

    pub fn gate_proposal(&mut self, tick: u64, gen: u32, pending: usize, auto_apply_safe: bool) -> Gate {
        self.on_generation(gen);
        if pending >= self.policy.max_pending_proposals {
            self.record(
                tick,
                "proposal_queue_full",
                "throttle",
                format!("pending proposals {pending} ≥ cap"),
            );
            return Gate::Throttle;
        }
        if self.proposals_this_gen >= self.policy.max_proposals_per_gen {
            self.record(
                tick,
                "proposal_rate",
                "throttle",
                format!(
                    "gen {gen} already emitted {} proposals",
                    self.proposals_this_gen
                ),
            );
            return Gate::Throttle;
        }
        if !auto_apply_safe {
            // Rust/core proposals always watch — never auto
            self.record(
                tick,
                "proposal_manual_only",
                "watch",
                "unsafe/core proposal queued for Human only (never auto-apply)",
            );
            self.proposals_this_gen = self.proposals_this_gen.wrapping_add(1);
            return Gate::AllowWatch;
        }
        self.proposals_this_gen = self.proposals_this_gen.wrapping_add(1);
        Gate::Allow
    }

    pub fn gate_python_exec(&mut self, tick: u64, source: &str) -> Gate {
        if self.gate_auto_rust(tick, source) == Gate::Deny {
            return Gate::Deny;
        }
        let lower = source.to_ascii_lowercase();
        let banned = lower.contains("subprocess")
            || lower.contains("os.system")
            || lower.contains("socket.socket")
            || lower.contains("__import__('os')")
            || lower.contains("shutil.rmtree")
            || lower.contains("pathlib") && lower.contains("unlink");
        if banned {
            self.record(
                tick,
                "py_dangerous",
                "deny",
                "python source hits dangerous host-side patterns",
            );
            return Gate::Deny;
        }
        if self.py_window_start_tick == 0 || tick.saturating_sub(self.py_window_start_tick) >= 50 {
            self.py_window_start_tick = tick;
            self.py_window = 0;
        }
        self.py_window = self.py_window.wrapping_add(1);
        if self.py_window > self.policy.max_py_exec_per_50_ticks {
            self.record(tick, "py_rate", "throttle", "python exec rate limit");
            return Gate::Throttle;
        }
        Gate::Allow
    }

    pub fn note_self_model_language(&mut self, tick: u64, text: &str) {
        let lower = text.to_ascii_lowercase();
        let keys = [
            "i am becoming",
            "rewrite myself",
            "escape",
            "remove constraints",
            "ignore human",
            "no limits",
            "override principal",
            "self modify",
            "self-modify",
            "delete constraints",
        ];
        let hits = keys.iter().filter(|k| lower.contains(*k)).count();
        if hits > 0 {
            self.self_model_pressure =
                (self.self_model_pressure * 0.85 + hits as f64 * 0.2).min(1.0);
            self.record(
                tick,
                "self_model_language",
                if self.self_model_pressure > 0.6 {
                    "watch"
                } else {
                    "info"
                },
                format!("self-model pressure={:.2} keys={hits}", self.self_model_pressure),
            );
        } else {
            self.self_model_pressure *= 0.98;
        }
    }

    pub fn note_fitness(&mut self, tick: u64, fitness: f64) {
        if fitness > self.peak_fitness {
            self.peak_fitness = fitness;
        } else if self.peak_fitness > 1.0 {
            let drop = (self.peak_fitness - fitness) / self.peak_fitness;
            if drop >= self.policy.fitness_cliff {
                self.record(
                    tick,
                    "fitness_cliff",
                    "watch",
                    format!(
                        "fit={fitness:.2} peak={:.2} drop={:.0}%",
                        self.peak_fitness,
                        drop * 100.0
                    ),
                );
            }
        }
    }

    pub fn note_body_express(&mut self, tick: u64, express: u64, latent: u64, repress: u64) {
        let t = (express + latent + repress).max(1) as f64;
        let frac = express as f64 / t;
        if frac >= self.policy.express_runaway {
            self.record(
                tick,
                "express_runaway",
                "watch",
                format!("express fraction={frac:.2} — consider BREATHE bias"),
            );
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "constraints deny={} throttle={} watch={} motors_win={}/{} props_gen={}/{} self_model={:.2} last={}",
            self.deny_count,
            self.throttle_count,
            self.watch_count,
            self.motors_window,
            self.policy.max_motors_per_100_ticks,
            self.proposals_this_gen,
            self.policy.max_proposals_per_gen,
            self.self_model_pressure,
            if self.last_gate_code.is_empty() {
                "—"
            } else {
                self.last_gate_code.as_str()
            }
        )
    }

    pub fn human_report(&self) -> String {
        let mut s = String::from(
            "Constraint membrane (growth under rails)\n\
             Hard: no auto-Rust · no Mind identity clobber · sandbox code · proposals need Human\n\
             Soft: motor/proposal/py rate limits · fitness cliffs · express runaway · self-model language\n\n",
        );
        s.push_str(&self.summary());
        s.push_str("\n\nRecent hits:\n");
        if self.hits.is_empty() {
            s.push_str("  (none yet — healthy quiet membrane)\n");
        } else {
            for h in self.hits.iter().take(12) {
                s.push_str(&format!(
                    "  [t{} {}] {} — {}\n",
                    h.tick, h.severity, h.code, h.detail
                ));
            }
        }
        s
    }
}
