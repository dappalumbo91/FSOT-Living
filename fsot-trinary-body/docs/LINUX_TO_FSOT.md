# Linux as informational foundation → FSOT biological body

We study Linux (and POSIX machines generally) as a **map of necessary layers**,
not as code to copy. Each layer is rewritten for **trinary genetics** and
**homeostatic biology**.

## Why Linux is useful as a map

A working multi-user OS already solved:

1. **Boot trust** — how does inert hardware become a living control loop?
2. **Isolation** — how do organs share chassis without corrupting each other?
3. **Scheduling** — who runs when energy/time is scarce?
4. **Namespace / files** — durable shared memory of the body.
5. **Drivers** — how physics (serial, disk, net) becomes sensation/action.
6. **Syscall boundary** — how mind talks to body without owning the metal.

Those *questions* stay. The *answers* change.

## Layer table

| Linux concept | Binary-static habit | FSOT trinary body |
|---------------|---------------------|-------------------|
| bit / flag | 0 or 1, frozen | **Trit** −1 / 0 / +1; continuous field + collapse |
| process | address space + PC | **Organ process** with genome-guided expression |
| scheduler (CFS) | fair CPU time | **Homeostasis pacemaker** — pressure × fitness × breath |
| syscalls | trap to kernel | **Mind ABI** (MOTOR / MIND telemetry) |
| `/proc`, `/sys` | kernel stats as files | **Interoception channels** as native samples |
| VFS | file + inode tree | **Organ membrane** (knowledge, tools, media later) |
| cgroups | resource limits | **Allostatic load** — thermal, heap, attention budget |
| signals | async events | **Pack / pressure events** (aligned with living habitat) |
| init / systemd | service graph | **Zygote → organ ignition order** |
| userspace | apps | Living-fsot mind, interpreters, breeders |
| mm / page tables | virtual memory | Frame pool + later **genome-backed pages** |
| drivers | hardware adapters | Sensory organs (serial first, then storage, net) |

## Boot path (Linux-informed, biology-named)

```
power-on / QEMU start
    → firmware (UEFI)                 # zygote membrane
    → bootloader loads kernel ELF     # implantation
    → kernel_main                     # first breath
    → enable body math (SIMD later)   # metabolism prep
    → heap + frames                   # tissue substrate
    → serial console                  # first sensory nerve
    → genetic fabric init             # germline expression
    → interoception ring              # vagus loop
    → homeostasis tick                # heartbeat
    → Mind ABI accept                 # cortex attach point
```

## Scheduler: not CFS, homeostasis

Linux optimizes fairness and latency under load averages.
We optimize **organism viability**:

```
tick priority ∝ expression(genome) × pressure_need × (1 − allostatic_load)
```

An organ that is starving for data but has high plasticity may preempt a
sated organ. Superposed loci can flip phenotype mid-tick when collapse
threshold is crossed — **that is genetic dynamics, not a config bit flip**.

## Files: not inodes-as-truth

Linux files are byte streams with metadata. Our primary truth is:

- **Fact / state cards** (aligned with living knowledge organ)
- **Genome snapshots** (trits + continuous fields)
- **Wire packing of Word27** only when leaving the body (disk, Mind ABI)

Bytes on disk are a *serialization*, not the native type system.

## Drivers: sensation before “I/O completeness”

Order of sensory organs:

1. Serial / console (debug nerve)
2. Timer / TSC (time sense)
3. Memory pressure (hunger)
4. Storage (long-term body memory)
5. Net / host bridge (environment)

## Syscall boundary → Mind ABI

Linux: `read`, `write`, `ioctl`, …
Body: line protocol first (proven in prior `fsot_os`):

- Kernel → world: `MIND seq=… focus=… breath=…`
- World → kernel: `MOTOR STATUS | ATTEND | BREATHE | REPORT | …`

Later: binary Mind ABI frames packing trits, not opaque binary blobs without
trinary semantics.

## What we deliberately do *not* import

- Monolithic “everything is a file” ideology as the *only* metaphor
- Binary flags for gene on/off
- Static config as destiny (no re-collapse)
- Remote LLM as cortex (prior experiment failed; brain stays attachable body)
- Flat single-process “OS” that is really a userspace app

## Bridge to living habitat

```
fsot-living-rust (Windows mind gym)
        ↕ Mind ABI / shared VFS organ
fsot-trinary-body (QEMU / bare metal)
```

Same dual-axis genome language, same S scalar family, different substrate.
