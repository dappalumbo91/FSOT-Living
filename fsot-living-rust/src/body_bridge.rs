//! Cortex-side Mind ABI bridge — living habitat ↔ trinary body.
//!
//! ## Transports (prefer real-time TCP)
//!
//! | Transport | How | Latency | Role |
//! |-----------|-----|---------|------|
//! | **TCP** | connect `127.0.0.1:7368` | milliseconds | Live companion (preferred) |
//! | **Files** | `BODY_STATUS.json` / `BODY_MOTOR.txt` | tens–hundreds of ms + disk | Fallback, audit log |
//!
//! Same line protocol either way. Files are the "black box flight recorder";
//! TCP is the "nerve" — closer to how a real OS talks to a companion process.
//!
//! Env:
//!   FSOT_BODY_TCP=127.0.0.1:7368   (default host mind_bridge; empty/off = TCP disabled)
//!   FSOT_BODY_QEMU_TCP=127.0.0.1:7370  optional QEMU serial-chardev nerve (OS path)
//!   FSOT_BODY_TCP=off              disable TCP, files only
//!
//! Nerve priority: host TCP (7368) → QEMU serial TCP (7370) → files.

use crate::export::export_dir;
use crate::habitat::Habitat;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BodyStatus {
    #[serde(default)]
    pub tick: u64,
    #[serde(default)]
    pub genome_fp: String,
    #[serde(default)]
    pub delta_psi: f64,
    #[serde(default)]
    pub d_eff: f64,
    #[serde(default)]
    pub emergence: f64,
    #[serde(default)]
    pub repress: u64,
    #[serde(default)]
    pub latent: u64,
    #[serde(default)]
    pub express: u64,
    #[serde(default)]
    pub mean_expr: f64,
    #[serde(default)]
    pub plasticity: f64,
    #[serde(default)]
    pub breath_count: u64,
    #[serde(default)]
    pub focus: u64,
    #[serde(default)]
    pub motor_bias: f64,
    #[serde(default)]
    pub organs: String,
    #[serde(default)]
    pub last_mind: String,
}

/// Snapshot kept on the habitat for UI / LIVE_STATUS.
#[derive(Debug, Clone, Default)]
pub struct BodyLink {
    pub online: bool,
    pub status: BodyStatus,
    pub motors_sent: u64,
    pub last_motor: String,
    pub last_error: String,
    pub couple_pressure: f64,
    /// "tcp" | "file" | "off"
    pub transport: String,
    /// Last generation we archived to body /home/genome
    pub last_genome_gen: u32,
    /// Fingerprint of tools last pushed to /home/tools
    pub tools_fp: String,
    /// Last proposal count pushed
    pub last_proposals_n: usize,
}

// ─── TCP session (lazy, reconnecting) ───────────────────────────────────────

struct TcpSession {
    stream: Option<TcpStream>,
    rx_buf: Vec<u8>,
    last_connect_attempt_ms: u128,
    addr: String,
}

impl TcpSession {
    fn new() -> Self {
        let addr = match std::env::var("FSOT_BODY_TCP") {
            Ok(s) if s.eq_ignore_ascii_case("off") || s.is_empty() => String::new(),
            Ok(s) => s,
            Err(_) => "127.0.0.1:7368".into(),
        };
        TcpSession {
            stream: None,
            rx_buf: Vec::with_capacity(4096),
            last_connect_attempt_ms: 0,
            addr,
        }
    }

    /// QEMU serial chardev socket (OS nerve rehearsal).
    fn new_qemu() -> Self {
        let addr = match std::env::var("FSOT_BODY_QEMU_TCP") {
            Ok(s) if s.eq_ignore_ascii_case("off") || s.is_empty() => String::new(),
            Ok(s) => s,
            Err(_) => "127.0.0.1:7370".into(),
        };
        TcpSession {
            stream: None,
            rx_buf: Vec::with_capacity(4096),
            last_connect_attempt_ms: 0,
            addr,
        }
    }

    fn enabled(&self) -> bool {
        !self.addr.is_empty()
    }

    fn now_ms() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }

    fn ensure_connected(&mut self) -> bool {
        if !self.enabled() {
            return false;
        }
        if self.stream.is_some() {
            return true;
        }
        let now = Self::now_ms();
        // Reconnect at most every 750ms — still gentle on 2012 Mini, snappier on Omen
        if now.saturating_sub(self.last_connect_attempt_ms) < 750 {
            return false;
        }
        self.last_connect_attempt_ms = now;
        let sock: std::net::SocketAddr = match self.addr.parse() {
            Ok(a) => a,
            Err(_) => match "127.0.0.1:7368".parse() {
                Ok(a) => a,
                Err(_) => return false,
            },
        };
        // Longer connect timeout: Windows localhost can stall under load
        match TcpStream::connect_timeout(&sock, Duration::from_millis(400)) {
            Ok(s) => {
                let _ = s.set_nodelay(true);
                let _ = s.set_nonblocking(true);
                let _ = s.set_read_timeout(Some(Duration::from_millis(2)));
                let _ = s.set_write_timeout(Some(Duration::from_millis(250)));
                self.stream = Some(s);
                true
            }
            Err(_) => false,
        }
    }

    fn poll_lines(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        if !self.ensure_connected() {
            return out;
        }
        let Some(stream) = self.stream.as_mut() else {
            return out;
        };
        let mut tmp = [0u8; 2048];
        // Burst up to 8 non-blocking reads per habitat tick (keeps 2012 Mini light)
        for _ in 0..8 {
            match stream.read(&mut tmp) {
                Ok(0) => {
                    self.stream = None;
                    break;
                }
                Ok(n) => {
                    self.rx_buf.extend_from_slice(&tmp[..n]);
                    if self.rx_buf.len() > 64 * 1024 {
                        self.rx_buf.drain(..self.rx_buf.len() / 2);
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(_) => {
                    self.stream = None;
                    break;
                }
            }
        }
        while let Some(pos) = self.rx_buf.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.rx_buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if !line.is_empty() {
                out.push(line);
            }
        }
        out
    }

    fn send_line(&mut self, line: &str) -> bool {
        if !self.ensure_connected() {
            return false;
        }
        let payload = format!("{line}\n");
        // First attempt
        if self.write_payload(&payload) {
            return true;
        }
        // Drop dead socket, reconnect once, retry (nerve fiber re-fire)
        self.stream = None;
        if !self.ensure_connected() {
            return false;
        }
        self.write_payload(&payload)
    }

    fn write_payload(&mut self, payload: &str) -> bool {
        let Some(stream) = self.stream.as_mut() else {
            return false;
        };
        match stream.write_all(payload.as_bytes()).and_then(|_| stream.flush()) {
            Ok(()) => true,
            Err(_) => {
                self.stream = None;
                false
            }
        }
    }
}

static TCP: Mutex<Option<TcpSession>> = Mutex::new(None);
static TCP_QEMU: Mutex<Option<TcpSession>> = Mutex::new(None);

fn with_tcp<F, R>(f: F) -> R
where
    F: FnOnce(&mut TcpSession) -> R,
{
    let mut guard = TCP.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(TcpSession::new());
    }
    f(guard.as_mut().unwrap())
}

fn with_qemu_tcp<F, R>(f: F) -> R
where
    F: FnOnce(&mut TcpSession) -> R,
{
    let mut guard = TCP_QEMU.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(TcpSession::new_qemu());
    }
    f(guard.as_mut().unwrap())
}

// ─── File paths ─────────────────────────────────────────────────────────────

fn body_status_path() -> PathBuf {
    export_dir().join("BODY_STATUS.json")
}

fn body_motor_path() -> PathBuf {
    export_dir().join("BODY_MOTOR.txt")
}

/// Read latest body status if present (file).
pub fn read_body_status() -> Option<BodyStatus> {
    let path = body_status_path();
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Append MOTOR line for file transport.
pub fn write_motor(line: &str) -> std::io::Result<()> {
    let path = body_motor_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{line}")?;
    Ok(())
}

fn status_is_fresh(max_age_secs: u64) -> bool {
    let path = body_status_path();
    let Ok(meta) = fs::metadata(&path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    modified
        .elapsed()
        .map(|d| d.as_secs() <= max_age_secs)
        .unwrap_or(false)
}

/// Parse a MIND telemetry line into BodyStatus fields (merge onto existing).
pub fn apply_mind_line(st: &mut BodyStatus, line: &str) {
    if !line.starts_with("MIND") {
        st.last_mind = line.to_string();
        return;
    }
    st.last_mind = line.to_string();
    for part in line.split_whitespace().skip(1) {
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        match k {
            "seq" => st.tick = v.parse().unwrap_or(st.tick),
            "focus" => st.focus = v.parse().unwrap_or(st.focus),
            "breath" => st.breath_count = v.parse().unwrap_or(st.breath_count),
            "repress" => st.repress = v.parse().unwrap_or(st.repress),
            "latent" => st.latent = v.parse().unwrap_or(st.latent),
            "express" => st.express = v.parse().unwrap_or(st.express),
            "mean_expr" => st.mean_expr = v.parse().unwrap_or(st.mean_expr),
            "plast" => st.plasticity = v.parse().unwrap_or(st.plasticity),
            "pressure" => {
                // map pressure into motor_bias soft field for couple formula
                if let Ok(p) = v.parse::<f64>() {
                    st.motor_bias = st.motor_bias * 0.5 + p * 0.15;
                }
            }
            "genome_fp" => st.genome_fp = v.to_string(),
            "delta_psi" | "dpsi" => st.delta_psi = v.parse().unwrap_or(st.delta_psi),
            "d_eff" | "deff" => st.d_eff = v.parse().unwrap_or(st.d_eff),
            "emergence" | "emerge" => st.emergence = v.parse().unwrap_or(st.emergence),
            _ => {}
        }
    }
    // If we saw phenotype tallies but not plast, keep prior
    if st.express + st.repress + st.latent > 0 && st.plasticity == 0.0 {
        st.plasticity = 0.5;
    }
}

fn emit_motor(h: &mut Habitat, line: &str, prefer_tcp: bool) {
    // Constraint membrane: rate + identity WRITE rails
    match h.constraints.gate_motor(h.tick, line) {
        crate::constraints::Gate::Deny | crate::constraints::Gate::Throttle => {
            h.body.last_error = format!("motor gated: {}", h.constraints.last_gate_code);
            return;
        }
        crate::constraints::Gate::AllowWatch | crate::constraints::Gate::Allow => {}
    }
    // Triple transport: host TCP → QEMU serial TCP → file flight recorder.
    let mut tcp_ok = false;
    if prefer_tcp {
        tcp_ok = with_tcp(|tcp| tcp.send_line(line));
        if !tcp_ok {
            // OS nerve: bare-metal body on QEMU chardev socket
            if with_qemu_tcp(|tcp| tcp.send_line(line)) {
                tcp_ok = true;
                h.body.transport = "qemu_serial".into();
            }
        }
    }
    let file_ok = write_motor(line).is_ok();
    let ok = tcp_ok || file_ok;
    if tcp_ok && h.body.transport != "qemu_serial" {
        h.body.transport = "tcp".into();
    } else if !tcp_ok && file_ok {
        h.body.transport = "file".into();
    }
    if ok {
        h.body.motors_sent += 1;
        h.body.last_motor = line.to_string();
        if !tcp_ok && prefer_tcp {
            h.body.last_error = "tcp missed — file mirror ok".into();
        } else if tcp_ok {
            h.body.last_error.clear();
        }
    } else {
        h.body.last_error = "motor emit failed (tcp+file)".into();
    }
}

/// One cortex↔body cycle. Call every habitat tick.
pub fn couple_body(h: &mut Habitat) {
    let mut got_tcp = false;

    // 1) Poll host TCP MIND lines (mind_bridge nerve)
    let lines = with_tcp(|tcp| {
        if !tcp.enabled() {
            return Vec::new();
        }
        tcp.poll_lines()
    });
    for line in lines {
        got_tcp = true;
        apply_mind_line(&mut h.body.status, &line);
        if line.starts_with("ORGANS") {
            h.body.status.organs = line.clone();
        }
    }
    // 1b) QEMU serial TCP (OS nerve) if host quiet
    if !got_tcp {
        let qlines = with_qemu_tcp(|tcp| {
            if !tcp.enabled() {
                return Vec::new();
            }
            tcp.poll_lines()
        });
        for line in qlines {
            got_tcp = true;
            apply_mind_line(&mut h.body.status, &line);
            if line.starts_with("ORGANS") {
                h.body.status.organs = line.clone();
            }
            if h.body.transport != "qemu_serial" {
                h.body.transport = "qemu_serial".into();
            }
        }
    }
    if got_tcp {
        h.body.online = true;
        if h.body.transport.is_empty() || h.body.transport == "file" {
            h.body.transport = "tcp".into();
        }
        h.body.last_error.clear();
    } else {
        // 2) File fallback
        match read_body_status() {
            Some(st) => {
                h.body.online = true;
                h.body.status = st;
                if h.body.transport.is_empty() || h.body.transport == "off" {
                    h.body.transport = "file".into();
                }
                h.body.last_error.clear();
            }
            None => {
                if !got_tcp {
                    h.body.online = false;
                    h.body.couple_pressure *= 0.95;
                    if h.body.transport.is_empty() {
                        h.body.transport = "off".into();
                    }
                    return;
                }
            }
        }
    }

    let st = h.body.status.clone();
    let loci = (st.repress + st.latent + st.express).max(1) as f64;
    let express_frac = st.express as f64 / loci;
    let repress_frac = st.repress as f64 / loci;
    let body_p = (express_frac - repress_frac) * 0.35 + st.motor_bias * 0.15;
    h.body.couple_pressure = body_p;

    for o in &mut h.pack {
        o.pressure = (o.pressure * 0.92 + body_p * 0.18).clamp(-1.5, 1.5);
        if st.delta_psi > 0.0 {
            o.delta_psi = o.delta_psi * 0.97 + st.delta_psi * 0.03;
        }
        if st.d_eff > 0.0 {
            o.d_eff = o.d_eff * 0.98 + st.d_eff * 0.02;
        }
    }

    // MOTOR only if live TCP or fresh file status
    let live = h.body.transport == "tcp" || status_is_fresh(180);
    if !live {
        return;
    }
    if h.tick % 25 != 0 {
        return;
    }

    let prefer_tcp = h.body.transport == "tcp" || with_tcp(|t| t.enabled() && t.stream.is_some());

    let mut cmds: Vec<String> = Vec::new();
    cmds.push("MOTOR STATUS".into());

    if express_frac > 0.72 && st.plasticity > 0.4 {
        cmds.push("MOTOR BREATHE".into());
    } else if repress_frac > 0.65 {
        cmds.push("MOTOR EXPRESS".into());
    } else if h.perceived.drive > 0.55 {
        cmds.push("MOTOR EXPRESS".into());
    } else if h.perceived.phase_cos < -0.4 {
        cmds.push("MOTOR BREATHE".into());
    }

    if h.tick % 50 == 0 {
        let ch = if h.perceived.drive > 0.5 {
            "breath"
        } else if h.hero().pressure > 0.4 {
            "heap"
        } else {
            "temp"
        };
        cmds.push(format!("MOTOR ATTEND {ch}"));
    }

    if h.tick % 40 == 0 {
        let hero = h.hero();
        let note = format!(
            "living t={} gen={} hero={} S={:.3} fit={:.2} acc={:.0}% drive={:.2}",
            h.tick,
            h.generation,
            hero.id,
            hero.s,
            hero.fitness_score(),
            hero.cognitive_accuracy * 100.0,
            h.perceived.drive
        );
        let safe: String = note.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
        cmds.push(format!("MOTOR WRITE /organs/bridge {safe}"));
        cmds.push(format!(
            "MOTOR WRITE /organs/knowledge living_pack fit={:.2} lang={:.0}% body_couple={:.3}",
            hero.fitness_score(),
            hero.language_accuracy * 100.0,
            body_p
        ));
        // Biological home LTM encode (Linux $HOME salvage)
        // Identity rewrite requires Human principal — Mind skips (cap_denied otherwise)
        // Knowledge LTM is Mind-authorized.
        cmds.push(format!(
            "MOTOR WRITE /home/knowledge fit={:.2} lang={:.0}% cog={:.0}% couple={:.3}",
            hero.fitness_score(),
            hero.language_accuracy * 100.0,
            hero.cognitive_accuracy * 100.0,
            body_p
        ));
    }
    if h.tick % 100 == 0 {
        cmds.push("MOTOR HOME".into());
        cmds.push("MOTOR SCHED".into());
        cmds.push("MOTOR INIT".into());
    }

    // —— Genome writeback (breeding → body germline archive) ——
    if h.generation > 0 && h.generation != h.body.last_genome_gen {
        let hero = h.hero();
        let genome = hero.genome.join("-");
        let safe_g: String = genome.chars().take(200).collect();
        let hero_id = hero.id.clone();
        let fit = hero.fitness_score();
        let breed = hero.breeding_value();
        let tier = h.scale.tier.clone();
        let gen = h.generation;
        cmds.push(format!(
            "MOTOR WRITE /home/genome gen={gen} hero={hero_id} fit={fit:.3} genome={safe_g}"
        ));
        // Identity: session append only (no waking DEFRAG — dream runs inside SLEEP)
        cmds.push(format!(
            "MOTOR APPEND /home/identity session: gen={gen} champion={hero_id} breed={breed:.2} scale={tier}"
        ));
        // SAVE = cold checkpoint; SLEEP = rest + FSOT dream weave + LTM
        cmds.push("MOTOR SAVE".into());
        cmds.push("MOTOR SLEEP".into());
        h.body.last_genome_gen = gen;
        h.push(format!(
            "body genome writeback gen={gen} + session append + SAVE + SLEEP(dream)"
        ));
    }

    // —— Procedural skills → home/tools (Linux ~/bin salvage) ——
    let tools_owned: Vec<String> = h.hero().tools_owned.clone();
    let js_tools: Vec<String> = h.hero().js_tools_owned.clone();
    let hero_id = h.hero().id.clone();
    let tools_fp = tools_owned.join(",");
    if !tools_fp.is_empty() && tools_fp != h.body.tools_fp && h.tick % 30 == 0 {
        for t in tools_owned.iter().take(12) {
            cmds.push(format!("MOTOR WRITE /home/tools skill={t}"));
        }
        for t in js_tools.iter().take(6) {
            cmds.push(format!("MOTOR WRITE /home/tools js_skill={t}"));
        }
        h.body.tools_fp = tools_fp;
    }

    // —— Proposals → home/proposals (novelty under selection) ——
    let pn = crate::proposals::count_pending();
    if pn > h.body.last_proposals_n && h.tick % 50 == 0 {
        cmds.push(format!(
            "MOTOR WRITE /home/proposals pending={pn} hero={hero_id}"
        ));
        h.body.last_proposals_n = pn;
    }

    for c in cmds {
        emit_motor(h, &c, prefer_tcp);
    }

    // Refresh felt interoception on pack when body online (self-model + dialogue)
    if h.body.online {
        let felt = felt_interoception(h);
        let ph = {
            let e = h.body.status.express;
            let l = h.body.status.latent;
            let r = h.body.status.repress;
            if e == 0 && l == 0 && r == 0 {
                "latent"
            } else if e >= l && e >= r {
                "express"
            } else if l >= e && l >= r {
                "latent"
            } else {
                "repress"
            }
        };
        for o in &mut h.pack {
            o.felt_body = felt.clone();
            o.body_link_seen = true;
            o.known_body_phenotype = Some(ph.into());
        }
    }

    if h.tick % 100 == 0 && h.body.online {
        h.push(format!(
            "body[{}] tick={} r/l/e={}/{}/{} couple={:.3} motors={}",
            h.body.transport,
            st.tick,
            st.repress,
            st.latent,
            st.express,
            body_p,
            h.body.motors_sent
        ));
    }
}

/// First-person-ish interoceptive report (felt body, not raw telemetry dump).
pub fn felt_interoception(h: &Habitat) -> String {
    if !h.body.online {
        return "Body quiet — no chassis pulse yet (start mind_bridge --tcp or QEMU serial nerve)."
            .into();
    }
    let s = &h.body.status;
    let total = (s.repress + s.latent + s.express).max(1) as f64;
    let r = s.repress as f64 / total;
    let l = s.latent as f64 / total;
    let e = s.express as f64 / total;
    let breath = if s.breath_count > 0 {
        format!("I have taken {} breaths with the chassis.", s.breath_count)
    } else {
        "Breath counter is still cold.".into()
    };
    let tone = if e > 0.45 {
        "I feel outward expression — motor bias leaning into the world."
    } else if r > 0.45 {
        "I feel repression — holding pattern, conserving."
    } else if l > 0.55 {
        "I feel mostly latent — superposed, waiting for pressure to collapse phenotype."
    } else {
        "I feel a mixed trinary balance across loci."
    };
    let couple = if h.body.couple_pressure.abs() < 0.05 {
        "Mind–body couple is soft right now."
    } else if h.body.couple_pressure > 0.0 {
        "I feel positive couple pressure from the body loop."
    } else {
        "I feel negative couple pressure — body is damping me a little."
    };
    let emerge = if s.emergence > 0.55 {
        "Body emergence preference is high."
    } else if s.emergence < 0.35 {
        "Body emergence preference is low (more damping)."
    } else {
        "Body emergence preference is mid-range."
    };
    format!(
        "{tone} {breath}\n\
         Felt mix: express {e:.0}% · latent {l:.0}% · repress {r:.0}% · mean_expr={:.2} plast={:.2}.\n\
         {couple} {emerge} δψ={:.3} D_eff={:.1} via {} tick={} fp={}.",
        s.mean_expr,
        s.plasticity,
        s.delta_psi,
        s.d_eff,
        if h.body.transport.is_empty() {
            "?"
        } else {
            h.body.transport.as_str()
        },
        s.tick,
        if s.genome_fp.is_empty() {
            "?"
        } else {
            s.genome_fp.as_str()
        },
    )
}

/// Human-readable body summary for dialogue / status line.
pub fn body_summary(h: &Habitat) -> String {
    if !h.body.online {
        return "Body offline — start: cargo run -p mind_bridge -- --tcp  (or file mode / QEMU serial). Ask again after body is up.".into();
    }
    let s = &h.body.status;
    format!(
        "{}\n\nChassis detail: via {} · tick={} genome={} repress={} latent={} express={} expr={:.2} plast={:.2} couple={:.3} motors={} last_motor={}",
        felt_interoception(h),
        if h.body.transport.is_empty() {
            "?"
        } else {
            h.body.transport.as_str()
        },
        s.tick,
        if s.genome_fp.is_empty() {
            "?"
        } else {
            s.genome_fp.as_str()
        },
        s.repress,
        s.latent,
        s.express,
        s.mean_expr,
        s.plasticity,
        h.body.couple_pressure,
        h.body.motors_sent,
        if h.body.last_motor.is_empty() {
            "(none)"
        } else {
            h.body.last_motor.as_str()
        }
    )
}
