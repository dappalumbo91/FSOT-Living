//! Internal Python (and later JS) sandbox — interpretable thought / practice / tools.
//! Evolutionary pressure: organisms that solve code exams keep higher fitness and retain tools.
//!
//! **Metabolic membrane** (same family as `constraints`): Python/JS spawns cost breath.
//! Budget resets each habitat tick — invent storms must not starve the organism.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

static RUN_SEQ: AtomicU64 = AtomicU64::new(1);
/// Metabolic membrane: max Python spawns per habitat breath (tick).
/// Soft-spot invent storms were wall-clock death — not tick count.
static PY_BUDGET: AtomicU32 = AtomicU32::new(24);
static PY_USED: AtomicU32 = AtomicU32::new(0);

/// Reset Python metabolic budget at the start of each habitat tick.
pub fn reset_python_budget(limit: u32) {
    PY_BUDGET.store(limit.max(4), Ordering::Relaxed);
    PY_USED.store(0, Ordering::Relaxed);
}

/// Ensure at least `need` remaining spawns this breath (organ-specific top-up).
/// Biology: reserve membrane for a high-priority organ without wiping used history
/// of other organs when already flush — only raises remaining if short.
pub fn ensure_python_budget(need: u32) {
    let rem = python_budget_remaining();
    if rem >= need {
        return;
    }
    let used = PY_USED.load(Ordering::Relaxed);
    let lim = PY_BUDGET.load(Ordering::Relaxed);
    // Raise ceiling so remaining ≈ need (keep used accounting)
    let new_lim = used.saturating_add(need).max(lim);
    PY_BUDGET.store(new_lim.max(4), Ordering::Relaxed);
}

pub fn python_budget_remaining() -> u32 {
    let lim = PY_BUDGET.load(Ordering::Relaxed);
    let used = PY_USED.load(Ordering::Relaxed);
    lim.saturating_sub(used)
}

fn take_python_budget() -> bool {
    let lim = PY_BUDGET.load(Ordering::Relaxed);
    let used = PY_USED.fetch_add(1, Ordering::Relaxed);
    used < lim
}

fn kill_process_tree(pid: u32) {
    // Windows: taskkill /T kills the whole tree (python launcher → real interpreter).
    // Unix: kill -9 the process group if we set it; fall back to kill pid.
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[derive(Clone, Debug)]
pub struct ExecResult {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

/// Workspace under the crate for organism tools / practice
pub fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("organism_workspace");
    p
}

pub fn ensure_workspace() -> std::io::Result<PathBuf> {
    let root = workspace_root();
    fs::create_dir_all(root.join("tools"))?;
    fs::create_dir_all(root.join("tools_js"))?;
    fs::create_dir_all(root.join("practice"))?;
    fs::create_dir_all(root.join("scratch"))?;
    fs::create_dir_all(root.join("proposals"))?;
    Ok(root)
}

fn find_python() -> &'static str {
    // Prefer `py -3` on Windows if needed; try python first
    static CACHED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CACHED.get_or_init(|| {
        if Command::new("python")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return "python".into();
        }
        if Command::new("py")
            .args(["-3", "--version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return "py".into();
        }
        "python".into()
    })
    .as_str()
}

/// Run a Python source string with timeout (seconds).
/// Keep timeout short so TUI never freezes for multi-second storms.
///
/// Callers that have habitat constraints should gate first via
/// `constraints.gate_python_exec`. This still applies a static dangerous-pattern denylist.
pub fn run_python(source: &str, timeout_secs: u64) -> ExecResult {
    let lower = source.to_ascii_lowercase();
    if lower.contains("subprocess")
        || lower.contains("os.system")
        || lower.contains("socket.socket")
        || (lower.contains("compile") && lower.contains("rust"))
    {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: "constraint: dangerous python pattern denied (sandbox membrane)".into(),
            exit_code: -2,
            timed_out: false,
        };
    }
    // Metabolic fail-closed: do not spawn when breath budget is gone
    if !take_python_budget() {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: "metabolic: python budget exhausted this breath".into(),
            exit_code: -3,
            timed_out: false,
        };
    }
    // Hard cap 1s — invent storms must not own the wall clock
    let timeout_secs = timeout_secs.min(1).max(1);
    let _ = ensure_workspace();
    let dir = workspace_root().join("scratch");
    // Unique file per invocation — shared run_{pid}.py races concurrent exec
    let seq = RUN_SEQ.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("run_{}_{}.py", std::process::id(), seq));
    if let Err(e) = fs::write(&file, source) {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: format!("write failed: {e}"),
            exit_code: -1,
            timed_out: false,
        };
    }

    let py = find_python();
    let mut cmd = Command::new(py);
    if py == "py" {
        cmd.arg("-3");
    }
    // Critical: scripts under scratch/ put scratch on sys.path[0], not cwd.
    // PYTHONPATH = workspace so multi-file L2 `from helpers.X import` works
    // (biological analogue: organs can address shared tissue modules).
    let root = workspace_root();
    cmd.arg(&file)
        .current_dir(&root)
        .env("PYTHONPATH", &root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&file);
            return ExecResult {
                ok: false,
                stdout: String::new(),
                stderr: format!("spawn python failed: {e}"),
                exit_code: -1,
                timed_out: false,
            };
        }
    };

    let pid = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = child.wait_with_output();
        let _ = tx.send(out);
    });

    // Hard wall-clock bound: on timeout kill the tree so we never hang on join.
    let result = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let code = output.status.code().unwrap_or(-1);
            ExecResult {
                ok: output.status.success(),
                stdout,
                stderr,
                exit_code: code,
                timed_out: false,
            }
        }
        Ok(Err(e)) => ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: format!("wait error: {e}"),
            exit_code: -1,
            timed_out: false,
        },
        Err(_) => {
            kill_process_tree(pid);
            // Drain waiter briefly after kill (do not block forever)
            let _ = rx.recv_timeout(Duration::from_millis(400));
            ExecResult {
                ok: false,
                stdout: String::new(),
                stderr: "timeout".into(),
                exit_code: -1,
                timed_out: true,
            }
        }
    };
    let _ = fs::remove_file(&file);
    result
}

/// Persist a tool script under tools/ if body is valid-ish Python
pub fn save_tool(name: &str, source: &str) -> std::io::Result<PathBuf> {
    let root = ensure_workspace()?;
    let safe: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let path = root.join("tools").join(format!("{safe}.py"));
    let mut f = fs::File::create(&path)?;
    f.write_all(source.as_bytes())?;
    Ok(path)
}

pub fn run_tool(name: &str, args_line: &str) -> ExecResult {
    let path = workspace_root().join("tools").join(format!("{name}.py"));
    if !path.exists() {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: format!("tool not found: {name}"),
            exit_code: -1,
            timed_out: false,
        };
    }
    let py = find_python();
    let mut cmd = Command::new(py);
    if py == "py" {
        cmd.arg("-3");
    }
    cmd.arg(&path);
    if !args_line.trim().is_empty() {
        for a in args_line.split_whitespace() {
            cmd.arg(a);
        }
    }
    cmd.current_dir(workspace_root())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match cmd.output() {
        Ok(output) => ExecResult {
            ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            timed_out: false,
        },
        Err(e) => ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
            timed_out: false,
        },
    }
}

/// Code exam tasks — evolutionary pressure for coding organ
pub struct CodeChallenge {
    pub id: &'static str,
    pub source: &'static str,
    pub expect_stdout: &'static str,
}

pub fn code_challenges() -> &'static [CodeChallenge] {
    &[
        CodeChallenge {
            id: "add",
            source: "print(2 + 40)\n",
            expect_stdout: "42",
        },
        CodeChallenge {
            id: "len_list",
            source: "print(len([1,2,3,4,5]))\n",
            expect_stdout: "5",
        },
        CodeChallenge {
            id: "string",
            source: "print('FSOT'.lower())\n",
            expect_stdout: "fsot",
        },
        CodeChallenge {
            id: "loop_sum",
            source: "print(sum(range(1,11)))\n",
            expect_stdout: "55",
        },
        CodeChallenge {
            id: "phi_approx",
            source: "print(round((1+5**0.5)/2, 3))\n",
            expect_stdout: "1.618",
        },
        CodeChallenge {
            id: "bool",
            source: "print(int(3 > 1))\n",
            expect_stdout: "1",
        },
        CodeChallenge {
            id: "dict",
            source: "d={'a':1,'b':2}; print(d['b'])\n",
            expect_stdout: "2",
        },
        CodeChallenge {
            id: "list_comp",
            source: "print(sum([x*x for x in range(5)]))\n",
            expect_stdout: "30",
        },
        // L2 — multi-step / structure (code organ complexity pressure)
        CodeChallenge {
            id: "pow2",
            source: "print(2**8)\n",
            expect_stdout: "256",
        },
        CodeChallenge {
            id: "fib_like",
            source: "a,b=0,1\nfor _ in range(6): a,b=b,a+b\nprint(a)\n",
            expect_stdout: "8",
        },
        CodeChallenge {
            id: "compose_double",
            source: "def d(x): return x*2\nprint(d(d(3)))\n",
            expect_stdout: "12",
        },
        CodeChallenge {
            id: "filter_sum",
            source: "print(sum(x for x in range(10) if x%2==0))\n",
            expect_stdout: "20",
        },
        CodeChallenge {
            id: "json_key",
            source: "import json; print(json.loads('{\"s\":0.42}')['s'])\n",
            expect_stdout: "0.42",
        },
    ]
}

/// Multi-file style L2: write helper then use it (interpretable organ growth).
/// Biology: multi-compartment enzyme cascade — must import across module membrane.
pub fn run_code_l2_exam(rng_pick: usize) -> bool {
    let cases = [
        (
            "helpers/mul.py",
            "def mul(a,b):\n    return a*b\n",
            // sys.path guard: even if scratch is path[0], workspace helpers resolve
            "import sys, os\nsys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))) if '__file__' in dir() else '.')\nfrom helpers.mul import mul\nprint(mul(6,7))\n",
            "42",
        ),
        (
            "helpers/sign.py",
            "def sign(x):\n    return 'emergence' if x>0 else 'damping' if x<0 else 'neutral'\n",
            "import sys, os\nroot=os.environ.get('PYTHONPATH','.').split(os.pathsep)[0]\nif root and root not in sys.path: sys.path.insert(0, root)\nfrom helpers.sign import sign\nprint(sign(0.3))\n",
            "emergence",
        ),
        (
            "helpers/clamp01.py",
            "def clamp01(x):\n    return max(0.0, min(1.0, float(x)))\n",
            "import sys, os\nroot=os.environ.get('PYTHONPATH','.').split(os.pathsep)[0]\nif root and root not in sys.path: sys.path.insert(0, root)\nfrom helpers.clamp01 import clamp01\nprint(clamp01(1.5))\n",
            "1.0",
        ),
        (
            "helpers/mean.py",
            "def mean(xs):\n    return sum(xs)/len(xs) if xs else 0\n",
            "import sys, os\nroot=os.environ.get('PYTHONPATH','.').split(os.pathsep)[0]\nif root and root not in sys.path: sys.path.insert(0, root)\nfrom helpers.mean import mean\nprint(mean([2,4,6]))\n",
            "4.0",
        ),
    ];
    let (path, helper, main, expect) = cases[rng_pick % cases.len()];
    let root = match ensure_workspace() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let hp = root.join(path);
    if let Some(parent) = hp.parent() {
        let _ = fs::create_dir_all(parent);
        let _ = fs::write(parent.join("__init__.py"), "");
    }
    if fs::write(&hp, helper).is_err() {
        return false;
    }
    let r = run_python(main, 3);
    if !r.ok {
        return false;
    }
    let out = r.stdout.trim();
    if out == expect {
        return true;
    }
    // float string variants: "1.0" vs "1"
    if let (Ok(a), Ok(b)) = (out.parse::<f64>(), expect.parse::<f64>()) {
        return (a - b).abs() < 1e-9;
    }
    false
}

pub fn run_code_exam(challenge: &CodeChallenge) -> bool {
    let r = run_python(challenge.source, 3);
    r.ok && r.stdout.trim() == challenge.expect_stdout
}

/// Organism "builds a tool" under pressure — small verified helpers
pub fn forge_tool_exam(tool_name: &str) -> bool {
    let source = match tool_name {
        "double" => "import sys\nx=int(sys.argv[1]) if len(sys.argv)>1 else 0\nprint(x*2)\n",
        "add" => {
            "import sys\na=int(sys.argv[1]) if len(sys.argv)>1 else 0\nb=int(sys.argv[2]) if len(sys.argv)>2 else 0\nprint(a+b)\n"
        }
        "fsot_sign" => {
            "import sys\nx=float(sys.argv[1]) if len(sys.argv)>1 else 0\nprint('emergence' if x>0 else 'damping' if x<0 else 'neutral')\n"
        }
        "multiply" => {
            "import sys\na=int(sys.argv[1]) if len(sys.argv)>1 else 1\nb=int(sys.argv[2]) if len(sys.argv)>2 else 1\nprint(a*b)\n"
        }
        "clamp01" => {
            "import sys\nx=float(sys.argv[1]) if len(sys.argv)>1 else 0\nprint(max(0.0,min(1.0,x)))\n"
        }
        "rolling_mean" => {
            "import sys\nxs=[float(x) for x in sys.argv[1:]]\nprint(0 if not xs else sum(xs)/len(xs))\n"
        }
        _ => return false,
    };
    if save_tool(tool_name, source).is_err() {
        return false;
    }
    match tool_name {
        "double" => {
            let r = run_tool("double", "21");
            r.ok && r.stdout.trim() == "42"
        }
        "add" => {
            let r = run_tool("add", "20 22");
            r.ok && r.stdout.trim() == "42"
        }
        "fsot_sign" => {
            let r = run_tool("fsot_sign", "0.5");
            r.ok && r.stdout.trim() == "emergence"
        }
        "multiply" => {
            let r = run_tool("multiply", "6 7");
            r.ok && r.stdout.trim() == "42"
        }
        "clamp01" => {
            let r = run_tool("clamp01", "1.5");
            r.ok && (r.stdout.trim() == "1.0" || r.stdout.trim() == "1")
        }
        "rolling_mean" => {
            let r = run_tool("rolling_mean", "1 2 3 4");
            r.ok && (r.stdout.trim() == "2.5" || r.stdout.trim().starts_with("2.5"))
        }
        _ => false,
    }
}

/// Freeform tool invention under pressure — organism "grows" a verified organ-tool.
pub fn invent_tool_exam(seed: usize) -> Option<String> {
    let catalog = ["multiply", "clamp01", "rolling_mean", "double", "add", "fsot_sign"];
    let name = catalog[seed % catalog.len()];
    if forge_tool_exam(name) {
        Some(name.to_string())
    } else {
        None
    }
}

// ─── JS sandbox (parallel interpretable organ) ───────────────────────────────

fn find_node() -> Option<&'static str> {
    static CACHED: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    CACHED
        .get_or_init(|| {
            if Command::new("node")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                Some("node".into())
            } else {
                None
            }
        })
        .as_ref()
        .map(|s| s.as_str())
}

pub fn node_available() -> bool {
    find_node().is_some()
}

pub fn run_js(source: &str, timeout_secs: u64) -> ExecResult {
    let timeout_secs = timeout_secs.min(2);
    let Some(node) = find_node() else {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: "node not available".into(),
            exit_code: -1,
            timed_out: false,
        };
    };
    let _ = ensure_workspace();
    // Unique scratch name (avoid collisions if multiple exams in one process)
    let uniq = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let file = workspace_root()
        .join("scratch")
        .join(format!("run_{}_{}.js", std::process::id(), uniq));
    if let Err(e) = fs::write(&file, source) {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: format!("write failed: {e}"),
            exit_code: -1,
            timed_out: false,
        };
    }
    let mut cmd = Command::new(node);
    cmd.arg(&file)
        .current_dir(workspace_root())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&file);
            return ExecResult {
                ok: false,
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: -1,
                timed_out: false,
            };
        }
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let out = child.wait_with_output();
        let _ = tx.send(out);
    });
    let result = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(output)) => ExecResult {
            ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            timed_out: false,
        },
        Ok(Err(e)) => ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
            timed_out: false,
        },
        Err(_) => ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: "timeout".into(),
            exit_code: -1,
            timed_out: true,
        },
    };
    let _ = handle.join();
    let _ = fs::remove_file(&file);
    result
}

pub fn js_challenges() -> &'static [CodeChallenge] {
    // Graded ladder: arithmetic → string → array → FSOT-flavored logic
    &[
        CodeChallenge {
            id: "js_add",
            source: "console.log(2+40)\n",
            expect_stdout: "42",
        },
        CodeChallenge {
            id: "js_pow",
            source: "console.log(2**8)\n",
            expect_stdout: "256",
        },
        CodeChallenge {
            id: "js_lower",
            source: "console.log('FSOT'.toLowerCase())\n",
            expect_stdout: "fsot",
        },
        CodeChallenge {
            id: "js_mul",
            source: "console.log(6*7)\n",
            expect_stdout: "42",
        },
        CodeChallenge {
            id: "js_len",
            source: "console.log('trinary'.length)\n",
            expect_stdout: "7",
        },
        CodeChallenge {
            id: "js_sum_arr",
            source: "console.log([1,2,3,4].reduce((a,b)=>a+b,0))\n",
            expect_stdout: "10",
        },
        CodeChallenge {
            id: "js_max",
            source: "console.log(Math.max(3,9,2))\n",
            expect_stdout: "9",
        },
        CodeChallenge {
            id: "js_sign",
            source: "const x=0.3; console.log(x>0?'emergence':x<0?'damping':'neutral')\n",
            expect_stdout: "emergence",
        },
        CodeChallenge {
            id: "js_clamp",
            source: "const x=1.5; console.log(Math.max(0,Math.min(1,x)))\n",
            expect_stdout: "1",
        },
        CodeChallenge {
            id: "js_join",
            source: "console.log(['A','T','G'].join(''))\n",
            expect_stdout: "ATG",
        },
        CodeChallenge {
            id: "js_abs",
            source: "console.log(Math.abs(-7))\n",
            expect_stdout: "7",
        },
        CodeChallenge {
            id: "js_floor",
            source: "console.log(Math.floor(3.9))\n",
            expect_stdout: "3",
        },
    ]
}

/// Easy subset for cold organ scaffolding (first 6).
pub fn js_challenges_scaffold() -> &'static [CodeChallenge] {
    let all = js_challenges();
    &all[..6.min(all.len())]
}

/// Mastery bar for the JS organ (aligned with knowledge standard).
pub const JS_TARGET: f64 = 0.85;

pub fn run_js_exam(challenge: &CodeChallenge) -> bool {
    if !node_available() {
        return false;
    }
    let r = run_js(challenge.source, 3);
    if !r.ok {
        return false;
    }
    let out = r.stdout.trim();
    if out == challenge.expect_stdout {
        return true;
    }
    // float / int print variants
    if let (Ok(a), Ok(b)) = (out.parse::<f64>(), challenge.expect_stdout.parse::<f64>()) {
        return (a - b).abs() < 1e-9;
    }
    false
}

pub fn save_js_tool(name: &str, source: &str) -> std::io::Result<PathBuf> {
    let root = ensure_workspace()?;
    let safe: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let path = root.join("tools_js").join(format!("{safe}.js"));
    fs::write(&path, source)?;
    Ok(path)
}

pub fn run_js_tool(name: &str, args: &str) -> ExecResult {
    let path = workspace_root().join("tools_js").join(format!("{name}.js"));
    if !path.exists() {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: format!("js tool not found: {name}"),
            exit_code: -1,
            timed_out: false,
        };
    }
    let Some(node) = find_node() else {
        return ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: "node not available".into(),
            exit_code: -1,
            timed_out: false,
        };
    };
    let mut cmd = Command::new(node);
    cmd.arg(&path);
    for a in args.split_whitespace() {
        cmd.arg(a);
    }
    cmd.current_dir(workspace_root())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match cmd.output() {
        Ok(output) => ExecResult {
            ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            timed_out: false,
        },
        Err(e) => ExecResult {
            ok: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
            timed_out: false,
        },
    }
}

fn js_out_matches(out: &str, expect: &str) -> bool {
    let o = out.trim();
    let e = expect.trim();
    if o == e {
        return true;
    }
    if let (Ok(a), Ok(b)) = (o.parse::<f64>(), e.parse::<f64>()) {
        return (a - b).abs() < 1e-9;
    }
    false
}

/// Forge JS organ tools under pressure (full tools_js inheritance path).
/// Uses argv.slice(2) so tool scripts are robust to how node is invoked.
pub fn forge_js_tool_exam(tool_name: &str) -> bool {
    if !node_available() {
        return false;
    }
    let source = match tool_name {
        "clamp" => {
            "const a=process.argv.slice(2).map(Number);\nconst x=a[0]||0, lo=a[1]||0, hi=(a[2]===undefined?1:a[2]);\nconsole.log(Math.max(lo,Math.min(hi,x)));\n"
        }
        "double" => {
            "const x=Number(process.argv[2]||0);\nconsole.log(2*x);\n"
        }
        "add" => {
            "const a=Number(process.argv[2]||0), b=Number(process.argv[3]||0);\nconsole.log(a+b);\n"
        }
        "fsot_sign" => {
            "const x=Number(process.argv[2]||0);\nconsole.log(x>0?'emergence':x<0?'damping':'neutral');\n"
        }
        "mul" => {
            "const a=Number(process.argv[2]||1), b=Number(process.argv[3]||1);\nconsole.log(a*b);\n"
        }
        _ => return false,
    };
    if save_js_tool(tool_name, source).is_err() {
        return false;
    }
    match tool_name {
        "clamp" => {
            let r = run_js_tool("clamp", "1.5 0 1");
            r.ok && js_out_matches(&r.stdout, "1")
        }
        "double" => {
            let r = run_js_tool("double", "21");
            r.ok && js_out_matches(&r.stdout, "42")
        }
        "add" => {
            let r = run_js_tool("add", "20 22");
            r.ok && js_out_matches(&r.stdout, "42")
        }
        "fsot_sign" => {
            let r = run_js_tool("fsot_sign", "0.5");
            r.ok && r.stdout.trim() == "emergence"
        }
        "mul" => {
            let r = run_js_tool("mul", "6 7");
            r.ok && js_out_matches(&r.stdout, "42")
        }
        _ => false,
    }
}

pub fn list_js_tools() -> Vec<String> {
    let dir = workspace_root().join("tools_js");
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|x| x.to_str()) == Some("js") {
                if let Some(s) = e.path().file_stem().and_then(|s| s.to_str()) {
                    out.push(s.to_string());
                }
            }
        }
    }
    out.sort();
    out
}

pub fn list_tools() -> Vec<String> {
    let dir = workspace_root().join("tools");
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|x| x.to_str()) == Some("py") {
                if let Some(s) = e.path().file_stem().and_then(|s| s.to_str()) {
                    out.push(s.to_string());
                }
            }
        }
    }
    out.sort();
    out
}

#[allow(dead_code)]
pub fn practice_path() -> PathBuf {
    workspace_root().join("practice")
}

pub fn write_practice_snippet(name: &str, source: &str) -> std::io::Result<PathBuf> {
    let dir = ensure_workspace()?.join("practice");
    let p = dir.join(format!("{name}.py"));
    fs::write(&p, source)?;
    Ok(p)
}

pub fn python_available() -> bool {
    let r = run_python("print('ok')\n", 2);
    r.ok && r.stdout.contains("ok")
}
