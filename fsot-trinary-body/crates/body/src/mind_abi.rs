//! Mind ABI — serial wire protocol between body (kernel) and cortex (host / living habitat).
//!
//! ```text
//! body → cortex : MIND key=value ...
//! cortex → body : MOTOR <verb> [args]
//! body → cortex : ACK verb=... detail=...
//! ```
//!
//! Verbs are small and observable. Binary opaque blobs are rejected —
//! trinary state is carried as trit indices and continuous fields.

use alloc::format;
use alloc::string::{String, ToString};

use crate::interoception::{Channel, Sample, CHANNEL_COUNT};
use genetic::GeneticFabric;

const MAX_LINE: usize = 256;
const MAX_NOTES: usize = 8;
const NOTE_LEN: usize = 96;

/// Motor command from cortex → body.
#[derive(Clone, Debug, PartialEq)]
pub enum MotorVerb {
    Status,
    Attend(usize),
    Breathe,
    Report,
    Note(String),
    /// Force positive expression pressure for N ticks.
    Express,
    /// Force repression pressure for N ticks.
    Repress,
    /// Dump organ membrane listing.
    Organs,
    /// Read a membrane path: MOTOR READ /body/genome
    Read(String),
    /// Write note into organ: MOTOR WRITE /organs/knowledge <text>
    /// Full replace — dangerous on /home/identity (Human only for that path).
    Write { path: String, data: String },
    /// Append without clobber: MOTOR APPEND /home/identity session:…
    Append { path: String, data: String },
    /// Reorganize/consolidate organ content without wiping core identity.
    /// MOTOR DEFRAG /home/identity
    Defrag { path: String },
    /// Sleep: plastic rest + consolidate working set → LTM snapshot
    Sleep,
    /// Save: checkpoint without full rest
    Save,
    /// Wake: host will apply LTM (see apply_wake_text)
    Wake,
    /// Report scale tier (goldfish tank size)
    Scale,
    /// List /home/* VFS (biological long-term compartments)
    Home,
    /// Organ scheduler priorities (homeostasis, not CFS)
    Sched,
    /// Report ignition / init service graph status
    Init,
}

/// Line buffer for assembling MOTOR commands from serial bytes.
pub struct CommandRx {
    buf: [u8; MAX_LINE],
    len: usize,
    notes: [[u8; NOTE_LEN]; MAX_NOTES],
    note_head: usize,
}

impl CommandRx {
    pub fn new() -> Self {
        CommandRx {
            buf: [0; MAX_LINE],
            len: 0,
            notes: [[0; NOTE_LEN]; MAX_NOTES],
            note_head: 0,
        }
    }

    /// Feed one byte; returns a verb when a full line arrives.
    pub fn push_byte(&mut self, byte: u8) -> Option<MotorVerb> {
        if byte == b'\r' {
            return None;
        }
        if byte == b'\n' {
            let line = core::str::from_utf8(&self.buf[..self.len]).unwrap_or("");
            let verb = parse_line(line, &mut self.notes, &mut self.note_head);
            self.len = 0;
            return verb;
        }
        if self.len < MAX_LINE {
            self.buf[self.len] = byte;
            self.len += 1;
        } else {
            self.len = 0; // resync
        }
        None
    }

    /// Host-side convenience: feed a whole line (with or without newline).
    pub fn push_line(&mut self, line: &str) -> Option<MotorVerb> {
        for b in line.bytes() {
            if let Some(v) = self.push_byte(b) {
                return Some(v);
            }
        }
        self.push_byte(b'\n')
    }
}

impl Default for CommandRx {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_line(
    line: &str,
    notes: &mut [[u8; NOTE_LEN]; MAX_NOTES],
    note_head: &mut usize,
) -> Option<MotorVerb> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let (head, rest) = match line.find(' ') {
        Some(i) => (&line[..i], line[i + 1..].trim()),
        None => (line, ""),
    };
    if !head.eq_ignore_ascii_case("MOTOR") {
        return None;
    }
    let (verb, arg) = match rest.find(' ') {
        Some(i) => (&rest[..i], rest[i + 1..].trim()),
        None => (rest, ""),
    };
    if verb.eq_ignore_ascii_case("STATUS") {
        return Some(MotorVerb::Status);
    }
    if verb.eq_ignore_ascii_case("BREATHE") {
        return Some(MotorVerb::Breathe);
    }
    if verb.eq_ignore_ascii_case("REPORT") {
        return Some(MotorVerb::Report);
    }
    if verb.eq_ignore_ascii_case("EXPRESS") {
        return Some(MotorVerb::Express);
    }
    if verb.eq_ignore_ascii_case("REPRESS") {
        return Some(MotorVerb::Repress);
    }
    if verb.eq_ignore_ascii_case("ORGANS") {
        return Some(MotorVerb::Organs);
    }
    if verb.eq_ignore_ascii_case("ATTEND") {
        return Channel::from_name(arg).map(|c| MotorVerb::Attend(c as usize));
    }
    if verb.eq_ignore_ascii_case("NOTE") {
        let slot = *note_head % MAX_NOTES;
        *note_head = note_head.wrapping_add(1);
        let bytes = arg.as_bytes();
        let n = bytes.len().min(NOTE_LEN - 1);
        notes[slot][..n].copy_from_slice(&bytes[..n]);
        notes[slot][n] = 0;
        return Some(MotorVerb::Note(String::from(arg)));
    }
    if verb.eq_ignore_ascii_case("READ") {
        return Some(MotorVerb::Read(String::from(arg)));
    }
    if verb.eq_ignore_ascii_case("WRITE") {
        // MOTOR WRITE /path rest of line
        let (path, data) = match arg.find(' ') {
            Some(i) => (arg[..i].to_string(), arg[i + 1..].trim().to_string()),
            None => (arg.to_string(), String::new()),
        };
        return Some(MotorVerb::Write { path, data });
    }
    if verb.eq_ignore_ascii_case("APPEND") {
        let (path, data) = match arg.find(' ') {
            Some(i) => (arg[..i].to_string(), arg[i + 1..].trim().to_string()),
            None => (arg.to_string(), String::new()),
        };
        return Some(MotorVerb::Append { path, data });
    }
    if verb.eq_ignore_ascii_case("DEFRAG") {
        let path = if arg.is_empty() {
            String::from("/home/identity")
        } else {
            String::from(arg.split_whitespace().next().unwrap_or("/home/identity"))
        };
        return Some(MotorVerb::Defrag { path });
    }
    if verb.eq_ignore_ascii_case("SLEEP") {
        return Some(MotorVerb::Sleep);
    }
    if verb.eq_ignore_ascii_case("SAVE") {
        return Some(MotorVerb::Save);
    }
    if verb.eq_ignore_ascii_case("WAKE") {
        return Some(MotorVerb::Wake);
    }
    if verb.eq_ignore_ascii_case("SCALE") {
        return Some(MotorVerb::Scale);
    }
    if verb.eq_ignore_ascii_case("HOME") {
        return Some(MotorVerb::Home);
    }
    if verb.eq_ignore_ascii_case("SCHED") || verb.eq_ignore_ascii_case("SCHEDULE") {
        return Some(MotorVerb::Sched);
    }
    if verb.eq_ignore_ascii_case("INIT") || verb.eq_ignore_ascii_case("IGNITION") {
        return Some(MotorVerb::Init);
    }
    None
}

/// Emit one MIND telemetry frame as a single line.
pub fn format_mind(
    sample: &Sample,
    focus: usize,
    breath_count: u64,
    fabric: &GeneticFabric,
) -> String {
    let (down, mid, up) = fabric.phenotype_tally();
    format!(
        "MIND seq={} focus={} breath={} \
         temp_raw={:.4} tsc_raw={:.3e} heap_raw={:.4} upt_raw={:.4} breath_raw={:.4} \
         temp_trit={} tsc_trit={} heap_trit={} upt_trit={} breath_trit={} \
         temp_attn={:.3} heap_attn={:.3} breath_attn={:.3} \
         pressure={:.4} repress={} latent={} express={} mean_expr={:.3} plast={:.3} \
         genome_fp=0x{:016x}",
        sample.seq,
        focus,
        breath_count,
        sample.raw[0],
        sample.raw[1],
        sample.raw[2],
        sample.raw[3],
        sample.raw[4],
        sample.trit[0],
        sample.trit[1],
        sample.trit[2],
        sample.trit[3],
        sample.trit[4],
        sample.attn[0],
        sample.attn[2],
        sample.attn[4],
        sample.pressure,
        down,
        mid,
        up,
        fabric.mean_expression(),
        fabric.mean_plasticity(),
        fabric.fingerprint(),
    )
}

pub fn format_ack(verb: &str, detail: &str) -> String {
    format!("ACK verb={} detail={}", verb, detail)
}

pub fn format_report(sample: &Sample, focus: usize) -> String {
    let mut s = format!(
        "REPORT seq={} focus={} pressure={:.4}",
        sample.seq, focus, sample.pressure
    );
    for i in 0..CHANNEL_COUNT {
        let label = match i {
            0 => "temp",
            1 => "tsc",
            2 => "heap",
            3 => "upt",
            _ => "breath",
        };
        s.push_str(&format!(
            " {}={{raw={:.4},trit={},attn={:.3}}}",
            label, sample.raw[i], sample.trit[i], sample.attn[i]
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status() {
        let mut notes = [[0u8; NOTE_LEN]; MAX_NOTES];
        let mut head = 0usize;
        assert_eq!(
            parse_line("MOTOR STATUS", &mut notes, &mut head),
            Some(MotorVerb::Status)
        );
    }

    #[test]
    fn parse_attend_breath() {
        let mut notes = [[0u8; NOTE_LEN]; MAX_NOTES];
        let mut head = 0usize;
        assert_eq!(
            parse_line("MOTOR ATTEND breath", &mut notes, &mut head),
            Some(MotorVerb::Attend(Channel::Breath as usize))
        );
    }

    #[test]
    fn command_rx_line() {
        let mut rx = CommandRx::new();
        assert_eq!(rx.push_line("MOTOR BREATHE"), Some(MotorVerb::Breathe));
    }

    #[test]
    fn parse_write() {
        let mut notes = [[0u8; NOTE_LEN]; MAX_NOTES];
        let mut head = 0usize;
        match parse_line(
            "MOTOR WRITE /organs/knowledge hello body",
            &mut notes,
            &mut head,
        ) {
            Some(MotorVerb::Write { path, data }) => {
                assert_eq!(path, "/organs/knowledge");
                assert_eq!(data, "hello body");
            }
            _ => panic!("expected Write"),
        }
    }
}
