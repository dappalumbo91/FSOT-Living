//! Self-expansion proposals — organism invents capability upgrades for *human review*
//!
//! Biologically: novelty generation under pressure (like exploratory behavior).
//! Engineering: NEVER auto-compile Rust or patch the binary.
//! Interpretable organs (Python/JS tools, curriculum, exams) may be proposed as text + code
//! and only land after review. Queue lives on disk for the human / agent.

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize)]
pub struct Proposal {
    pub id: String,
    pub org_id: String,
    pub kind: String,
    pub title: String,
    pub rationale: String,
    pub body: String,
    pub evidence: String,
    pub tick: u64,
    pub generation: u32,
    pub auto_apply_safe: bool,
    pub status: String,
}

fn proposals_dir() -> PathBuf {
    // Prefer state next to habitat for review alongside LIVE_STATUS
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("state");
    p.push("habitat-rust");
    p.push("PROPOSALS");
    p
}

fn workspace_proposals() -> PathBuf {
    crate::code_env::workspace_root().join("proposals")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    fs::create_dir_all(proposals_dir())?;
    fs::create_dir_all(workspace_proposals())?;
    // Human-facing README once
    let readme = proposals_dir().join("README.md");
    if !readme.exists() {
        fs::write(
            readme,
            "# Organism self-expansion proposals\n\n\
             These files are **suggestions** generated under evolutionary pressure.\n\n\
             - `auto_apply_safe: true` → interpretable tool/script only (Python/JS) after you verify tests\n\
             - `auto_apply_safe: false` → Rust/core architecture ideas — **manual review only, never auto-compile**\n\n\
             Workflow: read proposal → test in sandbox → implement if sound → leave status note.\n\
             The organism does **not** modify its own Rust binary.\n",
        )?;
    }
    Ok(())
}

/// Emit a proposal to the review queue. Returns proposal id.
pub fn submit(p: &Proposal) -> std::io::Result<String> {
    ensure_dirs()?;
    let dir = proposals_dir();
    let path = dir.join(format!("{}.json", p.id));
    let json = serde_json::to_string_pretty(p)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&path, json)?;
    // Markdown for easy human read
    let md = dir.join(format!("{}.md", p.id));
    let mut f = fs::File::create(md)?;
    writeln!(f, "# {}\n", p.title)?;
    writeln!(f, "- **id:** {}", p.id)?;
    writeln!(f, "- **org:** {}", p.org_id)?;
    writeln!(f, "- **kind:** {}", p.kind)?;
    writeln!(f, "- **tick/gen:** {} / {}", p.tick, p.generation)?;
    writeln!(f, "- **auto_apply_safe:** {}", p.auto_apply_safe)?;
    writeln!(f, "- **status:** {}\n", p.status)?;
    writeln!(f, "## Rationale\n\n{}\n", p.rationale)?;
    writeln!(f, "## Evidence\n\n{}\n", p.evidence)?;
    writeln!(f, "## Body\n\n```\n{}\n```\n", p.body)?;
    writeln!(
        f,
        "## Review\n\n- [ ] Reject\n- [ ] Accept (manual implement)\n- [ ] Accept tool only if safe\n"
    )?;
    // Append index
    let line = serde_json::json!({
        "id": p.id,
        "title": p.title,
        "kind": p.kind,
        "org_id": p.org_id,
        "auto_apply_safe": p.auto_apply_safe,
        "status": p.status,
        "tick": p.tick,
    });
    if let Ok(mut idx) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("index.jsonl"))
    {
        let _ = writeln!(idx, "{}", line);
    }
    // Mirror under workspace
    let _ = fs::write(
        workspace_proposals().join(format!("{}.json", p.id)),
        serde_json::to_string_pretty(p).unwrap_or_default(),
    );
    Ok(p.id.clone())
}

pub fn new_id(org_id: &str) -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let safe: String = org_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(16)
        .collect();
    format!("prop_{ms}_{safe}")
}

/// Generate a capability-expansion proposal from organism state (exam under pressure).
pub fn exam_propose(
    org_id: &str,
    tick: u64,
    generation: u32,
    lang_acc: f64,
    code_acc: f64,
    gen_acc: f64,
    tools: &[String],
    seed: usize,
) -> (bool, String) {
    let kinds = [
        (
            "tool_python",
            true,
            "Invent Python organ-tool: variance of S history",
            "def variance(xs):\n    if not xs: return 0\n    m=sum(xs)/len(xs)\n    return sum((x-m)**2 for x in xs)/len(xs)\nprint(variance([0.1,0.2,0.3]))\n",
            "Extends numerical thought for FSOT time-series stability.",
        ),
        (
            "tool_js",
            true,
            "Invent JS organ-tool: softmax-lite normalize",
            "const xs=process.argv.slice(2).map(Number); const m=Math.max(...xs); const e=xs.map(x=>Math.exp(x-m)); const s=e.reduce((a,b)=>a+b,0); console.log(e.map(v=>v/s).map(x=>x.toFixed(3)).join(','));\n",
            "Soft competition weights for pack mate selection affinity.",
        ),
        (
            "exam_battery",
            true,
            "Add multi-hop knowledge+FSOT exam",
            "prompt: 'Using knowledge and fsot_domain, is quantum emergence and why?'\nexpect: emergence + S or tool\n",
            "Stimulates knowledge under tool pressure.",
        ),
        (
            "curriculum",
            true,
            "Curriculum: selective breeding favors accuracy efficiency",
            "TEACH: mates attract by mean_accuracy and low pressure; weak lineages culled from breeding pool.",
            "Align cultural feed with breeding_value selection.",
        ),
        (
            "rust_core",
            false,
            "Architecture idea: deeper temporal video sequences (3+ frames)",
            "PROPOSAL ONLY — extend video.rs to 3-frame regime trajectories; fitness += temporal_coherence.\nHuman design + cargo.",
            "Next video complexity after basic frame sampling works.",
        ),
        (
            "organ_js_sandbox",
            false,
            "Architecture idea: freeform JS invention from generated tests",
            "PROPOSAL ONLY — organism writes novel .js that must pass host-generated asserts.",
            "Higher JS organ complexity under pressure.",
        ),
    ];
    let (kind, safe, title, body, rationale) = kinds[seed % kinds.len()];
    // Constraint: never auto-apply non-safe (rust/core) — still queue for Human, but
    // queue flood / competence / dedup gates apply first.
    let pending = count_pending();
    if pending >= 200 {
        return (false, format!("propose:skip queue full ({pending})"));
    }
    // Dedup: skip if same kind+title already pending in index
    if proposal_kind_recent(kind, title) {
        return (false, format!("propose:skip dedup {kind}"));
    }
    let evidence = format!(
        "org={org_id} tick={tick} gen={generation} lang={:.0}% code={:.0}% gen_acc={:.0}% tools={:?}",
        lang_acc * 100.0,
        code_acc * 100.0,
        gen_acc * 100.0,
        tools
    );
    // Gate: only emit if organism is competent enough (intelligence threshold)
    let competent = code_acc > 0.5 || lang_acc > 0.45 || gen_acc > 0.4;
    if !competent {
        return (false, "propose:skip (competence gate)".into());
    }
    let p = Proposal {
        id: new_id(org_id),
        org_id: org_id.into(),
        kind: kind.into(),
        title: title.into(),
        rationale: rationale.into(),
        body: body.into(),
        evidence,
        tick,
        generation,
        auto_apply_safe: safe,
        status: "pending_review".into(),
    };
    match submit(&p) {
        Ok(id) => {
            // If safe tool proposal, also drop a candidate file under proposals/ for review
            if safe && (kind == "tool_python" || kind == "tool_js") {
                let ext = if kind == "tool_js" { "js" } else { "py" };
                let cand = workspace_proposals().join(format!("candidate_{id}.{ext}"));
                let _ = fs::write(cand, &p.body);
            }
            (true, format!("propose:{kind} → {id} (review queue)"))
        }
        Err(e) => (false, format!("propose:fail {e}")),
    }
}

pub fn count_pending() -> usize {
    let dir = proposals_dir();
    fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .count()
        })
        .unwrap_or(0)
}

fn proposal_kind_recent(kind: &str, title: &str) -> bool {
    let idx = proposals_dir().join("index.jsonl");
    let Ok(text) = fs::read_to_string(idx) else {
        return false;
    };
    let mut n = 0u32;
    for line in text.lines().rev().take(80) {
        if line.contains(&format!("\"kind\":\"{kind}\"")) && line.contains(&title[..title.len().min(24)]) {
            n += 1;
        }
        // also kind-only spam
        if line.contains(&format!("\"kind\":\"{kind}\"")) {
            n += 1;
        }
    }
    n >= 2
}
