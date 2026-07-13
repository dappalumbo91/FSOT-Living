//! Host-side home VFS mount — Linux $HOME salvage as disk LTM compartments.
//!
//! Layout under bridge_dir/home/:
//!   knowledge/  tools/  media/  genome/  proposals/  identity/
//! Each has NOTES.txt (working organ mirror) + optional extra files.
//!
//! Biology: write to disk = systems consolidation; RAM organ = working cache.

use body::{OrganId, Soma};
use std::fs;
use std::path::{Path, PathBuf};

const HOME_DIRS: &[&str] = &[
    "knowledge",
    "tools",
    "media",
    "genome",
    "proposals",
    "identity",
];

pub fn home_root(bridge: &Path) -> PathBuf {
    bridge.join("home")
}

pub fn ensure_home_tree(bridge: &Path) {
    let root = home_root(bridge);
    let _ = fs::create_dir_all(&root);
    for d in HOME_DIRS {
        let p = root.join(d);
        let _ = fs::create_dir_all(&p);
        let readme = p.join("README.txt");
        if !readme.exists() {
            let bio = match *d {
                "knowledge" => "long-term semantic memory (hippocampal→cortical encode)",
                "tools" => "consolidated procedural skills / motor programs",
                "media" => "sensory engram pointers (paths), not bulk copies",
                "genome" => "archived germline + BODY_LTM linkage",
                "proposals" => "novelty under selection (prefrontal planning buffer)",
                "identity" => "autobiographical self — who this companion is",
                _ => "home compartment",
            };
            let _ = fs::write(
                readme,
                format!(
                    "FSOT home/{} — biological VFS\n{}\nLinux template: $HOME/{}\nWrite via MOTOR WRITE /home/{}\n",
                    d, bio, d, d
                ),
            );
        }
    }
}

fn organ_for_dir(dir: &str) -> Option<OrganId> {
    match dir {
        "knowledge" => Some(OrganId::HomeKnowledge),
        "tools" => Some(OrganId::HomeTools),
        "media" => Some(OrganId::HomeMedia),
        "genome" => Some(OrganId::HomeGenome),
        "proposals" => Some(OrganId::HomeProposals),
        "identity" => Some(OrganId::HomeIdentity),
        _ => None,
    }
}

/// Load NOTES.txt from each home compartment into soma RAM organs (wake).
pub fn load_home_into_soma(bridge: &Path, soma: &mut Soma) {
    ensure_home_tree(bridge);
    let root = home_root(bridge);
    for d in HOME_DIRS {
        let notes = root.join(d).join("NOTES.txt");
        if let Ok(text) = fs::read_to_string(&notes) {
            if let Some(id) = organ_for_dir(d) {
                let t = text.trim();
                if !t.is_empty() {
                    soma.membrane.get_mut(id).write_str(t);
                }
            }
        }
    }
}

/// Flush home organs from RAM → disk NOTES.txt (consolidate).
pub fn save_home_from_soma(bridge: &Path, soma: &Soma) {
    ensure_home_tree(bridge);
    let root = home_root(bridge);
    for d in HOME_DIRS {
        if let Some(id) = organ_for_dir(d) {
            let node = soma.membrane.get(id);
            let text = String::from_utf8_lossy(&node.content);
            let path = root.join(d).join("NOTES.txt");
            let _ = fs::write(
                path,
                format!(
                    "# FSOT home/{} trit={} field={:.3} writes={}\n{}\n",
                    d,
                    node.state.to_i8(),
                    node.field,
                    node.writes,
                    text
                ),
            );
        }
    }
}

/// Append a durable fact file under home/knowledge (LTM encode).
pub fn append_knowledge_fact(bridge: &Path, fact: &str) {
    ensure_home_tree(bridge);
    let path = home_root(bridge).join("knowledge").join("facts.jsonl");
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let line = format!(
            "{{\"t\":{},\"fact\":{}}}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            serde_json_escape(fact)
        );
        let _ = f.write_all(line.as_bytes());
    }
}

fn serde_json_escape(s: &str) -> String {
    format!("{:?}", s)
}

/// Register a forged tool name into home/tools (procedural LTM).
pub fn register_tool(bridge: &Path, tool_name: &str, detail: &str) {
    ensure_home_tree(bridge);
    let path = home_root(bridge).join("tools").join("skills.jsonl");
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let line = format!(
            "{{\"t\":{},\"tool\":{},\"detail\":{}}}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            serde_json_escape(tool_name),
            serde_json_escape(detail)
        );
        let _ = f.write_all(line.as_bytes());
    }
    // Mirror latest tools list into NOTES
    let notes = home_root(bridge).join("tools").join("NOTES.txt");
    let prev = fs::read_to_string(&notes).unwrap_or_default();
    let entry = format!("tool:{tool_name}");
    if !prev.contains(&entry) {
        let _ = fs::write(
            notes,
            format!("{prev}\n{entry} — {detail}\n").trim_start(),
        );
    }
}

/// Parsed champion germline from LATEST.txt / archive files.
#[derive(Clone, Debug)]
pub struct GermlineArchive {
    pub gen: u32,
    pub hero: String,
    pub fit: f64,
    pub genome: String,
}

/// Load champion DNA from home/genome/LATEST.txt (boot continuity).
pub fn load_latest_germline(bridge: &Path) -> Option<GermlineArchive> {
    ensure_home_tree(bridge);
    let path = home_root(bridge).join("genome").join("LATEST.txt");
    let text = fs::read_to_string(path).ok()?;
    parse_germline_text(&text)
}

/// Parse archive body (`gen=…\nhero=…\nfit=…\ngenome=…`).
pub fn parse_germline_text(text: &str) -> Option<GermlineArchive> {
    let mut gen = 0u32;
    let mut hero = String::from("unknown");
    let mut fit = 0.0f64;
    let mut genome = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(v) = line.strip_prefix("gen=") {
            gen = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("hero=") {
            hero = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("fit=") {
            fit = v.trim().parse().unwrap_or(0.0);
        } else if let Some(v) = line.strip_prefix("genome=") {
            genome = v.trim().to_string();
        }
    }
    if genome.is_empty() {
        None
    } else {
        Some(GermlineArchive {
            gen,
            hero,
            fit,
            genome,
        })
    }
}

/// Sanitize hero id for filesystem (Windows-safe, no path separators).
pub fn sanitize_hero(hero: &str) -> String {
    let s: String = hero
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .take(64)
        .collect();
    if s.is_empty() {
        "unknown".into()
    } else {
        s
    }
}

/// Write champion genome snapshot into home/genome (germline archive).
/// Returns Ok(path to gen_*.txt) on success.
pub fn register_genome(
    bridge: &Path,
    gen: u32,
    hero_id: &str,
    genome: &str,
    fit: f64,
) -> Result<PathBuf, String> {
    ensure_home_tree(bridge);
    let safe_hero = sanitize_hero(hero_id);
    let genome = genome.trim();
    if genome.is_empty() {
        return Err("empty genome".into());
    }
    let dir = home_root(bridge).join("genome");
    let path = dir.join(format!("gen_{gen:04}_{safe_hero}.txt"));
    let body = format!(
        "# FSOT germline archive\ngen={gen}\nhero={hero_id}\nhero_safe={safe_hero}\nfit={fit:.4}\ngenome={genome}\n"
    );
    fs::write(&path, &body).map_err(|e| format!("write {}: {e}", path.display()))?;
    fs::write(dir.join("LATEST.txt"), &body)
        .map_err(|e| format!("write LATEST.txt: {e}"))?;
    // Archive index (append) — survives NOTES overwrite by save_home_from_soma
    {
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("INDEX.jsonl"))
        {
            let line = format!(
                "{{\"gen\":{},\"hero\":{},\"fit\":{:.4},\"genome\":{},\"file\":{}}}\n",
                gen,
                serde_json_escape(hero_id),
                fit,
                serde_json_escape(genome),
                serde_json_escape(&path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
            );
            let _ = f.write_all(line.as_bytes());
        }
    }
    // Champion summary (not overwritten by membrane NOTES flush if we use CHAMPION.txt)
    let _ = fs::write(
        dir.join("CHAMPION.txt"),
        format!("latest gen={gen} hero={hero_id} fit={fit:.3}\n{genome}\n"),
    );
    Ok(path)
}

/// Log a proposal id into home/proposals (novelty under selection).
pub fn register_proposal(bridge: &Path, prop_id: &str, summary: &str) {
    ensure_home_tree(bridge);
    let path = home_root(bridge).join("proposals").join("index.jsonl");
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let line = format!(
            "{{\"t\":{},\"id\":{},\"summary\":{}}}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            serde_json_escape(prop_id),
            serde_json_escape(summary)
        );
        let _ = f.write_all(line.as_bytes());
    }
}
