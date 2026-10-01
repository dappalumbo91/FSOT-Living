# Architecture — FSOT Trinary Body

## Intent

Build a **companion OS body** that:

1. Boots under **QEMU** (UEFI path first).
2. Runs a **homeostatic kernel** whose native state is **trinary genetic**.
3. Exposes **interoception** and a **Mind ABI** so `fsot-living-rust` can
   attach as cortex without owning the metal.
4. Grows layer-by-layer (Linux map, FSOT semantics) — never as a static binary toy.

## Design invariants

### 1. Trinary genetic states (not binary)

```
Trit ∈ { SpinDown = −1, Superposed = 0, SpinUp = +1 }
```

Biological reading:

| Trit | Genetic reading | Body reading |
|------|-----------------|--------------|
| −1 | repress / damp | inhibit pathway |
| 0 | latent / undecided | open possibility |
| +1 | express / emerge | drive pathway |

A **locus** is never “a bit set once”:

```text
field: f64          continuous pre-phenotypic field
state: Trit         collapsed (or still Superposed)
expression: f64     strength of current expression
plasticity: f64     how easily field re-opens and re-collapses
```

Under **pressure** (interoceptive load, Mind motor commands, organ demand),
`collapse(field)` may change `state`. Under rest + high plasticity, field can
drift and re-superpose. **That is the anti-binary design.**

### 2. Dual-axis genome (shared with living habitat)

Codons → primary / secondary trit axes (same mapping as `fsot-living-rust`):

- Primary: purine +1 / pyrimidine −1  
- Secondary: A +1 / T −1 / G,C 0  

Genome guidance yields `delta_psi`, `d_eff`, emergence preference — inputs to
the FSOT scalar path, not dead config.

### 3. Word27

Native machine word: **27 trits** (3³), packable to 6 bytes for wire/disk.
Wire packing is transport; **semantic ops stay in trit space**.

### 4. Layer stack

```
┌─────────────────────────────────────────────┐
│  Cortex (later): living-fsot / Python / JS  │
├─────────────────────────────────────────────┤
│  Organ membrane: knowledge, tools, media    │
├─────────────────────────────────────────────┤
│  Mind ABI + interoception ring              │
├─────────────────────────────────────────────┤
│  Homeostasis pacemaker (scheduler)          │
├─────────────────────────────────────────────┤
│  Genetic fabric (dynamic loci)              │
├─────────────────────────────────────────────┤
│  Trinary ALU / Word27                       │
├─────────────────────────────────────────────┤
│  Soma: serial, heap, frames, panic          │
├─────────────────────────────────────────────┤
│  Zygote: UEFI bootloader                    │
├─────────────────────────────────────────────┤
│  Chassis: QEMU q35 / real x86_64            │
└─────────────────────────────────────────────┘
```

## Crates

| Crate | std? | Role |
|-------|------|------|
| `trinary` | no_std | Trit, Word27, gates, collapse |
| `genetic` | no_std | Loci, codons, genome, expression tick |
| `kernel` | no_std | Bare-metal entry, serial, body loop |
| `host_probe` | std | Host simulation / unit integration |
| `fsot_boot` | std | Build UEFI/BIOS images, optional QEMU |

## Kernel loop (v0)

```
boot
  print identity + genome fingerprint
  init genetic fabric from seed codons
loop:
  sample body proxies (tick, heap stub, breath phase)
  genetic fabric: apply pressure → re-collapse loci
  emit BODY line on serial (telemetry)
  if MOTOR command: adjust focus / force expression
```

## Evolution path (milestones)

| Milestone | Deliverable | Status |
|-----------|-------------|--------|
| M0 | Host trinary + genetic tests green | done |
| M1 | Kernel serial “body online” + UEFI image | done |
| M2 | Interoception ring + homeostasis breath | done (`body::interoception`) |
| M3 | Mind ABI MOTOR/MIND | done (`body::mind_abi`) |
| M4 | Organ membrane stub (VFS-like) | done (`body::organs`) |
| M5 | Bridge living-fsot ↔ body (files) | done (`mind_bridge`) |
| M6 | Habitat loop consumes BODY_STATUS / writes MOTOR | done (`body_bridge.rs`) |
| M7 | Kernel real heap + thermal MSR | done (`sensors.rs`) |
| M8 | QEMU interactive serial + closed-loop scripts | done |
| M9 | Real-time TCP Mind ABI (+ file fallback) | done |
| M10 | Min-spec 2012 Mac Mini (FSOT_LEAN, budgets) | done |
| M11 | Memory tiers LTM/STM/nerves + goldfish hardware scale | done |
| M12 | Companion home VFS + bio scheduler (Linux salvage) | done |
| M13 | Host desktop viz (browser face) | done |
| M14 | Ignition graph + scheduler-driven organ ticks | done |
| M15 | Genome writeback + tools/proposals → home LTM | done |
| M16 | QEMU framebuffer bare-metal face | later (after storage solid) |
| M17 | Principals (Human/Mind caps) + body-constrained breeding | done |
| M18 | Desktop viz query I/O + auto-boot with living | done |
| M19 | FSOT dream weave during SLEEP (not waking defrag) | done |
| M20 | Dream journal → viz + richer domain tags from Lean | next |
| M21 | QEMU framebuffer bare-metal face | later |

## Relationship to `<local folder, not included in repo: Fsot trinary/fsot_os>`

Prior art already proved QEMU UEFI, heap, frames, interoception, mind protocol,
TVM/FSOTB. This tree is a **clean ground-up body** next to the living mind:

- **Imports ideas**, not a bulk copy of every brain/LLM path.
- **Genetic dynamics first** (that was thinner in the prior kernel).
- Can **re-home** proven modules (serial, heap, mind_protocol) as organs mature.

## Non-goals (for now)

- Full POSIX / Linux ABI compatibility
- Networking stack completeness
- Remote LLM cortex
- GPU kernels inside QEMU first mile
