//! FSOT Living Organism — Rust native TUI
//!
//! Run (recommended):
//!   start_fsot_living.bat
//! or:
//!   cargo run --release

mod accuracy_homeo;
mod activity;
mod audio;
mod behavior_monitor;
mod bench;
mod body_bridge;
mod cell_tracking;
mod capability;
mod code_env;
mod constraints;
mod export;
mod fsot_tool;
mod genetic;
mod habitat;
mod homeostasis;
mod knowledge;
mod lang_plastic;
mod language;
mod lean_ground;
mod organism;
mod pack;
mod proposals;
mod query;
mod resume;
mod scalar;
mod self_model;
mod ui;
mod video;
mod vision;

fn main() {
    // Self-check (also proves console output works)
    let q = scalar::compute_for_domain("quantum");
    let c = scalar::compute_for_domain("cosmological");
    eprintln!(
        "FSOT scalar: quantum S={:.4} ({}) | cosmological S={:.4} ({}) | k={:.6}",
        q.s,
        q.regime,
        c.s,
        c.regime,
        scalar::constants::k()
    );
    let lean = lean_ground::check_runtime_invariants();
    eprintln!("{}", lean.summary_line());
    eprintln!("Starting TUI…");
    eprintln!("  Type naturally + Enter = talk to champion (language organ)");
    eprintln!("  Esc = quit | Space = pause (when not in input) | Tab = focus input");
    eprintln!("  Continuity: LIVE_STATUS + LATEST germline + identity LTM resume\n");

    if let Err(e) = ui::run_tui() {
        eprintln!("\n========== FSOT Living ERROR ==========");
        eprintln!("{e:?}");
        eprintln!("=======================================");
        eprintln!("\nCommon causes:");
        eprintln!("  - Must run inside a real console (cmd.exe / Windows Terminal)");
        eprintln!("  - Do not double-click if antivirus blocks console apps");
        eprintln!("  - Use start_fsot_living.bat from this folder");
        wait_for_key();
        std::process::exit(1);
    }
}

fn wait_for_key() {
    eprintln!("\nPress Enter to close…");
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
}
