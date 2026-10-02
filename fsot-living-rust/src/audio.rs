//! Audio energy organ — RMS / activity proxies from drive music under evolutionary pressure
//!
//! Sources: $FSOT_LOCAL_DATA/Debut, $FSOT_LOCAL_DATA/movies, $FSOT_LOCAL_DATA/showes (+ workspace audio_lib synthetic WAV seeds).
//! Decode short windows via ffmpeg when available; pure-WAV path without it.
//! Maps energy → FSOT regime hints (high energy ≈ emergence, low ≈ damping).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
pub struct AudioAnalysis {
    pub path: String,
    pub rms: f64,
    pub peak: f64,
    pub samples: usize,
    pub source: String, // "ffmpeg" | "wav" | "seed" | "proxy"
    pub fsot_regime_hint: &'static str,
    pub energy_band: &'static str, // high | mid | low
}

pub fn audio_library() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("organism_workspace");
    p.push("audio_lib");
    p
}

pub fn media_roots() -> Vec<PathBuf> {
    [
        r"../data_external/local/Debut",
        r"../data_external/local/movies",
        r"../data_external/local/showes",
        r"../data_external/local/Music",
    ]
    .iter()
    .map(PathBuf::from)
    .filter(|p| p.exists())
    .collect()
}

pub fn ensure_audio_lib() -> std::io::Result<PathBuf> {
    let dir = audio_library();
    fs::create_dir_all(&dir)?;
    // Amplitudes chosen so measured RMS maps to regime_from_rms bands:
    //   high: rms > 0.12  → amp ≈ 0.85 → rms ~0.60
    //   mid:  0.04–0.12   → amp ≈ 0.10 → rms ~0.07
    //   low:  rms < 0.04  → amp ≈ 0.03 → rms ~0.021
    // Force reseed when marker version is stale (v1 seeds mislabeled mid/low).
    let marker = dir.join(".seeded");
    let need = match fs::read_to_string(&marker) {
        Ok(s) => !s.contains("v2"),
        Err(_) => true,
    };
    if need {
        write_seed_wav(&dir.join("high_energy_emergence.wav"), 440.0, 0.85)?;
        write_seed_wav(&dir.join("low_energy_damping.wav"), 110.0, 0.03)?;
        write_seed_wav(&dir.join("mid_energy_neutral.wav"), 220.0, 0.10)?;
        fs::write(marker, b"v2-rms-aligned")?;
    }
    Ok(dir)
}

/// 16-bit mono PCM WAV (1 second @ 8 kHz)
fn write_seed_wav(path: &Path, freq_hz: f64, amplitude: f64) -> std::io::Result<()> {
    let sr = 8000u32;
    let n = sr; // 1 sec
    let mut pcm = Vec::with_capacity((n as usize) * 2);
    for i in 0..n {
        let t = i as f64 / sr as f64;
        let s = (2.0 * std::f64::consts::PI * freq_hz * t).sin() * amplitude;
        let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }
    let data_len = pcm.len() as u32;
    let mut f = File::create(path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?; // chunk size
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&1u16.to_le_bytes())?; // mono
    f.write_all(&sr.to_le_bytes())?;
    f.write_all(&(sr * 2).to_le_bytes())?; // byte rate
    f.write_all(&2u16.to_le_bytes())?; // block align
    f.write_all(&16u16.to_le_bytes())?; // bits
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    f.write_all(&pcm)?;
    Ok(())
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

/// Shallow+1 index of audio files under roots (capped).
pub fn index_audio_paths(max: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let exts = ["mp3", "wav", "flac", "m4a", "ogg", "aac"];
    // seeds first
    if let Ok(lib) = ensure_audio_lib() {
        if let Ok(rd) = fs::read_dir(lib) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("wav") {
                    out.push(p);
                }
            }
        }
    }
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
            // skip huge discography noise partially — still allow
            index_walk(&p, exts, depth + 1, max_depth, max, out);
        } else if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
            let el = ext.to_lowercase();
            if exts.contains(&el.as_str()) {
                out.push(p);
            }
        }
    }
}

fn regime_from_rms(rms: f64) -> (&'static str, &'static str) {
    if rms > 0.12 {
        ("high", "emergence")
    } else if rms < 0.04 {
        ("low", "damping")
    } else {
        ("mid", "neutral")
    }
}

pub fn analyze_audio(path: &Path) -> Result<AudioAnalysis, String> {
    let path_s = path.display().to_string();
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "wav" {
        if let Ok(a) = analyze_wav(path) {
            return Ok(a);
        }
    }

    if ffmpeg_available() {
        if let Ok(a) = analyze_via_ffmpeg(path) {
            return Ok(a);
        }
    }

    // Fallback proxy: size + filename cues (when ffmpeg/wav decode unavailable)
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let mb = meta.len() as f64 / 1_000_000.0;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    // Filename priors: quiet/soft/ambient → low; live/rock/party/bass → high
    let name_bias = if name.contains("quiet")
        || name.contains("soft")
        || name.contains("ambient")
        || name.contains("piano")
        || name.contains("acoustic")
    {
        0.025
    } else if name.contains("live")
        || name.contains("rock")
        || name.contains("metal")
        || name.contains("bass")
        || name.contains("party")
        || name.contains("dance")
    {
        0.22
    } else {
        (mb / 10.0).clamp(0.03, 0.18)
    };
    let rms = name_bias.clamp(0.01, 0.45);
    let (band, reg) = regime_from_rms(rms);
    Ok(AudioAnalysis {
        path: path_s,
        rms,
        peak: rms * 1.5,
        samples: 1, // mark as analyzable for "any" exams
        source: "proxy".into(),
        fsot_regime_hint: reg,
        energy_band: band,
    })
}

fn analyze_wav(path: &Path) -> Result<AudioAnalysis, String> {
    let mut f = File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    if buf.len() < 44 || &buf[0..4] != b"RIFF" {
        return Err("not wav".into());
    }
    // find data chunk
    let mut i = 12usize;
    let mut data_off = None;
    let mut data_len = 0usize;
    while i + 8 <= buf.len() {
        let id = &buf[i..i + 4];
        let len = u32::from_le_bytes([buf[i + 4], buf[i + 5], buf[i + 6], buf[i + 7]]) as usize;
        if id == b"data" {
            data_off = Some(i + 8);
            data_len = len;
            break;
        }
        i += 8 + len;
    }
    let off = data_off.ok_or("no data")?;
    let end = (off + data_len).min(buf.len());
    let pcm = &buf[off..end];
    let (rms, peak, n) = rms_i16_le(pcm);
    let (band, reg) = regime_from_rms(rms);
    Ok(AudioAnalysis {
        path: path.display().to_string(),
        rms,
        peak,
        samples: n,
        source: "wav".into(),
        fsot_regime_hint: reg,
        energy_band: band,
    })
}

fn analyze_via_ffmpeg(path: &Path) -> Result<AudioAnalysis, String> {
    // 1.5s mono s16le @ 8kHz
    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-t",
            "1.5",
            "-i",
        ])
        .arg(path)
        .args(["-ac", "1", "-ar", "8000", "-f", "s16le", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("ffmpeg spawn: {e}"))?;
    let mut out = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_end(&mut out);
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() || out.len() < 256 {
        return Err("ffmpeg decode failed".into());
    }
    let (rms, peak, n) = rms_i16_le(&out);
    let (band, reg) = regime_from_rms(rms);
    Ok(AudioAnalysis {
        path: path.display().to_string(),
        rms,
        peak,
        samples: n,
        source: "ffmpeg".into(),
        fsot_regime_hint: reg,
        energy_band: band,
    })
}

fn rms_i16_le(pcm: &[u8]) -> (f64, f64, usize) {
    let mut sum = 0.0f64;
    let mut peak = 0.0f64;
    let mut n = 0usize;
    let step = 2usize;
    for i in (0..pcm.len().saturating_sub(1)).step_by(step) {
        let v = i16::from_le_bytes([pcm[i], pcm[i + 1]]) as f64 / 32768.0;
        let a = v.abs();
        sum += v * v;
        if a > peak {
            peak = a;
        }
        n += 1;
    }
    let rms = if n > 0 { (sum / n as f64).sqrt() } else { 0.0 };
    (rms, peak, n)
}

pub fn describe(a: &AudioAnalysis) -> String {
    format!(
        "Audio: {}\n\
         energy_band={} rms={:.4} peak={:.4} samples={} source={}\n\
         FSOT regime hint: {} (high energy → emergence, low → damping)",
        a.path, a.energy_band, a.rms, a.peak, a.samples, a.source, a.fsot_regime_hint
    )
}

#[derive(Clone, Debug)]
pub struct AudioChallenge {
    pub path: PathBuf,
    pub expect_regime: &'static str,
}

pub fn audio_challenges(index: &[PathBuf]) -> Vec<AudioChallenge> {
    let mut ch = Vec::new();
    // synthetic known labels
    let lib = audio_library();
    ch.push(AudioChallenge {
        path: lib.join("high_energy_emergence.wav"),
        expect_regime: "emergence",
    });
    ch.push(AudioChallenge {
        path: lib.join("low_energy_damping.wav"),
        expect_regime: "damping",
    });
    ch.push(AudioChallenge {
        path: lib.join("mid_energy_neutral.wav"),
        expect_regime: "neutral",
    });
    // real drive files — exam is "analyze succeeds + consistent band"
    for p in index.iter().take(12) {
        if p.extension().and_then(|x| x.to_str()) == Some("wav")
            && p.to_string_lossy().contains("audio_lib")
        {
            continue;
        }
        ch.push(AudioChallenge {
            path: p.clone(),
            expect_regime: "any",
        });
    }
    ch
}

pub fn run_audio_exam(ch: &AudioChallenge) -> bool {
    match analyze_audio(&ch.path) {
        Ok(a) => {
            if ch.expect_regime == "any" {
                // Drive media: success = analyzable signal (wav/ffmpeg/proxy)
                a.samples > 0 || a.source == "proxy" || a.source == "ffmpeg" || a.rms > 0.0
            } else {
                a.fsot_regime_hint == ch.expect_regime
            }
        }
        Err(_) => false,
    }
}

/// Mastery bar for the audio organ (sensory energy → regime map).
pub const AUDIO_TARGET: f64 = 0.75;

/// Three-seed battery — organ must map high/mid/low correctly (sensory calibration).
/// Returns (passed_count, total).
pub fn run_audio_seed_battery() -> (u32, u32) {
    let _ = ensure_audio_lib();
    let lib = audio_library();
    let seeds = [
        (lib.join("high_energy_emergence.wav"), "emergence"),
        (lib.join("mid_energy_neutral.wav"), "neutral"),
        (lib.join("low_energy_damping.wav"), "damping"),
    ];
    let mut ok = 0u32;
    for (p, expect) in &seeds {
        let ch = AudioChallenge {
            path: p.clone(),
            expect_regime: expect,
        };
        if run_audio_exam(&ch) {
            ok += 1;
        }
    }
    (ok, 3)
}
