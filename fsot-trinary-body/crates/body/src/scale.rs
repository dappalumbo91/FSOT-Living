//! Environment scale — "goldfish rule": body grows into the tank it lives in.
//!
//! Detected (or forced) hardware capacity widens breath, consolidate cadence,
//! and living-pack budgets. Min-spec (2012 Mini / 4GB) stays small; an Omen
//! with lots of RAM + RTX 5070-class VRAM expands capabilities.

use alloc::format;
use alloc::string::String;

/// Named capacity tier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScaleTier {
    /// ≤~6GB RAM, no useful GPU — 2012 Mini class
    Lean,
    /// Typical modern laptop / mid desktop
    Standard,
    /// High RAM and/or discrete GPU VRAM (e.g. gaming PC)
    Expanded,
    /// High-end workstation (lots of RAM + ≥12GB VRAM)
    Max,
}

impl ScaleTier {
    pub fn as_str(self) -> &'static str {
        match self {
            ScaleTier::Lean => "lean",
            ScaleTier::Standard => "standard",
            ScaleTier::Expanded => "expanded",
            ScaleTier::Max => "max",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("lean")
            || s.eq_ignore_ascii_case("min")
            || s.eq_ignore_ascii_case("mini")
        {
            Some(ScaleTier::Lean)
        } else if s.eq_ignore_ascii_case("standard")
            || s.eq_ignore_ascii_case("std")
            || s.eq_ignore_ascii_case("normal")
        {
            Some(ScaleTier::Standard)
        } else if s.eq_ignore_ascii_case("expanded")
            || s.eq_ignore_ascii_case("expand")
            || s.eq_ignore_ascii_case("gpu")
        {
            Some(ScaleTier::Expanded)
        } else if s.eq_ignore_ascii_case("max")
            || s.eq_ignore_ascii_case("full")
            || s.eq_ignore_ascii_case("omen")
        {
            Some(ScaleTier::Max)
        } else {
            None
        }
    }
}

/// Full scale profile (host fills probe fields; body uses breath/consolidate).
#[derive(Clone, Debug)]
pub struct ScaleProfile {
    pub tier: ScaleTier,
    pub ram_mb: u64,
    pub vram_mb: u64,
    pub cpu_cores: u32,
    pub has_discrete_gpu: bool,
    /// Phase advance per body tick (wider breath = richer homeostasis).
    pub breath_step: f64,
    /// Host sleep between ticks (ms).
    pub tick_sleep_ms: u64,
    /// Auto-consolidate (sleep to disk) every N ticks.
    pub consolidate_every: u64,
    /// Telemetry emit period (ticks).
    pub mind_every: u64,
    /// Plastic rest period.
    pub rest_every: u64,
    /// Living pack size.
    pub pop_size: usize,
    pub elite: usize,
    pub knowledge_cap: usize,
    pub audio_index_cap: usize,
    pub video_index_cap: usize,
    pub exam_every: u64,
    pub feed_every: u64,
    /// How strongly body couple influences mind pressure.
    pub body_couple_gain: f64,
}

impl ScaleProfile {
    pub fn lean() -> Self {
        Self::from_tier(ScaleTier::Lean, 4096, 0, 2, false)
    }

    pub fn standard() -> Self {
        Self::from_tier(ScaleTier::Standard, 16384, 0, 8, false)
    }

    pub fn from_tier(
        tier: ScaleTier,
        ram_mb: u64,
        vram_mb: u64,
        cpu_cores: u32,
        has_discrete_gpu: bool,
    ) -> Self {
        match tier {
            ScaleTier::Lean => ScaleProfile {
                tier,
                ram_mb,
                vram_mb,
                cpu_cores,
                has_discrete_gpu,
                breath_step: 0.06,
                tick_sleep_ms: 50,
                consolidate_every: 128,
                mind_every: 10,
                rest_every: 80,
                pop_size: 4,
                elite: 2,
                knowledge_cap: 2_000,
                audio_index_cap: 24,
                video_index_cap: 12,
                exam_every: 50,
                feed_every: 10,
                body_couple_gain: 0.08,
            },
            ScaleTier::Standard => ScaleProfile {
                tier,
                ram_mb,
                vram_mb,
                cpu_cores,
                has_discrete_gpu,
                breath_step: 0.08,
                tick_sleep_ms: 40,
                consolidate_every: 96,
                mind_every: 8,
                rest_every: 64,
                pop_size: 6,
                elite: 2,
                knowledge_cap: 12_000,
                audio_index_cap: 80,
                video_index_cap: 40,
                exam_every: 40,
                feed_every: 8,
                body_couple_gain: 0.10,
            },
            ScaleTier::Expanded => ScaleProfile {
                tier,
                ram_mb,
                vram_mb,
                cpu_cores,
                has_discrete_gpu,
                breath_step: 0.11,
                tick_sleep_ms: 28,
                consolidate_every: 64,
                mind_every: 6,
                rest_every: 48,
                pop_size: 8,
                elite: 3,
                knowledge_cap: 40_000,
                audio_index_cap: 160,
                video_index_cap: 80,
                exam_every: 32,
                feed_every: 6,
                body_couple_gain: 0.14,
            },
            ScaleTier::Max => ScaleProfile {
                tier,
                ram_mb,
                vram_mb,
                cpu_cores,
                has_discrete_gpu,
                breath_step: 0.14,
                tick_sleep_ms: 20,
                consolidate_every: 48,
                mind_every: 4,
                rest_every: 40,
                pop_size: 10,
                elite: 3,
                knowledge_cap: 80_000,
                audio_index_cap: 240,
                video_index_cap: 120,
                exam_every: 28,
                feed_every: 5,
                body_couple_gain: 0.18,
            },
        }
    }

    /// Pick tier from measured resources (goldfish rule).
    pub fn from_resources(ram_mb: u64, vram_mb: u64, cpu_cores: u32) -> Self {
        let has_gpu = vram_mb >= 2048;
        let tier = if ram_mb > 0 && ram_mb < 6144 && !has_gpu {
            ScaleTier::Lean
        } else if vram_mb >= 12000 {
            // High-end discrete GPU (e.g. RTX 5070 12GB on Omen 35L) —
            // treat as max tank even if OS RAM probe under-reports.
            ScaleTier::Max
        } else if vram_mb >= 8000 || ram_mb >= 24000 {
            ScaleTier::Expanded
        } else if ram_mb >= 8192 || cpu_cores >= 6 {
            ScaleTier::Standard
        } else {
            ScaleTier::Lean
        };
        Self::from_tier(tier, ram_mb, vram_mb, cpu_cores.max(1), has_gpu)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "SCALE tier={} ram_mb={} vram_mb={} cores={} gpu={} breath={:.3} consolidate_every={} pop={} knowledge_cap={}",
            self.tier.as_str(),
            self.ram_mb,
            self.vram_mb,
            self.cpu_cores,
            self.has_discrete_gpu as u8,
            self.breath_step,
            self.consolidate_every,
            self.pop_size,
            self.knowledge_cap
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lean_from_small_ram() {
        let p = ScaleProfile::from_resources(4096, 0, 2);
        assert_eq!(p.tier, ScaleTier::Lean);
        assert!(p.breath_step < 0.09);
    }

    #[test]
    fn omen_class_is_max() {
        // 12GB VRAM alone (5070-class) is enough for max tank
        let p = ScaleProfile::from_resources(8192, 12288, 16);
        assert_eq!(p.tier, ScaleTier::Max);
        assert!(p.pop_size >= 8);
        assert!(p.breath_step > 0.10);
    }

    #[test]
    fn expanded_mid_gpu() {
        let p = ScaleProfile::from_resources(16000, 8192, 8);
        assert_eq!(p.tier, ScaleTier::Expanded);
    }
}
