//! Hardware environment probe — goldfish rule for the living mind gym.
//!
//! Measures RAM / CPU / optional NVIDIA VRAM and builds a [`body`-compatible]
//! scale profile so the pack grows into the tank (2012 Mini → lean; Omen 35L
//! + RTX 5070 → expanded/max).

use std::process::Command;
use std::thread;

/// Mirrors body::ScaleProfile numbers (kept local so living stays independent).
#[derive(Clone, Debug)]
pub struct HostScale {
    pub tier: String,
    pub ram_mb: u64,
    pub vram_mb: u64,
    pub cpu_cores: u32,
    pub has_discrete_gpu: bool,
    pub breath_step: f64,
    pub pop_size: usize,
    pub elite: usize,
    pub knowledge_cap: usize,
    pub audio_index_cap: usize,
    pub video_index_cap: usize,
    pub exam_every: u64,
    pub feed_every: u64,
    pub body_couple_gain: f64,
    pub consolidate_every: u64,
}

impl HostScale {
    pub fn summary(&self) -> String {
        format!(
            "SCALE tier={} ram_mb={} vram_mb={} cores={} gpu={} pop={} knowledge_cap={} couple={:.2}",
            self.tier,
            self.ram_mb,
            self.vram_mb,
            self.cpu_cores,
            self.has_discrete_gpu as u8,
            self.pop_size,
            self.knowledge_cap,
            self.body_couple_gain
        )
    }
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
}

fn probe_cpu_cores() -> u32 {
    thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

fn probe_ram_mb() -> u64 {
    if let Ok(s) = std::env::var("FSOT_RAM_MB") {
        if let Ok(v) = s.parse() {
            return v;
        }
    }
    #[cfg(target_os = "windows")]
    {
        // Modern Windows removed `wmic`. Prefer PowerShell CIM (bytes → MiB).
        // Omen 35L example: TotalPhysicalMemory ≈ 33923244032 → ~32352 MiB.
        if let Ok(out) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
            ])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Ok(bytes) = text.trim().parse::<u64>() {
                    if bytes > 0 {
                        return (bytes / (1024 * 1024)).max(1);
                    }
                }
            }
        }
        // Fallback: systeminfo parse ("Total Physical Memory: 32,352 MB")
        if let Ok(out) = Command::new("systeminfo").output() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("Total Physical Memory") {
                    let digits: String = line
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(mb) = digits.parse::<u64>() {
                        if mb > 512 {
                            return mb;
                        }
                    }
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(text) = std::fs::read_to_string("/proc/meminfo") {
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb: u64 = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    if kb > 0 {
                        return kb / 1024;
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
        {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if let Ok(bytes) = s.trim().parse::<u64>() {
                    return (bytes / (1024 * 1024)).max(1);
                }
            }
        }
    }
    8192 // assume 8GB if unknown
}

fn probe_vram_mb() -> u64 {
    if let Ok(s) = std::env::var("FSOT_VRAM_MB") {
        if let Ok(v) = s.parse() {
            return v;
        }
    }
    // NVIDIA — works on Omen + 5070 when drivers present
    if let Ok(out) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            // Multi-GPU: take max
            let mut best = 0u64;
            for line in text.lines() {
                if let Ok(mb) = line.trim().parse::<u64>() {
                    best = best.max(mb);
                }
            }
            if best > 0 {
                return best;
            }
        }
    }
    0
}

fn tier_from_resources(ram_mb: u64, vram_mb: u64, cpu_cores: u32) -> &'static str {
    let has_gpu = vram_mb >= 2048;
    if env_truthy("FSOT_LEAN") || env_truthy("FSOT_MIN_SPEC") {
        return "lean";
    }
    if let Ok(s) = std::env::var("FSOT_SCALE") {
        let s = s.to_ascii_lowercase();
        if matches!(
            s.as_str(),
            "lean" | "standard" | "expanded" | "max" | "auto"
        ) {
            if s != "auto" {
                return match s.as_str() {
                    "lean" => "lean",
                    "standard" => "standard",
                    "expanded" => "expanded",
                    "max" => "max",
                    _ => "standard",
                };
            }
        }
    }
    if ram_mb > 0 && ram_mb < 6144 && !has_gpu {
        "lean"
    } else if vram_mb >= 12000 {
        "max" // RTX 5070-class 12GB+ VRAM (Omen 35L etc.)
    } else if vram_mb >= 8000 || ram_mb >= 24000 {
        "expanded"
    } else if ram_mb >= 8192 || cpu_cores >= 6 {
        "standard"
    } else {
        "lean"
    }
}

fn profile_for_tier(
    tier: &str,
    ram_mb: u64,
    vram_mb: u64,
    cpu_cores: u32,
    has_gpu: bool,
) -> HostScale {
    match tier {
        "lean" => HostScale {
            tier: "lean".into(),
            ram_mb,
            vram_mb,
            cpu_cores,
            has_discrete_gpu: has_gpu,
            breath_step: 0.06,
            pop_size: 4,
            elite: 2,
            knowledge_cap: 2_000,
            audio_index_cap: 24,
            video_index_cap: 12,
            exam_every: 50,
            feed_every: 10,
            body_couple_gain: 0.08,
            consolidate_every: 128,
        },
        "expanded" => HostScale {
            tier: "expanded".into(),
            ram_mb,
            vram_mb,
            cpu_cores,
            has_discrete_gpu: has_gpu,
            breath_step: 0.11,
            pop_size: 8,
            elite: 3,
            knowledge_cap: 40_000,
            audio_index_cap: 160,
            video_index_cap: 80,
            exam_every: 32,
            feed_every: 6,
            body_couple_gain: 0.14,
            consolidate_every: 64,
        },
        "max" => HostScale {
            tier: "max".into(),
            ram_mb,
            vram_mb,
            cpu_cores,
            has_discrete_gpu: has_gpu,
            breath_step: 0.14,
            pop_size: 10,
            elite: 3,
            knowledge_cap: 80_000,
            audio_index_cap: 240,
            video_index_cap: 120,
            exam_every: 28,
            feed_every: 5,
            body_couple_gain: 0.18,
            consolidate_every: 48,
        },
        _ => HostScale {
            tier: "standard".into(),
            ram_mb,
            vram_mb,
            cpu_cores,
            has_discrete_gpu: has_gpu,
            breath_step: 0.08,
            pop_size: 6,
            elite: 2,
            knowledge_cap: 12_000,
            audio_index_cap: 80,
            video_index_cap: 40,
            exam_every: 40,
            feed_every: 8,
            body_couple_gain: 0.10,
            consolidate_every: 96,
        },
    }
}

/// Probe the machine and return a scale profile.
pub fn probe() -> HostScale {
    let ram_mb = probe_ram_mb();
    let vram_mb = probe_vram_mb();
    let cpu_cores = probe_cpu_cores();
    let has_gpu = vram_mb >= 2048;
    let tier = tier_from_resources(ram_mb, vram_mb, cpu_cores);
    profile_for_tier(tier, ram_mb, vram_mb, cpu_cores, has_gpu)
}
