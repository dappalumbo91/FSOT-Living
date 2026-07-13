//! Vision organ — image load + structural understanding under evolutionary pressure
//! Not a giant VL model: feature extraction + FSOT-linked regime mapping + exam fitness.

use image::{GenericImageView, Pixel};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ImageAnalysis {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub mean_r: f64,
    pub mean_g: f64,
    pub mean_b: f64,
    pub brightness: f64,
    pub saturation: f64,
    pub aspect: f64,
    pub dominant: &'static str, // red/green/blue/neutral
    pub fsot_regime_hint: &'static str, // emergence if warm/bright, damping if cool/dark
}

pub fn vision_library() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("organism_workspace");
    p.push("vision_lib");
    p
}

pub fn ensure_vision_lib() -> std::io::Result<PathBuf> {
    let dir = vision_library();
    std::fs::create_dir_all(&dir)?;
    // Generate simple synthetic training images if empty
    let marker = dir.join(".seeded");
    if !marker.exists() {
        seed_synthetic_images(&dir)?;
        std::fs::write(marker, b"1")?;
    }
    Ok(dir)
}

fn seed_synthetic_images(dir: &Path) -> std::io::Result<()> {
    use image::{ImageBuffer, Rgb};
    // warm bright -> emergence
    let warm: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(64, 64, |_, _| Rgb([220u8, 120, 40]));
    warm.save(dir.join("warm_emergence.png"))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // cool dark -> damping
    let cool: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(64, 64, |_, _| Rgb([20u8, 40, 160]));
    cool.save(dir.join("cool_damping.png"))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // green mid
    let green: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(64, 64, |_, _| Rgb([40u8, 200, 60]));
    green
        .save(dir.join("green_field.png"))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // gray
    let gray: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_fn(32, 48, |_, _| Rgb([128u8, 128, 128]));
    gray.save(dir.join("gray_neutral.png"))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn analyze_image(path: &Path) -> Result<ImageAnalysis, String> {
    let img = image::open(path).map_err(|e| format!("open image: {e}"))?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err("empty image".into());
    }
    let mut sr = 0u64;
    let mut sg = 0u64;
    let mut sb = 0u64;
    let mut n = 0u64;
    // subsample for speed
    let step_x = (w / 64).max(1);
    let step_y = (h / 64).max(1);
    for y in (0..h).step_by(step_y as usize) {
        for x in (0..w).step_by(step_x as usize) {
            let p = img.get_pixel(x, y).to_rgb();
            let c = p.0;
            sr += c[0] as u64;
            sg += c[1] as u64;
            sb += c[2] as u64;
            n += 1;
        }
    }
    let n = n.max(1) as f64;
    let mean_r = sr as f64 / n / 255.0;
    let mean_g = sg as f64 / n / 255.0;
    let mean_b = sb as f64 / n / 255.0;
    let brightness = (mean_r + mean_g + mean_b) / 3.0;
    let max_c = mean_r.max(mean_g).max(mean_b);
    let min_c = mean_r.min(mean_g).min(mean_b);
    let saturation = if max_c > 1e-6 {
        (max_c - min_c) / max_c
    } else {
        0.0
    };
    let dominant = if mean_r >= mean_g && mean_r >= mean_b {
        "red"
    } else if mean_g >= mean_r && mean_g >= mean_b {
        "green"
    } else {
        "blue"
    };
    // FSOT-linked heuristic: green/warm/bright → emergence; cool+dark → damping
    let fsot_regime_hint = if mean_g > mean_r && mean_g > mean_b && brightness > 0.25 {
        "emergence"
    } else if brightness > 0.45 && (mean_r > mean_b + 0.05) {
        "emergence"
    } else if brightness < 0.45 && mean_b > mean_r + 0.05 {
        "damping"
    } else if saturation < 0.12 {
        "neutral"
    } else {
        "neutral"
    };

    Ok(ImageAnalysis {
        path: path.display().to_string(),
        width: w,
        height: h,
        mean_r,
        mean_g,
        mean_b,
        brightness,
        saturation,
        aspect: w as f64 / h as f64,
        dominant,
        fsot_regime_hint,
    })
}

pub fn describe(analysis: &ImageAnalysis) -> String {
    format!(
        "Image {}x{} path={}\n\
         mean RGB=({:.3},{:.3},{:.3}) brightness={:.3} sat={:.3}\n\
         dominant={} aspect={:.3}\n\
         FSOT visual regime hint: {}",
        analysis.width,
        analysis.height,
        analysis.path,
        analysis.mean_r,
        analysis.mean_g,
        analysis.mean_b,
        analysis.brightness,
        analysis.saturation,
        analysis.dominant,
        analysis.aspect,
        analysis.fsot_regime_hint
    )
}

#[derive(Clone, Debug)]
pub struct VisionChallenge {
    pub file: &'static str,
    pub expect_regime: &'static str,
    pub expect_dominant: Option<&'static str>,
}

pub fn vision_challenges() -> &'static [VisionChallenge] {
    &[
        VisionChallenge {
            file: "warm_emergence.png",
            expect_regime: "emergence",
            expect_dominant: Some("red"),
        },
        VisionChallenge {
            file: "cool_damping.png",
            expect_regime: "damping",
            expect_dominant: Some("blue"),
        },
        VisionChallenge {
            file: "green_field.png",
            expect_regime: "emergence",
            expect_dominant: Some("green"),
        },
        VisionChallenge {
            file: "gray_neutral.png",
            expect_regime: "neutral",
            expect_dominant: None,
        },
    ]
}

pub fn run_vision_exam(ch: &VisionChallenge) -> bool {
    let _ = ensure_vision_lib();
    let path = vision_library().join(ch.file);
    match analyze_image(&path) {
        Ok(a) => {
            let reg_ok = a.fsot_regime_hint == ch.expect_regime;
            let dom_ok = ch
                .expect_dominant
                .map(|d| a.dominant == d)
                .unwrap_or(true);
            reg_ok && dom_ok
        }
        Err(_) => false,
    }
}

pub fn list_vision_images() -> Vec<String> {
    let _ = ensure_vision_lib();
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(vision_library()) {
        for e in rd.flatten() {
            let p = e.path();
            if matches!(
                p.extension().and_then(|x| x.to_str()).map(|s| s.to_lowercase()).as_deref(),
                Some("png") | Some("jpg") | Some("jpeg")
            ) {
                out.push(p.display().to_string());
            }
        }
    }
    out
}
