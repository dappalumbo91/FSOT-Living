//! Cell-tracking vision organ — Biohub Kaggle v50 bridge (2026-07-13).
//!
//! Ports the competitive pipeline from FSOT-2.1-Lean `fsot_living_emergence.py`:
//! - accuracy_homeo vision band (CORE 0.62): adaptive emergence only below proxy
//! - closed-set rank: fuse U-Net confidence with FSOT fertile scalar
//! - CPU Kaggle path: dormant at proxy ≥ 0.90; conf-ranked NMS only when winning

use serde::{Deserialize, Serialize};

pub const VISION_CORE_THR: f64 = 0.62;
pub const DEFAULT_PROXY: f64 = 0.90;
pub const DEFAULT_TARGET_NODES_PER_FRAME: f64 = 258.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivingVisionState {
    pub regime: String,
    pub organ_accuracy_proxy: f64,
    pub deficit: f64,
    pub boost_strength: f64,
    pub det_threshold_delta: f64,
    pub nms_um_delta: f64,
    pub gate_frac_delta: f64,
    pub per_frame_cap_scale: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KaggleCompetitiveDefaults {
    pub link_mode: String,
    pub det_threshold: f64,
    pub nms_um: f64,
    pub use_ilp: bool,
    pub ilp_max_edges: u32,
    pub device: String,
    pub train_proxy_score: f64,
    pub leaderboard_target: f64,
    pub lean_ref: String,
}

impl Default for KaggleCompetitiveDefaults {
    fn default() -> Self {
        Self {
            link_mode: "fsot".into(),
            det_threshold: 0.48,
            nms_um: 6.0,
            use_ilp: true,
            ilp_max_edges: 80_000,
            device: "cpu".into(),
            train_proxy_score: 0.9044,
            leaderboard_target: 0.97,
            lean_ref: "https://github.com/dappalumbo91/FSOT-2.1-Lean.git".into(),
        }
    }
}

pub fn boost_strength(acc: f64, thr: f64, teach_gain: f64) -> f64 {
    let deficit = (thr - acc).max(0.0);
    (0.55 + deficit * 1.5) * teach_gain.clamp(0.4, 1.4)
}

pub fn frame_regime_from_stats(
    brightness: f64,
    mean_r: f64,
    mean_g: f64,
    mean_b: f64,
    sat: f64,
) -> &'static str {
    if mean_g > mean_r && mean_g > mean_b && brightness > 0.25 {
        return "emergence";
    }
    if brightness > 0.45 && (mean_r > mean_b + 0.05) {
        return "emergence";
    }
    if brightness < 0.45 && mean_b > mean_r + 0.05 {
        return "damping";
    }
    if sat < 0.12 {
        return "neutral";
    }
    "neutral"
}

pub fn fuse_detection_score(unet_conf: f64, fsot_scalar: f64, ml_w: f64, fsot_w: f64) -> f64 {
    let uc = unet_conf.max(1e-6);
    let fs = fsot_scalar.max(1e-6);
    uc.powf(ml_w) * fs.powf(fsot_w)
}

pub fn living_should_activate(proxy_accuracy: f64, adaptive: bool) -> bool {
    if !adaptive {
        return true;
    }
    proxy_accuracy < VISION_CORE_THR
}

pub fn living_vision_state(
    proxy_accuracy: f64,
    mean_brightness: f64,
    mean_r: f64,
    mean_g: f64,
    mean_b: f64,
    saturation: f64,
    nodes_per_frame: f64,
    target_nodes_per_frame: f64,
    teach_gain: f64,
) -> LivingVisionState {
    let mut regime = frame_regime_from_stats(
        mean_brightness, mean_r, mean_g, mean_b, saturation,
    )
    .to_string();
    let deficit = (VISION_CORE_THR - proxy_accuracy).max(0.0);
    let boost = boost_strength(proxy_accuracy, VISION_CORE_THR, teach_gain);

    let over_density = target_nodes_per_frame > 0.0
        && nodes_per_frame > target_nodes_per_frame * 1.15;
    let severe_over = target_nodes_per_frame > 0.0
        && nodes_per_frame > target_nodes_per_frame * 1.25;
    let under_density = target_nodes_per_frame > 0.0
        && nodes_per_frame < target_nodes_per_frame * 0.90;

    if severe_over {
        regime = "damping".into();
    } else if over_density && proxy_accuracy < VISION_CORE_THR {
        regime = "damping".into();
    } else if under_density && proxy_accuracy < VISION_CORE_THR {
        regime = "emergence".into();
    }

    let (mut det_delta, mut nms_delta, mut gate_delta, mut cap_scale) =
        (0.0, 0.0, 0.0, 1.0);

    match regime.as_str() {
        "emergence" => {
            det_delta = -0.03 * boost;
            nms_delta = -0.5;
            gate_delta = -0.02;
            cap_scale = 1.05;
        }
        "damping" => {
            det_delta = 0.03 + 0.02 * deficit.min(0.5);
            nms_delta = if over_density { 0.5 } else { 0.0 };
            gate_delta = 0.02;
            cap_scale = if severe_over { 0.95 } else { 1.0 };
        }
        _ => {
            det_delta = -0.01 * deficit;
            gate_delta = -0.005 * deficit;
        }
    }

    LivingVisionState {
        regime,
        organ_accuracy_proxy: proxy_accuracy,
        deficit,
        boost_strength: boost,
        det_threshold_delta: det_delta,
        nms_um_delta: nms_delta,
        gate_frac_delta: gate_delta,
        per_frame_cap_scale: cap_scale,
    }
}

pub fn apply_living_det_adjustments(base_det: f64, state: &LivingVisionState, lo: f64, hi: f64) -> f64 {
    (base_det + state.det_threshold_delta).clamp(lo, hi)
}

/// Rolling proxy from Kaggle feedback or local train-proxy (written by Lean bundle).
pub fn load_proxy_from_state() -> Option<f64> {
    let path = proxy_state_path();
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("proxy_accuracy")?.as_f64()
}

pub fn proxy_state_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("state");
    p.push("habitat-rust");
    p.push("KAGGLE_CELL_TRACKING_PROXY.json");
    p
}

pub fn write_proxy_state(proxy: f64, source: &str) -> std::io::Result<()> {
    let path = proxy_state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let doc = serde_json::json!({
        "proxy_accuracy": proxy,
        "source": source,
        "adaptive_band": VISION_CORE_THR,
        "competitive_defaults": KaggleCompetitiveDefaults::default(),
        "note": "Dormant Living emergence when proxy >= 0.62; conf-rank NMS when proxy >= 0.90"
    });
    std::fs::write(path, serde_json::to_string_pretty(&doc)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dormant_at_competitive_proxy() {
        assert!(!living_should_activate(0.90, true));
        assert!(living_should_activate(0.55, true));
    }

    #[test]
    fn fuse_prefers_high_conf_and_scalar() {
        let a = fuse_detection_score(0.9, 0.4, 0.72, 0.28);
        let b = fuse_detection_score(0.5, 0.4, 0.72, 0.28);
        assert!(a > b);
    }
}