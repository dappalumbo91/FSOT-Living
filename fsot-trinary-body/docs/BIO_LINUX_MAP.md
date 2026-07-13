# Linux salvage → biological / genetic wiring (FSOT)

We salvage **jobs** from Linux, not code. Each piece maps to a biological
function and a **trinary genetic** place in the body so it wires into FSOT
pressure, not a dead config flag.

## Core template

```
Linux job  →  Biological equivalent  →  Genetic / FSOT wiring  →  Our apparatus
```

| Layer | Linux (template) | Biology | Genetics / FSOT | Our build |
|-------|------------------|---------|-----------------|-----------|
| **Boot** | firmware → kernel | fertilization → first breath | zygote genome expression ON | `kernel_main` / soma boot |
| **Process** | PID + address space | organ / tissue process | organ locus field + expression | organ membrane nodes |
| **VFS / $HOME** | files + paths | long-term memory + identity stores | chromatin / LTM encode | `/home/*` + disk LTM |
| **Scheduler** | CFS / nice | attention + metabolic priority | pressure × expression × plasticity | organ priority queue |
| **Syscalls** | trap to kernel | motor / vagus efferents | MOTOR verbs collapse intent | Mind ABI |
| **/proc /sys** | kernel stats as files | interoception (vagus afferents) | sensor → trit attention | intero ring |
| **cgroups** | resource caps | allostasis / tank size | ScaleProfile goldfish | lean→max breath |
| **IPC** | pipes/sockets | nerves / hormones | TCP MIND/MOTOR | mind_bridge |
| **Drivers** | hardware adapters | sensory / motor end-organs | organ-specific loci | serial, heap, thermal… |
| **init/systemd** | service graph | developmental ignition order | expression cascade | zygote → organs |
| **journald** | logs | episodic memory trace | telemetry append | BODY_TELEMETRY |
| **packages** | apt install | learning / breeding skills | proposals under selection | PROPOSALS + forge |
| **users/ns** | principals | self vs other | Human / Mind tokens | later principals |
| **framebuffer** | pixels to display | visual cortex projection | host viz now; QEMU FB later | desktop_viz |

## VFS / home (this milestone)

| Linux | Biology | Genetic wiring |
|-------|---------|----------------|
| `$HOME` | autobiographical self + belongings | identity compartment; high plasticity early, consolidates on SLEEP |
| `~/Documents` | semantic / episodic memory | `/home/knowledge` — write = encode, read = recall under pressure |
| `~/bin` tools | motor skills / procedural memory | `/home/tools` — forged skills (py/js) |
| media library | sensory memory traces | `/home/media` — **pointers** not bulk copies (like synaptic tags) |
| genome / .config | germline + regulatory DNA | `/home/genome` + `BODY_LTM` |
| Desktop / proposals | prefrontal planning buffer | `/home/proposals` — novelty under review |

**FSOT rule:** disk write during **SLEEP/SAVE** = consolidation (like systems consolidation in sleep).  
RAM organ buffers = **working memory**. Trit state of compartment = **expressed / latent / repressed** access.

## Scheduler (next)

| Linux CFS | Biology | FSOT |
|-----------|---------|------|
| vruntime fairness | not fairness — **viability** | `priority = expr × need × (1−load) × breath_step` |
| nice | neuromodulation | MOTOR ATTEND / EXPRESS / REPRESS |
| blocked I/O | waiting on sensation | organ blocked until sensor/TCP data |

## Framebuffer (plain English)

A **framebuffer** is **not** the whole desktop OS. It is:

> a region of memory that holds the **pixels** currently shown on the screen.

- GPU/CPU writes colors into that memory  
- Display hardware scans it out to the monitor  

**Desktop** (windows, icons, mouse) is software *drawing into* a framebuffer (or into a compositor that feeds one).  

| Today | Later |
|-------|--------|
| **Host face:** browser `desktop_viz` (HTML/canvas) — like a remote monitor | **Bare-metal face:** QEMU/guest framebuffer — pixels from inside the body OS |
| Template: Rust TUI + old JS dashboard | Same *information* (S, phenotype, organs), different *paint surface* |

**When bare-metal FB?** After home VFS + storage organ are solid, and kernel can own a simple pixel console without breaking serial Mind ABI. Order-of-magnitude: **not next week** — likely **after** home persistence, organ scheduler, and one real storage path are green (call it **phase “face”**, post **phase “gut/memory”**). Host viz is the right face **now**.

## Genetic states (never binary)

Every salvaged subsystem must expose:

- continuous **field** (FSOT channel)  
- **Trit** phenotype: repress / latent / express  
- **plasticity** (can re-open on BREATHE)  
- optional **disk LTM** commit on SLEEP  

Binary “feature flags” are rejected as primary state.
