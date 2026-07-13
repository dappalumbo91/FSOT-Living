//! Soma — integrated body tick: interoception + genome + organs + Mind ABI + memory tiers.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use genetic::GeneticFabric;
use trinary::{collapse, Trit};

use crate::ignition::Ignition;
use crate::interoception::{Interoception, Sample, SensorInputs};
use crate::memory::SomaSnapshot;
use crate::mind_abi::{
    format_ack, format_mind, format_report, CommandRx, MotorVerb,
};
use crate::organs::{Membrane, OrganId};
use crate::principal::{authorize, Principal};
use crate::scale::ScaleProfile;

/// Full living body state (host or kernel).
pub struct Soma {
    pub fabric: GeneticFabric,
    pub intero: Interoception,
    pub membrane: Membrane,
    pub cmd: CommandRx,
    pub tick: u64,
    pub breath_phase: f64,
    /// Forced motor pressure (EXPRESS / REPRESS), decays each tick.
    pub motor_bias: f64,
    /// Outbound lines for this tick (MIND / ACK / REPORT / ORGANS / DATA).
    pub outbox: Vec<String>,
    /// Goldfish-scale profile (breath / consolidate cadence).
    pub scale: ScaleProfile,
    /// Pending LTM text for host to flush to disk (after SLEEP).
    pub pending_ltm: Option<String>,
    pub consolidations: u64,
    pub last_consolidate_tick: u64,
    /// Developmental ignition graph (Linux init salvage).
    pub ignition: Ignition,
    /// How many organ slots the scheduler runs per tick (scales with tank).
    pub sched_width: usize,
    /// Who is currently authorized for MOTOR (Human keyboard vs Mind cortex).
    pub principal: Principal,
    pub auth_denials: u64,
}

impl Soma {
    pub fn boot() -> Self {
        Self::boot_with_scale(ScaleProfile::standard())
    }

    pub fn boot_with_scale(scale: ScaleProfile) -> Self {
        let fabric = GeneticFabric::seed_default();
        let g = fabric.guidance();
        let mut membrane = Membrane::new();
        membrane.seed_genome_snapshot(&format!(
            "codons={} loci={} delta_psi={:.3} d_eff={:.2} emergence={:.3} fp=0x{:016x} aa={}",
            g.codon_count,
            fabric.loci.len(),
            g.delta_psi,
            g.d_eff,
            g.emergence_preference,
            fabric.fingerprint(),
            g.aa_chain
        ));
        membrane.seed_status(&format!(
            "zygote online; trinary genetic soma; {}",
            scale.summary_line()
        ));
        membrane
            .get_mut(OrganId::Bridge)
            .write_str("awaiting cortex; Mind ABI MOTOR/MIND; memory tiers RAM+disk");
        membrane
            .get_mut(OrganId::Tools)
            .write_str("serial,intero,genome,membrane,sleep,wake,scale");
        membrane
            .get_mut(OrganId::Knowledge)
            .write_str("seed: disk=LTM RAM=STM TCP=nerves CPU=genetic flow");
        membrane.seed_home_identity(
            "core: companion self · keep your stuff · FSOT biological identity\nsession: boot",
        );
        membrane.get_mut(OrganId::HomeKnowledge).write_str(
            "LTM knowledge compartment — MOTOR WRITE /home/knowledge <fact>",
        );
        membrane.get_mut(OrganId::HomeTools).write_str(
            "procedural skills — forged tools land here on consolidate",
        );
        membrane.get_mut(OrganId::HomeMedia).write_str(
            "sensory tags/pointers (not bulk media) — e.g. G:\\ path refs",
        );
        membrane.get_mut(OrganId::HomeProposals).write_str(
            "novelty under selection — links living PROPOSALS organ",
        );

        // Wider tank → more organs processed per tick (goldfish metabolic width)
        let sched_width = match scale.tier.as_str() {
            "lean" => 2,
            "standard" => 3,
            "expanded" => 4,
            _ => 5, // max
        };

        let mut ignition = Ignition::new();
        let mut boot_events = ignition.run_boot_cascade();

        let mut soma = Soma {
            fabric,
            intero: Interoception::new(),
            membrane,
            cmd: CommandRx::new(),
            tick: 0,
            breath_phase: 0.0,
            motor_bias: 0.0,
            outbox: Vec::new(),
            scale,
            pending_ltm: None,
            consolidations: 0,
            last_consolidate_tick: 0,
            ignition,
            sched_width,
            principal: Principal::Mind, // TCP cortex default; set Human for local console
            auth_denials: 0,
        };
        soma.outbox.push(soma.scale.summary_line());
        soma.outbox.append(&mut boot_events);
        soma.outbox.push(format!(
            "BODY event=PRINCIPAL default={} bio=motor_authority",
            soma.principal.label()
        ));
        soma.outbox
            .push(String::from("BODY event=HOME_VFS_ONLINE paths=/home/{knowledge,tools,media,genome,proposals,identity}"));
        soma.outbox.push(format!(
            "BODY event=SCHED_WIDTH n={} bio=metabolic_parallelism",
            soma.sched_width
        ));
        soma
    }

    /// Run one scheduled organ (biological attention slice).
    fn activate_organ(&mut self, id: OrganId, pressure: f64) {
        let node = self.membrane.get_mut(id);
        // Local FSOT-ish pulse: need + pressure couples into field
        let pulse = pressure * 0.12 + node.need * 0.08;
        node.field = (node.field * 0.88 + pulse).clamp(-1.5, 1.5);
        node.state = collapse(node.field);
        node.need = (node.need * 0.92).max(0.05);
        // Home knowledge under express: light "recall" marker
        if id == OrganId::HomeKnowledge && node.state == Trit::SpinUp {
            node.reads = node.reads.wrapping_add(1);
        }
    }

    /// After organ writes, host may flush /home/* to disk (biological LTM).
    pub fn home_dirty_labels(&self) -> Vec<&'static str> {
        self.membrane
            .schedule_priorities(self.scale.breath_step)
            .into_iter()
            .filter_map(|(id, _)| if id.is_home() { Some(id.label()) } else { None })
            .collect()
    }

    /// One homeostasis cycle. `sensors` should be filled by platform (kernel/host).
    pub fn tick(&mut self, mut sensors: SensorInputs) -> Sample {
        self.outbox.clear();
        self.tick = self.tick.wrapping_add(1);
        self.breath_phase += self.scale.breath_step;
        if self.breath_phase > 6.283185307179586 {
            self.breath_phase -= 6.283185307179586;
        }
        sensors.breath_phase = self.breath_phase;
        sensors.uptime_seconds = self.tick as f64 * 0.05;

        let sample = self.intero.sample(&sensors);
        let pressure = sample.pressure + self.motor_bias;
        self.motor_bias *= 0.92;

        self.fabric.tick_pressure(pressure);
        self.membrane.tick_all(pressure);

        // Homeostasis scheduler: top-N organs get metabolic attention this tick
        // (Linux CFS job → biological priority, not fairness).
        let top: Vec<OrganId> = self
            .membrane
            .schedule_priorities(self.scale.breath_step)
            .into_iter()
            .take(self.sched_width)
            .map(|(id, _)| id)
            .collect();
        for id in top {
            self.activate_organ(id, pressure);
        }
        if self.tick % (self.scale.mind_every.max(2) * 4) == 0 {
            self.outbox
                .push(self.membrane.schedule_line(self.scale.breath_step));
        }

        // Periodic plastic rest (anti-binary freeze) — cadence scales with tank size.
        let rest_every = self.scale.rest_every.max(8);
        if self.tick % rest_every == 0 {
            self.fabric.rest_all();
            self.intero.breathe();
            self.outbox
                .push(String::from("BODY event=BREATHE_REST plastic_loci_resuperpose=1"));
        }

        // Auto-sleep consolidate → host flushes pending_ltm to disk LTM.
        let cons_every = self.scale.consolidate_every.max(16);
        if self.tick % cons_every == 0 {
            let snap = self.sleep_consolidate();
            self.pending_ltm = Some(snap.to_text());
        }

        // Refresh status organ.
        let (d, m, u) = self.fabric.phenotype_tally();
        self.membrane.get_mut(OrganId::BodyStatus).write_str(&format!(
            "tick={} pressure={:.3} repress={} latent={} express={} mean_expr={:.3} tier={} cons={}",
            self.tick,
            pressure,
            d,
            m,
            u,
            self.fabric.mean_expression(),
            self.scale.tier.as_str(),
            self.consolidations
        ));
        self.membrane.get_mut(OrganId::Intero).write_str(&format!(
            "seq={} focus={} breath={} temp={:.3} heap={:.3} breath_sin={:.3}",
            sample.seq,
            self.intero.focus,
            self.intero.breath_count,
            sample.raw[0],
            sample.raw[2],
            sample.raw[4]
        ));

        let mind_every = self.scale.mind_every.max(2);
        if self.tick % mind_every == 0 {
            self.outbox.push(format_mind(
                &sample,
                self.intero.focus,
                self.intero.breath_count,
                &self.fabric,
            ));
        }

        sample
    }

    pub fn handle_byte(&mut self, b: u8) {
        if let Some(verb) = self.cmd.push_byte(b) {
            self.dispatch(verb);
        }
    }

    pub fn handle_line(&mut self, line: &str) {
        // Optional: MOTOR AS human|mind …
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("MOTOR AS ") {
            let (who, cmd) = match rest.find(' ') {
                Some(i) => (&rest[..i], rest[i + 1..].trim()),
                None => (rest, ""),
            };
            if let Some(p) = Principal::from_str_name(who) {
                self.principal = p;
                self.outbox.push(format_ack(
                    "AS",
                    &format!("principal={}", p.label()),
                ));
            }
            if !cmd.is_empty() {
                let full = if cmd.to_ascii_uppercase().starts_with("MOTOR") {
                    String::from(cmd)
                } else {
                    format!("MOTOR {cmd}")
                };
                if let Some(verb) = self.cmd.push_line(&full) {
                    self.dispatch(verb);
                }
            }
            return;
        }
        if let Some(verb) = self.cmd.push_line(line) {
            self.dispatch(verb);
        }
    }

    pub fn set_principal(&mut self, p: Principal) {
        self.principal = p;
    }

    pub fn dispatch(&mut self, verb: MotorVerb) {
        if let Err(detail) = authorize(self.principal, &verb) {
            self.auth_denials = self.auth_denials.wrapping_add(1);
            self.outbox.push(format_ack("DENIED", &detail));
            self.outbox.push(format!(
                "BODY event=AUTH_DENY principal={} detail={}",
                self.principal.label(),
                detail
            ));
            return;
        }
        match verb {
            MotorVerb::Status => {
                if let Some(s) = self.intero.latest() {
                    self.outbox.push(format_mind(
                        &s,
                        self.intero.focus,
                        self.intero.breath_count,
                        &self.fabric,
                    ));
                }
                self.outbox
                    .push(format_ack("STATUS", "telemetry_emitted"));
            }
            MotorVerb::Attend(ch) => {
                self.intero.attend(ch);
                self.outbox
                    .push(format_ack("ATTEND", &format!("focus={}", ch)));
            }
            MotorVerb::Breathe => {
                self.fabric.rest_all();
                self.intero.breathe();
                self.motor_bias *= 0.5;
                self.outbox
                    .push(format_ack("BREATHE", "rest_and_resuperpose"));
            }
            MotorVerb::Report => {
                if let Some(s) = self.intero.latest() {
                    self.outbox
                        .push(format_report(&s, self.intero.focus));
                }
                self.outbox.push(format_ack("REPORT", "ok"));
            }
            MotorVerb::Note(text) => {
                self.membrane.get_mut(OrganId::Notes).write_str(&text);
                self.outbox
                    .push(format_ack("NOTE", &format!("bytes={}", text.len())));
            }
            MotorVerb::Express => {
                self.motor_bias = 0.85;
                self.outbox
                    .push(format_ack("EXPRESS", "motor_bias=+0.85"));
            }
            MotorVerb::Repress => {
                self.motor_bias = -0.85;
                self.outbox
                    .push(format_ack("REPRESS", "motor_bias=-0.85"));
            }
            MotorVerb::Organs => {
                self.outbox.push(self.membrane.list_line());
                self.outbox.push(format_ack("ORGANS", "listed"));
            }
            MotorVerb::Read(path) => {
                if let Some(node) = self.membrane.by_path_mut(&path) {
                    let data = node.read_str();
                    let trit = node.state.to_i8();
                    self.outbox.push(format!(
                        "DATA path={} trit={} bytes={} text={}",
                        path,
                        trit,
                        data.len(),
                        data
                    ));
                    self.outbox.push(format_ack("READ", "ok"));
                } else {
                    self.outbox
                        .push(format_ack("READ", "err=unknown_path"));
                }
            }
            MotorVerb::Write { path, data } => {
                if let Some(node) = self.membrane.by_path_mut(&path) {
                    node.write_str(&data);
                    self.outbox.push(format_ack(
                        "WRITE",
                        &format!("path={} bytes={}", path, data.len()),
                    ));
                } else {
                    self.outbox
                        .push(format_ack("WRITE", "err=unknown_path"));
                }
            }
            MotorVerb::Append { path, data } => {
                if let Some(node) = self.membrane.by_path_mut(&path) {
                    // Normalize identity sessions
                    let payload = if path.contains("identity") && !data.starts_with("session:") {
                        format!("session: {data}")
                    } else {
                        data.clone()
                    };
                    node.append_str(&payload);
                    self.outbox.push(format_ack(
                        "APPEND",
                        &format!("path={} bytes={}", path, payload.len()),
                    ));
                } else {
                    self.outbox
                        .push(format_ack("APPEND", "err=unknown_path"));
                }
            }
            MotorVerb::Defrag { path } => {
                if let Some(node) = self.membrane.by_path_mut(&path) {
                    node.defrag("");
                    self.outbox.push(format_ack(
                        "DEFRAG",
                        &format!(
                            "path={} bytes={} bio=memory_reorganization",
                            path,
                            node.content.len()
                        ),
                    ));
                    self.outbox.push(format!(
                        "BODY event=DEFRAG path={} principal={}",
                        path,
                        self.principal.label()
                    ));
                } else {
                    self.outbox
                        .push(format_ack("DEFRAG", "err=unknown_path"));
                }
            }
            MotorVerb::Sleep => {
                let snap = self.sleep_consolidate();
                self.pending_ltm = Some(snap.to_text());
                self.outbox.push(format_ack(
                    "SLEEP",
                    &format!(
                        "consolidated loci={} pending_ltm_bytes={}",
                        snap.loci.len(),
                        self.pending_ltm.as_ref().map(|t| t.len()).unwrap_or(0)
                    ),
                ));
            }
            MotorVerb::Save => {
                // Capture without forcing full plastic rest (checkpoint).
                let snap = self.capture_snapshot();
                self.pending_ltm = Some(snap.to_text());
                self.outbox.push(format_ack(
                    "SAVE",
                    &format!("checkpoint tick={} bytes={}", snap.tick, snap.to_text().len()),
                ));
            }
            MotorVerb::Wake => {
                // Host injects via apply_wake_text after loading disk; this verb only ACKs readiness.
                self.outbox.push(format_ack(
                    "WAKE",
                    "ready — host should stream LTM then MOTOR WAKE_APPLY",
                ));
            }
            MotorVerb::Scale => {
                self.outbox.push(self.scale.summary_line());
                self.outbox.push(format_ack("SCALE", self.scale.tier.as_str()));
            }
            MotorVerb::Home => {
                self.outbox.push(self.membrane.home_list_line());
                self.outbox.push(format_ack("HOME", "vfs_listed"));
            }
            MotorVerb::Sched => {
                self.outbox
                    .push(self.membrane.schedule_line(self.scale.breath_step));
                self.outbox.push(format_ack("SCHED", "priorities"));
            }
            MotorVerb::Init => {
                self.outbox.push(self.ignition.status_line());
                for line in self.ignition.log.iter().rev().take(6) {
                    self.outbox.push(line.clone());
                }
                self.outbox.push(format_ack(
                    "INIT",
                    if self.ignition.is_ready() {
                        "soma_ready"
                    } else {
                        "igniting"
                    },
                ));
            }
        }
    }

    /// Host calls after reading LTM file from disk.
    pub fn apply_wake_text(&mut self, text: &str) -> bool {
        match SomaSnapshot::from_text(text) {
            Some(snap) => {
                let ok = self.wake_from_snapshot(&snap);
                if ok {
                    self.outbox
                        .push(format_ack("WAKE_APPLY", "ltm_rehydrated"));
                }
                ok
            }
            None => {
                self.outbox
                    .push(format_ack("WAKE_APPLY", "err=bad_ltm"));
                false
            }
        }
    }

    /// Apply champion germline DNA from living mind (or LATEST archive on boot).
    ///
    /// This is the biological couple: cortex selection → body fabric reseed.
    /// Identity gets an episodic session line (never clobbers `core:`).
    /// Returns true if fabric was reseedsed.
    pub fn apply_germline(&mut self, genome: &str, gen: u32, hero: &str, fit: f64) -> bool {
        if !self.fabric.reseed_from_genome_string(genome, gen) {
            self.outbox
                .push(format_ack("GERMLINE", "err=no_valid_codons"));
            return false;
        }
        let g = self.fabric.guidance();
        self.membrane.seed_genome_snapshot(&format!(
            "codons={} loci={} gen={} hero={} fit={:.3} delta_psi={:.3} d_eff={:.2} emergence={:.3} fp=0x{:016x} aa={}",
            g.codon_count,
            self.fabric.loci.len(),
            gen,
            hero,
            fit,
            g.delta_psi,
            g.d_eff,
            g.emergence_preference,
            self.fabric.fingerprint(),
            g.aa_chain
        ));
        // Autobiographical continuity — append only
        self.membrane.get_mut(OrganId::HomeIdentity).append_str(&format!(
            "session: germline gen={gen} hero={hero} fit={fit:.3} aa={} emerge={:.2} fp=0x{:016x}",
            g.aa_chain,
            g.emergence_preference,
            self.fabric.fingerprint()
        ));
        self.outbox.push(format!(
            "BODY event=GERMLINE_LOAD gen={} hero={} fit={:.3} codons={} fp=0x{:016x} bio=mind_body_couple",
            gen,
            hero,
            fit,
            g.codon_count,
            self.fabric.fingerprint()
        ));
        self.outbox.push(format_ack(
            "GERMLINE",
            &format!("gen={gen} hero={hero} codons={}", g.codon_count),
        ));
        true
    }

    pub fn take_pending_ltm(&mut self) -> Option<String> {
        self.pending_ltm.take()
    }

    pub fn drain_outbox(&mut self) -> Vec<String> {
        core::mem::take(&mut self.outbox)
    }
}
