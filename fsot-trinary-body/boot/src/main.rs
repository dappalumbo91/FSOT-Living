//! Build UEFI/BIOS images for fsot_body_kernel and optionally run QEMU.
//!
//!   $env:KERNEL_ELF = "...\fsot_body_kernel"
//!   cargo run -p fsot_body_boot -- --run

use std::env;
use std::path::PathBuf;
use std::process::{exit, Command};

fn main() {
    let kernel_path = match env::var("KERNEL_ELF") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            eprintln!("KERNEL_ELF is not set.");
            eprintln!("Build the kernel, then:");
            eprintln!(
                "  $env:KERNEL_ELF = \"...\\target\\x86_64-unknown-none\\debug\\fsot_body_kernel\""
            );
            eprintln!("  cargo run -p fsot_body_boot -- --run");
            exit(2);
        }
    };
    if !kernel_path.exists() {
        eprintln!("kernel ELF not found: {}", kernel_path.display());
        exit(2);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| {
        env::current_dir()
            .expect("cwd")
            .join("target/body_image")
            .to_string_lossy()
            .into_owned()
    }));
    std::fs::create_dir_all(&out_dir).expect("mkdir image dir");

    let uefi_img = out_dir.join("fsot_body_uefi.img");

    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_img)
        .expect("UEFI image build");

    println!("UEFI image: {}", uefi_img.display());

    let run = env::args().any(|a| a == "--run");
    if !run {
        return;
    }

    // UEFI body requires OVMF firmware.
    let ovmf = env::var("OVMF_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            // Common Windows install locations (optional).
            let candidates = [
                r"C:\Program Files\qemu\share\edk2-x86_64-code.fd",
                r"C:\Program Files\qemu\share\OVMF.fd",
            ];
            candidates.iter().map(PathBuf::from).find(|p| p.exists())
        });

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-machine").arg("q35");
    cmd.arg("-m").arg("256M");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-display").arg("none");
    cmd.arg("-no-reboot");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={}", uefi_img.display()));

    if let Some(code) = ovmf {
        println!("QEMU UEFI with firmware: {}", code.display());
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            code.display()
        ));
    } else {
        eprintln!("WARNING: OVMF_PATH not set and no QEMU edk2 found.");
        eprintln!("Image built; set OVMF_PATH then re-run with --run for live serial.");
        eprintln!("Example: $env:OVMF_PATH = 'C:\\path\\to\\OVMF_CODE.fd'");
        return;
    }

    let status = cmd.status().expect("failed to spawn qemu-system-x86_64");
    exit(status.code().unwrap_or(1));
}
