//! Host probe — full body stack without QEMU.
//!
//! Exercises: trinary fabric, interoception, Mind ABI, organ membrane.

use body::{SensorInputs, Soma};
use trinary::{collapse, Trit, Word27};

fn main() {
    println!("=== FSOT Trinary Body — full stack host probe ===");
    println!("Layers: trinary | genetic | interoception | Mind ABI | organ membrane\n");

    println!("[trinary] collapse:");
    for v in [-0.5, 0.0, 0.5] {
        let t = collapse(v);
        println!("  {:+.1} → {} ({})", v, t.name(), t.to_i8());
    }
    let w = Word27::encode(42, 27);
    println!("[trinary] Word27(42) decode={}", w.decode());

    let mut soma = Soma::boot();
    println!("\n[soma] booted genome_fp=0x{:016x}", soma.fabric.fingerprint());
    println!("[soma] membrane: {}", soma.membrane.list_line());

    // --- Interoception + genetic pressure ticks ---
    println!("\n[intero] running 24 ticks…");
    for i in 0..24 {
        let sensors = SensorInputs {
            cpu_temp_norm: 0.7 + (i as f64) * 0.005,
            tsc_delta: 1.0e6 + (i as f64) * 1.0e4,
            heap_used: 50_000 + i * 3_000,
            heap_cap: 512 * 1024,
            uptime_seconds: 0.0,
            breath_phase: 0.0,
        };
        soma.tick(sensors);
        for line in soma.drain_outbox() {
            if line.starts_with("MIND") || line.starts_with("BODY") {
                println!("  {}", line);
            }
        }
    }

    // --- Mind ABI verbs ---
    println!("\n[mind_abi] MOTOR dialogue:");
    let cmds = [
        "MOTOR STATUS",
        "MOTOR ATTEND heap",
        "MOTOR EXPRESS",
        "MOTOR ORGANS",
        "MOTOR READ /body/genome",
        "MOTOR WRITE /organs/knowledge living habitat bridge ready",
        "MOTOR READ /organs/knowledge",
        "MOTOR NOTE cortex_ack=1",
        "MOTOR BREATHE",
        "MOTOR REPORT",
        "MOTOR REPRESS",
    ];
    for c in cmds {
        println!("  >> {}", c);
        soma.handle_line(c);
        for line in soma.drain_outbox() {
            println!("  << {}", line);
        }
        // Advance body so EXPRESS/REPRESS pressure applies.
        soma.tick(SensorInputs {
            cpu_temp_norm: 0.75,
            tsc_delta: 1.2e6,
            heap_used: 80_000,
            heap_cap: 512 * 1024,
            uptime_seconds: 0.0,
            breath_phase: 0.0,
        });
        for line in soma.drain_outbox() {
            if line.starts_with("MIND") {
                println!("  << {}", line);
            }
        }
    }

    let (d, m, u) = soma.fabric.phenotype_tally();
    println!(
        "\n[assert] phenotype repress={} latent={} express={} plast={:.3}",
        d,
        m,
        u,
        soma.fabric.mean_plasticity()
    );
    println!(
        "[assert] knowledge organ writes={} bridge writes={}",
        soma.membrane.get(body::OrganId::Knowledge).writes,
        soma.membrane.get(body::OrganId::Bridge).writes
    );

    let ok = d + m + u == soma.fabric.loci.len()
        && soma.fabric.mean_plasticity() > 0.3
        && soma.membrane.get(body::OrganId::Knowledge).writes >= 1
        && soma.intero.latest().is_some();

    // Touch three-state vocabulary
    let _ = (Trit::SpinDown, Trit::Superposed, Trit::SpinUp);

    if ok {
        println!("\nFULL STACK: OK — interoception + Mind ABI + organ membrane live");
        println!("Next: cargo run -p mind_bridge");
        println!("      cargo build -p fsot_body_kernel --target x86_64-unknown-none");
    } else {
        eprintln!("\nFULL STACK: FAIL");
        std::process::exit(1);
    }
}
