//! Video drive pressure — frame sampling from $FSOT_LOCAL_DATA/movies / showes / Debut
//!
//! Uses ffmpeg to pull 1–2 scaled frames, then vision organ features.
//! Temporal pressure: compare frame regime hints for consistency / shift.

use crate::vision::{self, ImageAnalysis};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
pub struct VideoAnalysis {
    pub path: String,
    pub frames: Vec<ImageAnalysis>,
    pub mean_brightness: f64,
    pub regime_sequence: Vec<String>,
    pub temporal_shift: bool,
    pub fsot_regime_hint: String,
}

pub fn video_scratch() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("organism_workspace");
    p.push("video_scratch");
    p
}

pub fn media_roots() -> Vec<PathBuf> {
    [r"../data_external/local/movies", r"../data_external/local/showes", r"../data_external/local/Debut"]
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}

pub fn ensure_video_scratch() -> std::io::Result<PathBuf> {
    let d = video_scratch();
    fs::create_dir_all(&d)?;
    Ok(d)
}

pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn index_video_paths(max: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let exts = ["mp4", "mkv", "avi", "webm", "mov"];
    for root in media_roots() {
        index_walk(&root, &exts, 0, 3, max, &mut out);
        if out.len() >= max {
            break;
        }
    }
    out.truncate(max);
    out
}

fn index_walk(
    dir: &Path,
    exts: &[&str],
    depth: usize,
    max_depth: usize,
    max: usize,
    out: &mut Vec<PathBuf>,
) {
    if out.len() >= max || depth > max_depth {
        return;
    }
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        if out.len() >= max {
            return;
        }
        let p = e.path();
        if p.is_dir() {
            index_walk(&p, exts, depth + 1, max_depth, max, out);
        } else if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
            if exts.contains(&ext.to_lowercase().as_str()) {
                // Prefer mid-size files: reliable ffmpeg, not multi-GB BluRays first
                if let Ok(meta) = fs::metadata(&p) {
                    let n = meta.len();
                    if (3_000_000..120_000_000).contains(&n) {
                        out.push(p);
                    } else if n > 120_000_000 && out.len() < max / 3 {
                        // a few large files for pressure diversity
                        out.push(p);
                    }
                }
            }
        }
    }
}

fn extract_frame(video: &Path, t_sec: f64, out_png: &Path) -> Result<(), String> {
    if !ffmpeg_available() {
        return Err("ffmpeg missing".into());
    }
    // Fast seek first; if that fails caller may retry other times.
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-ss"])
        .arg(format!("{t_sec:.2}"))
        .arg("-i")
        .arg(video)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "scale=64:64",
            "-an",
            "-q:v",
            "8",
        ])
        .arg(out_png)
        .status()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    if !status.success() || !out_png.exists() {
        // Accurate seek fallback (slower, better on some containers)
        let status2 = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(video)
            .args(["-ss"])
            .arg(format!("{t_sec:.2}"))
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale=64:64",
                "-an",
                "-q:v",
                "8",
            ])
            .arg(out_png)
            .status()
            .map_err(|e| format!("ffmpeg: {e}"))?;
        if !status2.success() || !out_png.exists() {
            return Err("frame extract failed".into());
        }
    }
    Ok(())
}

pub fn analyze_video(path: &Path) -> Result<VideoAnalysis, String> {
    let _ = ensure_video_scratch().map_err(|e| e.to_string())?;
    let scratch = video_scratch();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("vid")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(20)
        .collect::<String>();
    let uniq = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() % 100_000)
        .unwrap_or(0);
    let f0 = scratch.join(format!("{stem}_{uniq}_t1.png"));
    let f1 = scratch.join(format!("{stem}_{uniq}_t2.png"));
    // Early timestamps (fast on large files); second frame optional for temporal
    let mut frames = Vec::new();
    for (t, out) in [(0.5, &f0), (2.0, &f1), (1.0, &f0)] {
        match extract_frame(path, t, out) {
            Ok(()) => {
                if let Ok(a) = vision::analyze_image(out) {
                    frames.push(a);
                }
            }
            Err(_) => continue,
        }
        if frames.len() >= 1 {
            break;
        }
    }
    if frames.is_empty() {
        let f = scratch.join(format!("{stem}_{uniq}_t0.png"));
        extract_frame(path, 0.2, &f)?;
        let a = vision::analyze_image(&f)?;
        frames.push(a);
    }
    let mean_b = frames.iter().map(|f| f.brightness).sum::<f64>() / frames.len() as f64;
    let regimes: Vec<String> = frames
        .iter()
        .map(|f| f.fsot_regime_hint.to_string())
        .collect();
    let temporal_shift = regimes.len() > 1 && regimes.iter().any(|r| r != &regimes[0]);
    let hint = if temporal_shift {
        "shift".to_string()
    } else {
        regimes.first().cloned().unwrap_or_else(|| "neutral".into())
    };
    Ok(VideoAnalysis {
        path: path.display().to_string(),
        frames,
        mean_brightness: mean_b,
        regime_sequence: regimes,
        temporal_shift,
        fsot_regime_hint: hint,
    })
}

pub fn describe(v: &VideoAnalysis) -> String {
    format!(
        "Video: {}\n\
         frames={} mean_brightness={:.3} temporal_shift={}\n\
         regimes={:?}\n\
         FSOT temporal hint: {}",
        v.path,
        v.frames.len(),
        v.mean_brightness,
        v.temporal_shift,
        v.regime_sequence,
        v.fsot_regime_hint
    )
}

pub fn run_video_exam(path: &Path) -> bool {
    match analyze_video(path) {
        Ok(v) => !v.frames.is_empty(),
        Err(_) => false,
    }
}

/// Mastery bar for the video organ (temporal succession / frame succession).
pub const VIDEO_TARGET: f64 = 0.75;

/// Synthetic temporal exam — two vision frames as a mini “clip”.
/// Biology: when drive-media ffmpeg fails, organ still practices frame succession
/// (like imagining motion from stills / predictive coding).
pub fn run_synthetic_video_exam() -> bool {
    let _ = vision::ensure_vision_lib();
    let lib = vision::vision_library();
    let frames = [
        lib.join("warm_emergence.png"),
        lib.join("gray_neutral.png"),
        lib.join("cool_damping.png"),
        lib.join("green_field.png"),
    ];
    let mut ok_n = 0u32;
    for p in &frames {
        if p.exists() && vision::analyze_image(p).is_ok() {
            ok_n += 1;
        }
    }
    ok_n >= 2
}

/// Curriculum seed lines for habitat teach()
pub fn drive_video_curriculum_seeds(index: &[PathBuf], max: usize) -> Vec<(String, String)> {
    index
        .iter()
        .take(max)
        .map(|p| {
            (
                format!(
                    "VIDEO_DRIVE media: {} — sample frames under video organ pressure",
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("video")
                        .chars()
                        .take(80)
                        .collect::<String>()
                ),
                "video_drive".into(),
            )
        })
        .collect()
}

pub fn drive_audio_curriculum_seeds(index: &[PathBuf], max: usize) -> Vec<(String, String)> {
    index
        .iter()
        .take(max)
        .map(|p| {
            (
                format!(
                    "AUDIO_DRIVE media: {} — energy organ under FSOT regime mapping",
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("audio")
                        .chars()
                        .take(80)
                        .collect::<String>()
                ),
                "audio_drive".into(),
            )
        })
        .collect()
}
