# Linux salvage map — next expansions for the FSOT body

Linux is still the **informational blueprint**. We do not port libc or userspace wholesale;
we **salvage the jobs** each subsystem does and rebuild them as biology + trinary + FSOT.

## Already salvaged

| Linux idea | FSOT body today |
|------------|-----------------|
| Layered boot | zygote → heap → soma → Mind ABI |
| Syscalls | MOTOR / MIND / ACK lines |
| `/proc` | interoception + BODY_STATUS |
| cgroups soft limits | goldfish `ScaleProfile` |
| logs | BODY_TELEMETRY, LIVE_STATUS, LTM |
| IPC | TCP nerves + file desk notes |

## High-value next salvages (ordered)

### 1. VFS home directory — **IMPLEMENTED**
**Linux:** `$HOME`, XDG dirs.  
**Biology:** long-term memory compartments + autobiographical self.  
**On disk:** `state/habitat-rust/home/{knowledge,tools,media,genome,proposals,identity}/`  
**Mind ABI:** `MOTOR HOME` · `MOTOR READ/WRITE /home/...`  
**See:** [BIO_LINUX_MAP.md](BIO_LINUX_MAP.md)

### 2. Scheduler as homeostasis — **v0 IMPLEMENTED**
**Linux:** CFS, nice, cgroups.  
**Biology:** attention + metabolic priority (not fairness).  
**Mind ABI:** `MOTOR SCHED`  
```
priority ∝ expression × need × (1 − overload) × breath_step
```

Audio/video/code organs already exist in living — body-side should **meter** them by heap/thermal.

### 3. Drivers as sensory organs
**Linux:** udev, DRM, ALSA, netdev.  
**Salvage order:** serial ✓ → timer/TSC ✓ → storage LTM ✓ → **framebuffer console** (visual in QEMU) → host folder mount → optional net bridge.

Desktop viz (browser) is the **host-side DRM**; QEMU framebuffer is the **bare-metal face**.

### 4. init / service graph
**Linux:** systemd units, dependencies.  
**Salvage:** zygote ignition order already documented; formalize as:

```
serial → heap → genome → intero → membrane → mind_abi → organs/*
```

Failures degrade gracefully (body offline ≠ mind crash).

### 5. namespaces / principals
**Linux:** user/ns, pid/ns.  
**Salvage:** Human vs Mind principals (from prior `fsot_os`) — capability tokens so cortex cannot raw-write kernel heap.

### 6. journald / audit
**Linux:** structured logs, auditd.  
**Salvage:** append-only `BODY_TELEMETRY` + LTM rotations (`BODY_LTM.prev.txt`) — extend with signed lineage digests.

### 7. package manager → breeding / organ install
**Linux:** apt/pacman installs software.  
**Salvage:** **breeding + proposal organs** install *skills* under review, never silent kernel overwrite.

## What not to salvage yet

- Full POSIX userspace / glibc  
- Wayland/X11 as the mind (browser viz is enough for host)  
- Network stack before storage home is solid  
- GPU kernels as *required* path (5070 is **scale expansion**, not dependency)

## Link to FSOT 2.1 Lean verification

Lean repo formalizes **S = k·(T1+T2+T3)**, codon/protein trinary, Trinary OS priors.  
Body OS should keep consuming that as **oracle constants + domain priors**, not re-invent math.

Priority bridge: `FSOT.Formal.TrinaryOS*` + `CodonPriors` / `ProteinPriors` → genetic fabric LTM schema.
