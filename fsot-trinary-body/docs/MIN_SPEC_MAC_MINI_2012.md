# Minimum hardware target: 2012 Mac Mini class

We design so the **biological + companion intelligence stack** runs on hardware no newer than a **late-2012 Mac Mini**.

## What that machine is (floor)

| Resource | Typical 2012 Mac Mini |
|----------|------------------------|
| CPU | Intel Core i5/i7 Ivy Bridge, 2–4 cores, ~2.5 GHz |
| RAM | **4 GB** stock (often 8–16 if upgraded) |
| GPU | Intel HD 4000 — **no CUDA, no ML GPU path required** |
| Disk | HDD or small SSD — avoid thrashing |
| OS | macOS (or Linux/Windows x86_64 of similar age) |
| Arch | **x86_64** (Intel) |

If it runs here, it runs on almost any laptop/desktop still in use.

## What we do **not** require

- Discrete GPU / CUDA / Metal acceleration  
- 16+ GB RAM for the mind gym  
- Cloud LLM cortex  
- 60 FPS video training loops  
- Multi-GB models in-process  

## Budget (targets)

| Component | RAM target | Notes |
|-----------|------------|--------|
| `mind_bridge --tcp` body host | **&lt; 50 MB** | Soma + trinary fabric only |
| QEMU body (optional) | **256–512 MB** guest | UEFI image, serial only |
| `fsot-living` mind gym | **&lt; 1.5 GB** lean | Pack + knowledge cap |
| Total comfortable on 4 GB | **~2 GB peak** | Leave room for OS |

## Lean mode (set on 4 GB hosts)

```bash
export FSOT_LEAN=1          # or FSOT_MIN_SPEC=1
# Windows PowerShell:
$env:FSOT_LEAN = "1"
```

Effects today:

- Pack size **4** organisms (default 6)  
- Knowledge FactCards capped ~**2000**  
- Smaller audio/video path indexes  
- TCP motor queue hard-capped; telemetry append throttled  

## Recommended run (2012 Mini)

```bash
# 1) Body host — always first
cd fsot-trinary-body
cargo run --release -p mind_bridge -- --tcp

# 2) Living cortex — lean
cd fsot-living-rust
export FSOT_LEAN=1
cargo run --release
```

Optional QEMU only when testing bare-metal:

```bash
# 256M guest is enough for current soma
qemu-system-x86_64 -machine q35 -m 256M -serial mon:stdio ...
```

## Build notes (macOS Intel)

- Rust stable/nightly per `rust-toolchain.toml` in each project  
- Body bootloader needs **nightly-2026-02-01** pin (see body README)  
- Living habitat builds on **stable** fine  
- Prefer `--release` on old CPUs  

## Companion OS vision (same min-spec)

Long-term you keep **your stuff** (files, knowledge, tools) in body organs / VFS like a home directory — not trapped in a chat log.  
The 2012 floor forces us to stay **efficient**: trinary genetics and FSOT pressure, not billion-parameter models as the mind.

## Checklist before calling a feature “done”

- [ ] Works without GPU  
- [ ] Documented RAM impact  
- [ ] Has `FSOT_LEAN` path if heavy  
- [ ] Default QEMU ≤ 512 MB if used  
- [ ] No mandatory cloud service  
