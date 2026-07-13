//! Host hardware probe for mind_bridge (std only).

use body::{ScaleProfile, ScaleTier};
use std::process::Command;
use std::thread;

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
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
        // wmic removed on modern Windows — use CIM via PowerShell (bytes).
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
    8192
}

fn probe_vram_mb() -> u64 {
    if let Ok(s) = std::env::var("FSOT_VRAM_MB") {
        if let Ok(v) = s.parse() {
            return v;
        }
    }
    if let Ok(out) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut best = 0u64;
            for line in text.lines() {
                if let Ok(mb) = line.trim().parse::<u64>() {
                    best = best.max(mb);
                }
            }
            return best;
        }
    }
    0
}

pub fn probe_scale() -> ScaleProfile {
    if env_truthy("FSOT_LEAN") || env_truthy("FSOT_MIN_SPEC") {
        return ScaleProfile::lean();
    }
    if let Ok(s) = std::env::var("FSOT_SCALE") {
        if let Some(t) = ScaleTier::from_str_name(&s) {
            let ram = probe_ram_mb();
            let vram = probe_vram_mb();
            let cores = probe_cpu_cores();
            return ScaleProfile::from_tier(t, ram, vram, cores, vram >= 2048);
        }
    }
    let ram = probe_ram_mb();
    let vram = probe_vram_mb();
    let cores = probe_cpu_cores();
    ScaleProfile::from_resources(ram, vram, cores)
}
