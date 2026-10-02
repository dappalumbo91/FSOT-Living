# FSOT Trinary Body (QEMU companion OS)

**Ground-up hardware body** for the living FSOT mind gym (`../fsot-living-rust`).

This is **not** a port of Linux and **not** a binary OS with static flags.
It uses Linux *as an informational map of what a layered machine needs*, then
rebuilds each layer as **FSOT biology + dynamic trinary genetic states**.

| Role | Path |
|------|------|
| Mind gym (working) | `../fsot-living-rust` |
| Body (this tree) | `fsot-trinary-body` |
| Prior art (reference only) | `<local folder, not included in repo: Fsot trinary/fsot_os>` |

## Core law

- **State is trinary:** SpinDown (−1) · Superposed (0) · SpinUp (+1)
- **Genes are not bits.** Loci hold continuous fields that **collapse** under
  pressure and can **re-open** (plasticity). Binary static storage is rejected
  at the design level.
- **S = k · (T1 + T2 + T3)** — same scalar family as the living habitat.
- **Body first, mind later.** Boot → interoception → Mind ABI → VFS organs →
  attach living-fsot as cortex.

## Layer map (Linux → FSOT body)

See [docs/LINUX_TO_FSOT.md](docs/LINUX_TO_FSOT.md) and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

```
Hardware / QEMU          →  Body chassis (x86_64 UEFI)
Bootloader               →  Zygote (UEFI ignition)
Kernel core              →  Soma (scheduler = homeostasis pacemaker)
Trinary + genetic        →  Genome / expression fabric
Interoception            →  Vagus analogue (self-sensing)
Drivers / serial         →  Sensory–motor membrane
VFS / services           →  Organ membrane + knowledge store
Userspace / interpreters →  Living mind + Python/JS later
```

## Workspace layout

```
fsot-trinary-body/
  crates/
    trinary/     # Trit, Word27, gates, collapse (no_std)
    genetic/     # Dynamic loci, codons, dual-axis genome (no_std)
    kernel/      # Bare-metal soma (x86_64-unknown-none)
    host_probe/  # Host tests without QEMU
  boot/          # UEFI disk image + optional QEMU launch
  scripts/       # Windows run helpers
  docs/          # Architecture + Linux mapping
```

## Quick start (host — no QEMU required)

```powershell
cd "fsot-trinary-body"
cargo test -p trinary -p genetic
cargo run -p host_probe
```

## QEMU body boot

Pinned toolchain (see `rust-toolchain.toml`): **nightly-2026-02-01**  
Boot path is **UEFI-only** (BIOS stages disabled — more reliable on Windows).

```powershell
.\scripts\run_qemu.ps1
# or stepwise:
cargo build -p fsot_body_kernel --target x86_64-unknown-none
$env:KERNEL_ELF = (Resolve-Path target\x86_64-unknown-none\debug\fsot_body_kernel).Path
$env:OUT_DIR    = (Resolve-Path .).Path + "\target\body_image"
cargo run -p fsot_body_boot
# Live serial (needs OVMF + qemu-system-x86_64):
$env:OVMF_PATH = "C:\path\to\OVMF_CODE.fd"   # or QEMU edk2-x86_64-code.fd
cargo run -p fsot_body_boot -- --run
```

Disk image lands at: `target/body_image/fsot_body_uefi.img`

## Principles we will not violate

1. **No binary gene model** as the primary state (bits may only be *wire*
   packing of trits, never the semantic layer).
2. **Dynamic expression** — loci re-collapse; plasticity is first-class.
3. **Layered body** — do not flatten organs into one process “for speed.”
4. **Living habitat stays running** — mind gym is separate; Mind ABI bridges later.
5. **Open Linux knowledge informs structure**; FSOT + biology decide semantics.

## Mind ABI + closed loop (TCP preferred)

```powershell
cargo run -p host_probe
.\scripts\run_tcp_body.ps1          # real-time body on 127.0.0.1:7368
# other terminal → fsot-living-rust\start_fsot_living_lean.bat
.\scripts\run_qemu_serial.ps1       # bare-metal serial MOTOR
```

| Transport | Role |
|-----------|------|
| **TCP** | Live nerve (ms) — preferred companion link |
| **Files** | Flight recorder / fallback — always mirrored |

Docs: [MEMORY_TIERS.md](docs/MEMORY_TIERS.md) · [TRANSPORT_TCP_VS_FILES.md](docs/TRANSPORT_TCP_VS_FILES.md) · [MIND_ABI.md](docs/MIND_ABI.md) · [MIN_SPEC_MAC_MINI_2012.md](docs/MIN_SPEC_MAC_MINI_2012.md)

**Goldfish scale:** auto-probes RAM + NVIDIA VRAM. Omen 35L + RTX 5070 → `max` breath/pack. Force: `FSOT_SCALE=max` / `FSOT_LEAN=1`.

**Home VFS (keep your stuff):** `state/habitat-rust/home/{knowledge,tools,media,genome,proposals,identity}/`  
`MOTOR HOME` · `MOTOR SCHED` · bio map: [docs/BIO_LINUX_MAP.md](docs/BIO_LINUX_MAP.md)

**Desktop visual (host face):** `fsot-living-rust\start_desktop_viz.bat` → http://127.0.0.1:3850

## Status

- [x] Folder + workspace + architecture docs (Linux → FSOT map)
- [x] `trinary` + `genetic` crates — **dynamic loci, not binary-static**
- [x] Host tests + `host_probe` (pressure / rest / re-collapse)
- [x] Bare-metal `fsot_body_kernel` (`x86_64-unknown-none`)
- [x] UEFI disk image `fsot_body_uefi.img`
- [x] Interoception ring (temp/tsc/heap/uptime/breath → pressure + trits)
- [x] Mind ABI (`MOTOR` / `MIND` / `ACK` / `DATA`)
- [x] Organ membrane (`/body/*`, `/organs/*`)
- [x] `mind_bridge` file bridge to living habitat state dir
- [x] Living habitat `body_bridge` (couple pressure + MOTOR policy)
- [x] Kernel real heap + thermal MSR (QEMU-safe fallback)
- [x] QEMU interactive serial + closed-loop scripts

## Living habitat (unchanged)

Keep running the mind gym separately:

```powershell
cd ..\fsot-living-rust
.\start_fsot_living.bat
```

Body and mind stay layered; Mind ABI is the future join point.
