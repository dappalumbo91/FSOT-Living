//! Mind bridge — body host for cortex (living habitat / tools).
//!
//! ## Transports
//!
//! | Mode | Latency | Use |
//! |------|---------|-----|
//! | **TCP** (`--tcp`) | ~ms real-time | Live companion loop (preferred) |
//! | **Files** | 10–100ms+, disk | Debug, crash recovery, offline review |
//!
//! Same Mind ABI lines either way: `MIND` / `MOTOR` / `ACK` / …
//!
//! ## Usage
//!
//! ```text
//! cargo run -p mind_bridge -- --tcp              # listen 127.0.0.1:7368 forever
//! cargo run -p mind_bridge -- --tcp 7370         # custom port
//! cargo run -p mind_bridge -- 40                 # file-only, 40 ticks then exit
//! cargo run -p mind_bridge -- --tcp --ticks 100  # TCP + exit after 100 ticks
//! ```
//!
//! Env:
//!   FSOT_BRIDGE_DIR  — file state dir (also writes status while TCP runs)
//!   FSOT_BODY_PORT   — default TCP port (7368)

mod home_fs;
mod probe;

use body::{SensorInputs, Soma};
use std::collections::VecDeque;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DEFAULT_PORT: u16 = 7368;

fn default_bridge_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("../state/habitat-rust"),
        PathBuf::from("../../state/habitat-rust"),
        PathBuf::from("../../../state/habitat-rust"),
        PathBuf::from("bridge_state"),
    ];
    for c in candidates {
        if c.exists() || c.parent().map(|p| p.exists()).unwrap_or(false) {
            if let Some(parent) = c.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::create_dir_all(&c);
            if c.exists() {
                return c;
            }
        }
    }
    let fallback = PathBuf::from("bridge_state");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

fn append_line(path: &Path, line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{}", line);
    }
}

fn write_status(path: &Path, soma: &Soma, last_mind: &str) {
    let (d, m, u) = soma.fabric.phenotype_tally();
    let g = soma.fabric.guidance();
    let organs = soma.membrane.list_line();
    let json = format!(
        "{{\n  \"tick\": {},\n  \"genome_fp\": \"0x{:016x}\",\n  \"delta_psi\": {:.4},\n  \"d_eff\": {:.2},\n  \"emergence\": {:.4},\n  \"repress\": {},\n  \"latent\": {},\n  \"express\": {},\n  \"mean_expr\": {:.4},\n  \"plasticity\": {:.4},\n  \"breath_count\": {},\n  \"focus\": {},\n  \"motor_bias\": {:.4},\n  \"organs\": {:?},\n  \"last_mind\": {:?},\n  \"transport\": \"tcp_or_file\",\n  \"scale_tier\": {:?},\n  \"ram_mb\": {},\n  \"vram_mb\": {},\n  \"consolidations\": {},\n  \"breath_step\": {:.4}\n}}\n",
        soma.tick,
        soma.fabric.fingerprint(),
        g.delta_psi,
        g.d_eff,
        g.emergence_preference,
        d,
        m,
        u,
        soma.fabric.mean_expression(),
        soma.fabric.mean_plasticity(),
        soma.intero.breath_count,
        soma.intero.focus,
        soma.motor_bias,
        organs,
        last_mind,
        soma.scale.tier.as_str(),
        soma.scale.ram_mb,
        soma.scale.vram_mb,
        soma.consolidations,
        soma.scale.breath_step
    );
    let _ = fs::write(path, json);
}

fn flush_ltm(bridge: &Path, soma: &mut Soma) {
    if let Some(text) = soma.take_pending_ltm() {
        let ltm = bridge.join("BODY_LTM.txt");
        let prev = bridge.join("BODY_LTM.prev.txt");
        if ltm.exists() {
            let _ = fs::copy(&ltm, &prev);
        }
        if fs::write(&ltm, &text).is_ok() {
            println!(
                "[ltm] consolidated → {} ({} bytes)",
                ltm.display(),
                text.len()
            );
        }
    }
}

/// Pull `key=value` tokens; `genome=` may contain hyphens so it is last-wins rest-of-line.
fn parse_kv_token<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    // Prefer whitespace-delimited token for simple keys
    if key != "genome" {
        return line
            .split_whitespace()
            .find_map(|p| p.strip_prefix(&prefix));
    }
    // genome= may be the rest of the line (codon string with hyphens)
    if let Some(idx) = line.find(&prefix) {
        let rest = &line[idx + prefix.len()..];
        // stop at next known key if any (defensive)
        let end = [" gen=", " hero=", " fit="]
            .iter()
            .filter_map(|k| rest.find(k))
            .min()
            .unwrap_or(rest.len());
        let g = rest[..end].trim();
        if g.is_empty() {
            None
        } else {
            Some(g)
        }
    } else {
        None
    }
}

fn apply_motor_side_effects(bridge: &Path, soma: &mut Soma, line: &str) {
    // Genome writeback: MOTOR WRITE /home/genome gen=… hero=… fit=… genome=…
    // Disk archive + live fabric reseed (mind selection → body germline couple).
    let is_genome_write = line.contains("WRITE /home/genome")
        || (line.contains("/home/genome") && line.contains("genome="));
    if is_genome_write {
        let gen: u32 = parse_kv_token(line, "gen")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let hero = parse_kv_token(line, "hero").unwrap_or("unknown");
        let fit: f64 = parse_kv_token(line, "fit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let genome = parse_kv_token(line, "genome").unwrap_or("").trim();
        if !genome.is_empty() {
            match home_fs::register_genome(bridge, gen, hero, genome, fit) {
                Ok(p) => println!(
                    "  [home] genome archive gen={gen} hero={hero} → {}",
                    p.display()
                ),
                Err(e) => eprintln!("  [home] genome archive FAILED gen={gen}: {e}"),
            }
            if soma.apply_germline(genome, gen, hero, fit) {
                println!(
                    "  [germline] fabric reseed gen={gen} hero={hero} fp=0x{:016x}",
                    soma.fabric.fingerprint()
                );
            }
        } else {
            eprintln!("  [home] genome write missing genome= payload: {line}");
        }
    }
    if line.contains("WRITE /home/tools") {
        let skill = line
            .split_whitespace()
            .find_map(|p| p.strip_prefix("skill=").or_else(|| p.strip_prefix("js_skill=")))
            .unwrap_or("tool");
        home_fs::register_tool(bridge, skill, "living_cortex");
    }
    if line.contains("WRITE /home/proposals") {
        home_fs::register_proposal(bridge, "batch", line);
    }
    if line.contains("WRITE /home/knowledge") {
        if let Some(rest) = line.split("/home/knowledge").nth(1) {
            home_fs::append_knowledge_fact(bridge, rest.trim());
        }
    }
}

fn drain_motor_file(path: &Path, bridge: &Path, soma: &mut Soma) -> usize {
    if !path.exists() {
        return 0;
    }
    let mut text = String::new();
    match fs::File::open(path).and_then(|mut f| f.read_to_string(&mut text)) {
        Ok(_) => {}
        Err(e) => {
            // Windows share races: retry once
            thread::sleep(Duration::from_millis(15));
            if fs::File::open(path)
                .and_then(|mut f| {
                    text.clear();
                    f.read_to_string(&mut text)
                })
                .is_err()
            {
                eprintln!("  [motor/file] open failed: {e}");
                return 0;
            }
        }
    }
    if text.trim().is_empty() {
        return 0;
    }
    let mut n = 0;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Truncate log for huge genome lines
        if line.len() > 140 {
            println!("  [motor/file] {}…", &line[..140]);
        } else {
            println!("  [motor/file] {}", line);
        }
        soma.set_principal(body::Principal::Mind);
        apply_motor_side_effects(bridge, soma, line);
        soma.handle_line(line);
        n += 1;
    }
    // Truncate only after successful process (rename swap if possible)
    let tmp = path.with_extension("txt.drained");
    let _ = fs::write(&tmp, "");
    if fs::rename(&tmp, path).is_err() {
        let _ = fs::write(path, "");
    }
    if n > 0 {
        println!("  [motor/file] drained {n} command(s)");
    }
    n
}

fn drain_motor_queue(q: &Arc<Mutex<VecDeque<String>>>, bridge: &Path, soma: &mut Soma) -> usize {
    let mut n = 0;
    let mut guard = match q.lock() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    while let Some(line) = guard.pop_front() {
        let line = line.trim().to_string();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        println!("  [motor/tcp] {}", line);
        // Cortex traffic is Mind principal (Human uses MOTOR AS human …)
        soma.set_principal(body::Principal::Mind);
        apply_motor_side_effects(bridge, soma, &line);
        soma.handle_line(&line);
        n += 1;
    }
    n
}

type ClientList = Arc<Mutex<Vec<TcpStream>>>;

fn spawn_accept_loop(listener: TcpListener, clients: ClientList, motor_q: Arc<Mutex<VecDeque<String>>>) {
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                    println!("[tcp] cortex connected: {:?}", stream.peer_addr());
                    // Clone first: `writer` for broadcast, `stream` for reader thread.
                    let mut writer = match stream.try_clone() {
                        Ok(w) => w,
                        Err(e) => {
                            eprintln!("[tcp] clone failed: {e}");
                            continue;
                        }
                    };
                    let _ = writeln!(writer, "BODY event=WELCOME transport=tcp");
                    let _ = writer.flush();
                    let q = Arc::clone(&motor_q);
                    thread::spawn(move || {
                        let mut reader = BufReader::new(stream);
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match reader.read_line(&mut line) {
                                Ok(0) => break,
                                Ok(_) => {
                                    let t = line.trim();
                                    if !t.is_empty() {
                                        if let Ok(mut g) = q.lock() {
                                            if g.len() < 256 {
                                                g.push_back(t.to_string());
                                            }
                                        }
                                    }
                                }
                                Err(ref e)
                                    if e.kind() == std::io::ErrorKind::WouldBlock
                                        || e.kind() == std::io::ErrorKind::TimedOut =>
                                {
                                    thread::sleep(Duration::from_millis(5));
                                }
                                Err(_) => break,
                            }
                        }
                        println!("[tcp] cortex reader closed");
                    });
                    if let Ok(mut g) = clients.lock() {
                        g.retain(|s| s.peer_addr().is_ok());
                        if g.len() < 4 {
                            g.push(writer);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[tcp] accept error: {e}");
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
}

fn broadcast(clients: &ClientList, line: &str) {
    let payload = format!("{line}\n");
    let bytes = payload.as_bytes();
    if let Ok(mut g) = clients.lock() {
        g.retain_mut(|stream| {
            match stream.write_all(bytes).and_then(|_| stream.flush()) {
                Ok(()) => true,
                Err(_) => false,
            }
        });
    }
}

fn parse_args() -> (bool, u16, Option<u64>) {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tcp = false;
    let mut port = env::var("FSOT_BODY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let mut ticks: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tcp" => {
                tcp = true;
                if let Some(next) = args.get(i + 1) {
                    if let Ok(p) = next.parse::<u16>() {
                        port = p;
                        i += 1;
                    }
                }
            }
            "--ticks" => {
                if let Some(next) = args.get(i + 1) {
                    ticks = next.parse().ok();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!(
                    "mind_bridge [--tcp [port]] [--ticks N] [N]\n\
                     Default port {DEFAULT_PORT}. File state in FSOT_BRIDGE_DIR."
                );
                std::process::exit(0);
            }
            other => {
                if let Ok(n) = other.parse::<u64>() {
                    ticks = Some(n);
                }
            }
        }
        i += 1;
    }
    // Bare number without --tcp → file-only finite run
    if !tcp && ticks.is_none() {
        ticks = Some(40);
    }
    (tcp, port, ticks)
}

fn main() {
    let (tcp, port, ticks) = parse_args();

    let bridge = env::var("FSOT_BRIDGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_bridge_dir());
    let _ = fs::create_dir_all(&bridge);

    let telem = bridge.join("BODY_TELEMETRY.jsonl");
    let outbox_path = bridge.join("BODY_OUTBOX.txt");
    let motor_path = bridge.join("BODY_MOTOR.txt");
    let status_path = bridge.join("BODY_STATUS.json");
    let ltm_path = bridge.join("BODY_LTM.txt");

    let motor_q: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

    let scale = probe::probe_scale();
    println!("=== FSOT Mind Bridge ===");
    println!("{}", scale.summary_line());
    println!(
        "memory: RAM working set · TCP nerves · disk LTM ({})",
        ltm_path.display()
    );

    if tcp {
        let addr = format!("127.0.0.1:{port}");
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                let _ = listener.set_nonblocking(false);
                println!("transport: TCP real-time @ {addr}");
                println!("files also written → {}", bridge.display());
                spawn_accept_loop(listener, Arc::clone(&clients), Arc::clone(&motor_q));
            }
            Err(e) => {
                eprintln!("FATAL: cannot bind {addr}: {e}");
                eprintln!("Is another mind_bridge already running?");
                std::process::exit(1);
            }
        }
    } else {
        println!("transport: files only");
        println!("bridge_dir: {}", bridge.display());
    }

    if let Some(n) = ticks {
        println!("ticks: {n} (then exit)");
    } else {
        println!("ticks: infinite (Ctrl+C to stop)");
    }

    let mut soma = Soma::boot_with_scale(scale.clone());
    // Biological home VFS mount (Linux $HOME salvage)
    home_fs::ensure_home_tree(&bridge);
    home_fs::load_home_into_soma(&bridge, &mut soma);
    println!("[home] VFS mounted at {}", home_fs::home_root(&bridge).display());

    // Wake from disk LTM if present (long-term memory rehydrate — phenotypic continuity)
    if ltm_path.exists() {
        if let Ok(text) = fs::read_to_string(&ltm_path) {
            if soma.apply_wake_text(&text) {
                println!(
                    "[ltm] woke from {} (tick={})",
                    ltm_path.display(),
                    soma.tick
                );
            }
        }
    }

    // Champion germline from living mind (lineage DNA continuity across boots)
    if let Some(g) = home_fs::load_latest_germline(&bridge) {
        if soma.apply_germline(&g.genome, g.gen, &g.hero, g.fit) {
            println!(
                "[germline] LATEST gen={} hero={} fit={:.3} codons={} fp=0x{:016x}",
                g.gen,
                g.hero,
                g.fit,
                soma.fabric.codons.len(),
                soma.fabric.fingerprint()
            );
        } else {
            eprintln!(
                "[germline] LATEST present but reseed failed (gen={} hero={})",
                g.gen, g.hero
            );
        }
    } else {
        println!("[germline] no LATEST.txt yet — zygote default fabric");
    }

    // Compact identity narrative (preserve core:, keep recent sessions)
    soma.set_principal(body::Principal::Mind);
    soma.handle_line("MOTOR DEFRAG /home/identity");
    soma.handle_line(&format!(
        "MOTOR APPEND /home/identity session: boot_wake scale={} tcp={}",
        scale.tier.as_str(),
        tcp
    ));

    let mut last_mind = String::new();
    append_line(
        &telem,
        &format!(
            "{{\"event\":\"bridge_start\",\"tcp\":{},\"tier\":{:?},\"genome_fp\":\"0x{:016x}\"}}",
            tcp,
            scale.tier.as_str(),
            soma.fabric.fingerprint()
        ),
    );

    if tcp {
        broadcast(
            &clients,
            &format!(
                "BODY event=ONLINE transport=tcp tier={} genome_fp=0x{:016x}",
                scale.tier.as_str(),
                soma.fabric.fingerprint()
            ),
        );
        broadcast(&clients, &scale.summary_line());
    }

    let sleep_ms = scale.tick_sleep_ms.max(10);
    let mut i = 0u64;
    loop {
        if let Some(max) = ticks {
            if i >= max {
                break;
            }
        }
        i += 1;

        drain_motor_file(&motor_path, &bridge, &mut soma);
        drain_motor_queue(&motor_q, &bridge, &mut soma);

        let sensors = SensorInputs {
            cpu_temp_norm: 0.72,
            tsc_delta: 1.0e6 + (i as f64) * 5.0e3,
            heap_used: 40_000 + (i as usize % 20) * 2000,
            heap_cap: 512 * 1024,
            uptime_seconds: 0.0,
            breath_phase: 0.0,
        };
        soma.tick(sensors);
        flush_ltm(&bridge, &mut soma);
        // Periodic home disk sync (biological systems consolidation)
        if i % scale.consolidate_every.max(16) == 0 {
            home_fs::save_home_from_soma(&bridge, &soma);
        }

        let mut out_dump = String::new();
        for line in soma.drain_outbox() {
            out_dump.push_str(&line);
            out_dump.push('\n');
            if tcp {
                broadcast(&clients, &line);
            }
            if line.starts_with("MIND") {
                last_mind = line.clone();
                if ticks.is_some() || i % 4 == 0 {
                    append_line(&telem, &format!("{{\"t\":{},\"line\":{:?}}}", soma.tick, line));
                }
            } else if line.starts_with("ACK")
                || line.starts_with("DATA")
                || line.starts_with("ORGANS")
                || line.starts_with("REPORT")
                || line.starts_with("BODY event=SLEEP")
                || line.starts_with("BODY event=WAKE")
                || line.starts_with("SCALE")
            {
                if ticks.is_some() || i % 2 == 0 {
                    append_line(
                        &telem,
                        &format!("{{\"t\":{},\"line\":{:?}}}", soma.tick, line),
                    );
                }
            }
            if !tcp || i % 8 == 0 || line.starts_with("BODY event=SLEEP") {
                println!("{}", line);
            }
        }
        if !out_dump.is_empty() {
            let _ = fs::write(&outbox_path, &out_dump);
        }
        write_status(&status_path, &soma, &last_mind);

        thread::sleep(Duration::from_millis(sleep_ms));
    }

    // Final checkpoint to disk LTM + home VFS
    soma.handle_line("MOTOR SAVE");
    flush_ltm(&bridge, &mut soma);
    home_fs::save_home_from_soma(&bridge, &soma);
    for line in soma.drain_outbox() {
        println!("{}", line);
    }

    println!("\nbridge stop after {i} ticks · consolidations={}", soma.consolidations);
    println!("status: {}", status_path.display());
    println!("ltm:    {}", ltm_path.display());
    println!("home:   {}", home_fs::home_root(&bridge).display());
}
