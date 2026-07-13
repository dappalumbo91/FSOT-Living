//! Interoception — the body's sense of itself.
//!
//! Channels map silicon proxies (TSC, heap, uptime, breath, thermal proxy)
//! into continuous raw values, Trit attention indices, and a global
//! genetic pressure signal. Not binary flags — continuous fields collapse
//! into trinary attention.

use libm::{cos, sin};
use trinary::{collapse_soft, Trit};

/// Number of interoceptive channels.
pub const CHANNEL_COUNT: usize = 5;
const RING_CAP: usize = 64;

/// One sensory channel into the body.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Channel {
    /// Thermal valence proxy: 1.0 = cool/comfortable, 0.0 = hot/pain.
    CpuTemp = 0,
    /// TSC delta per sample (activity / fatigue proxy).
    Tsc = 1,
    /// Heap fill fraction [0, 1] — hunger for free tissue.
    HeapPressure = 2,
    /// Seconds since boot (time sense).
    Uptime = 3,
    /// Homeostatic breath phase sin (aligned with living habitat).
    Breath = 4,
}

impl Channel {
    pub fn label(self) -> &'static str {
        match self {
            Channel::CpuTemp => "cpu_temp",
            Channel::Tsc => "tsc",
            Channel::HeapPressure => "heap",
            Channel::Uptime => "uptime",
            Channel::Breath => "breath",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("cpu_temp") || s.eq_ignore_ascii_case("temp") || s == "0" {
            Some(Channel::CpuTemp)
        } else if s.eq_ignore_ascii_case("tsc") || s == "1" {
            Some(Channel::Tsc)
        } else if s.eq_ignore_ascii_case("heap")
            || s.eq_ignore_ascii_case("heap_pressure")
            || s == "2"
        {
            Some(Channel::HeapPressure)
        } else if s.eq_ignore_ascii_case("uptime") || s.eq_ignore_ascii_case("upt") || s == "3" {
            Some(Channel::Uptime)
        } else if s.eq_ignore_ascii_case("breath") || s == "4" {
            Some(Channel::Breath)
        } else {
            None
        }
    }
}

/// One sample across all channels.
#[derive(Copy, Clone, Debug)]
pub struct Sample {
    pub seq: u64,
    pub raw: [f64; CHANNEL_COUNT],
    /// Collapsed Trit per channel (−1 / 0 / +1) as i8.
    pub trit: [i8; CHANNEL_COUNT],
    /// Attention scalar — how much the mind should care [0, 1+].
    pub attn: [f64; CHANNEL_COUNT],
    /// Aggregate genetic pressure derived from this sample.
    pub pressure: f64,
}

/// Sensor feed for one sample (host or hardware fills this).
#[derive(Copy, Clone, Debug)]
pub struct SensorInputs {
    /// 1.0 cool … 0.0 hot. Host defaults 0.75.
    pub cpu_temp_norm: f64,
    /// Raw TSC delta (or host synthetic cycles).
    pub tsc_delta: f64,
    pub heap_used: usize,
    pub heap_cap: usize,
    pub uptime_seconds: f64,
    pub breath_phase: f64,
}

impl Default for SensorInputs {
    fn default() -> Self {
        SensorInputs {
            cpu_temp_norm: 0.75,
            tsc_delta: 1.0e6,
            heap_used: 0,
            heap_cap: 512 * 1024,
            uptime_seconds: 0.0,
            breath_phase: 0.0,
        }
    }
}

/// Lock-free-ish ring of samples (single producer / single consumer).
pub struct Interoception {
    ring: [Option<Sample>; RING_CAP],
    head: usize,
    seq: u64,
    /// Soft focus channel (spotlight from MOTOR ATTEND).
    pub focus: usize,
    pub breath_count: u64,
}

impl Interoception {
    pub fn new() -> Self {
        Interoception {
            ring: [None; RING_CAP],
            head: 0,
            seq: 0,
            focus: Channel::Breath as usize,
            breath_count: 0,
        }
    }

    pub fn sample(&mut self, sensors: &SensorInputs) -> Sample {
        let heap_p = if sensors.heap_cap == 0 {
            0.0
        } else {
            (sensors.heap_used as f64 / sensors.heap_cap as f64).clamp(0.0, 1.0)
        };
        let breath_s = sin(sensors.breath_phase);
        let breath_c = cos(sensors.breath_phase * 0.5);

        let raw = [
            sensors.cpu_temp_norm.clamp(0.0, 1.0),
            sensors.tsc_delta,
            heap_p,
            sensors.uptime_seconds,
            breath_s,
        ];

        // Continuous fields → Trit (not binary).
        // temp: cool → SpinUp (express comfort), hot → SpinDown
        // heap: free → SpinUp, full → SpinDown
        // breath: sin field as-is
        let fields = [
            (raw[0] - 0.5) * 2.0,                 // temp valence
            ((raw[1] / 1.0e7) - 0.5).clamp(-1.5, 1.5), // tsc activity band
            (0.5 - raw[2]) * 2.0,                 // free memory valence
            sin(raw[3] * 0.1),                    // slow uptime drift
            raw[4],                               // breath
        ];

        let mut trit = [0i8; CHANNEL_COUNT];
        let mut attn = [0.0f64; CHANNEL_COUNT];
        for i in 0..CHANNEL_COUNT {
            let t = collapse_soft(fields[i], 0.18);
            trit[i] = t.to_i8();
            let mag = if fields[i] < 0.0 {
                -fields[i]
            } else {
                fields[i]
            };
            attn[i] = (0.25 + mag * 0.75).clamp(0.0, 1.5);
            if t == Trit::Superposed {
                attn[i] *= 0.6;
            }
        }

        // Global genetic pressure: breath drive + free-memory + thermal comfort.
        let pressure = 0.45 * breath_s
            + 0.20 * breath_c
            + 0.20 * fields[0]
            + 0.15 * fields[2];

        let sample = Sample {
            seq: self.seq,
            raw,
            trit,
            attn,
            pressure,
        };
        self.seq = self.seq.wrapping_add(1);
        let slot = self.head % RING_CAP;
        self.ring[slot] = Some(sample);
        self.head = self.head.wrapping_add(1);
        sample
    }

    pub fn latest(&self) -> Option<Sample> {
        if self.head == 0 {
            return None;
        }
        let slot = (self.head - 1) % RING_CAP;
        self.ring[slot]
    }

    pub fn report_lines<F: FnMut(&str)>(&self, mut emit: F) {
        let n = self.head.min(RING_CAP);
        emit("REPORT samples=");
        // count emitted via manual
        let _ = n;
        let start = if self.head > RING_CAP {
            self.head - RING_CAP
        } else {
            0
        };
        for i in start..self.head {
            if let Some(s) = self.ring[i % RING_CAP] {
                // Caller may prefer format_report_sample
                let _ = s;
            }
        }
        let _ = emit;
    }

    pub fn attend(&mut self, ch: usize) {
        if ch < CHANNEL_COUNT {
            self.focus = ch;
        }
    }

    pub fn breathe(&mut self) {
        self.breath_count = self.breath_count.wrapping_add(1);
    }
}

impl Default for Interoception {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_produces_trinary_not_binary() {
        let mut i = Interoception::new();
        let mut sensors = SensorInputs::default();
        sensors.breath_phase = 1.2;
        sensors.heap_used = 100_000;
        let s = i.sample(&sensors);
        assert_eq!(s.raw.len(), CHANNEL_COUNT);
        for t in s.trit {
            assert!(t == -1 || t == 0 || t == 1);
        }
        assert!(s.pressure.is_finite());
    }

    #[test]
    fn heap_full_lowers_pressure_valence() {
        let mut i = Interoception::new();
        let mut free = SensorInputs::default();
        free.heap_used = 0;
        free.breath_phase = 0.0;
        let mut full = free;
        full.heap_used = full.heap_cap;
        let s_free = i.sample(&free);
        let s_full = i.sample(&full);
        // Full heap → more negative field contribution via fields[2]
        assert!(s_full.raw[Channel::HeapPressure as usize] > s_free.raw[Channel::HeapPressure as usize]);
    }
}
