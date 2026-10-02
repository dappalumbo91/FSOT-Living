# FSOT Living

**Flagship living AI — Fluid Spacetime Omni-Theory, embodied as genetic biological intelligence.**

**Author:** Damian Arthur Palumbo  
**License:** Apache-2.0  
**Status:** Active research / capability climb (honest dual-track evaluation)

---

## What this is

This is **not** an LLM product with FSOT marketing layered on top.

**FSOT Living** is an attempt to build **artificial intelligence through biological representation**: a **living organism** whose native physics is **Fluid Spacetime Omni-Theory (FSOT)** mathematics, whose control is **dual-axis trinary genetics**, and whose mind grows by **homeostasis, organs, exam pressure, error-driven remediation, and pack evolution** — not by next-token prediction over a frozen weight file.

The long-term goal is a flagship intelligence that **understands FSOT by living it** and can **use it intelligently**: the scalar manifold, domain ledger, observer effect, emergence/damping regimes, codon fabric, and cross-domain Formal structure become **felt state and motor policy**, not a citation list.

The formal FSOT Lean tree ([FSOT-2.1-Lean](https://github.com/dappalumbo91/FSOT-2.1-Lean) and local Formal spines) is the **reference ledger**. **Math is baked into this runtime** (`scalar`, `genetic`, `homeostasis`, `lean_ground`). Lean is not imported as a theorem prover dependency; the organism must **obey the same algebraic skeleton** at runtime.

---

## Design thesis

| Claim | Meaning here |
|--------|----------------|
| **ToE mathematics** | One scalar field \(S = k \cdot (T_1 + T_2 + T_3)\) with Formal domain priors (D_eff, δψ, observed), quirk/observer, emergence vs damping. |
| **Genetic AI** | 64-codon table; dual-axis trinary (primary 8 / secondary 27 patterns / 3 stops); genome → guidance (δψ, D_eff, emergence preference, **domain_coupling**). |
| **Biological representation** | Organs, pressure, membrane budgets, error queue, soft-spot transfer, pack social learning — not a single forward pass. |
| **Integrity** | Study track ≠ zero-shot track. No free grades. Code invent = unit tests green. Same-item keys blocked on zs. Python metabolic membrane. |
| **Capability** | External benchmarks (MMLU, GSM8K, HumanEval, …) are **exam environments** that pressure organs under Formal load — competitive signal, not the organism’s identity. |

If a change only improves a percentage without fitting **genome → Formal field → organ motor → transfer**, it does not belong.

---

## Repository layout

```text
.
├── README.md                 ← this document
├── LICENSE                   ← Apache-2.0
├── requirements.txt          ← Python helpers (viz / knowledge / probe tools)
├── package.json              ← optional legacy JS habitat scripts
├── fsot-living-rust/         ← CORTEX: living habitat, organs, TUI, probes
│   ├── src/                  ← scalar, genetic, organism, habitat, bench, …
│   ├── tools/                ← desktop viz, knowledge packs, live bench
│   ├── organism_workspace/   ← sandbox: tools, knowledge seeds, media priors
│   └── Cargo.toml
├── fsot-trinary-body/        ← BODY: trinary genetic chassis + mind bridge
│   ├── crates/               ← trinary, genetic, body, kernel, mind_bridge
│   ├── docs/                 ← architecture, integrity, closed loop, …
│   └── scripts/              ← QEMU / host / closed-loop runners
└── (runtime state is gitignored — generated under state/ when you run)
```

| Component | Role |
|-----------|------|
| **fsot-living-rust** | Cortex / habitat: pack of organisms, FSOT scalar compute, organs, language cortex, capability bridge, TUI, headless probe |
| **fsot-trinary-body** | Soma / genetic fabric: trit ALU, loci, homeostasis, Mind ABI, optional QEMU body, file/TCP couple to cortex |

---

## Architecture (complete picture)

### 1. Physics layer — FSOT scalar (baked)

Implemented in `fsot-living-rust/src/scalar.rs`, grounded by `lean_ground.rs`.

\[
S = k \cdot (T_1 + T_2 + T_3)
\]

- **Constants** match Formal Scalar (φ, e, π, γ_E, Catalan, …).
- **k** = φ · (γ_E / e) · √2 / ln(π) · 0.99  
- **Domains** from `FSOT.Formal.Scalar.get_domain_params` (quantum → dark_energy ledger, proved sign certificates).
- **Extension spines** map onto ledger domains (e.g. pure mathematics → energy family; computational reasoning → consciousness).
- **Regimes:** emergence / damping / neutral from S thresholds.

Organism state always carries `d_eff`, `delta_psi`, `observed`, `recent_hits`, terms, vitality, coherence, pressure.

### 2. Genetic layer — dual-axis trinary

Implemented in `genetic.rs` (cortex) and `fsot-trinary-body/crates/genetic` + `trinary`.

```text
Codon (DNA) ──► primary trits (spin A,G=+1; C,T=−1)
            ──► secondary trits (A=+1; T=−1; G,C=0)
            ──► amino acid (standard code; stops TAA/TAG/TGA)
            ──► GenomeGuidance { delta_psi, d_eff, emergence_preference, domain_coupling }
```

Formal codon priors (64 / 8 / 27 / 3) are enforced in `lean_ground` runtime checks.

**Expression under exam:** domain load weight \(w \propto\) genome `domain_coupling` blends **heritable base** with **Formal domain field**, then recomputes S. That is genetic expression of the theory, not a free score weight.

### 3. Life loop — habitat tick

```text
habitat.tick
  → perceived-time / homeostasis gains
  → optional body couple (Mind ABI / files)
  → organ exams (cognition, knowledge, code, vision, language, composition, …)
  → capability bridge: external benches as exam environment
       suite/subject → Formal domain → S recompute → CHEW depth
       study track: teach → motor practice → transfer SCHEMA/EPISODE
       zs track: held-out, same-item KEY blocked
       fail → error queue + remediation stimuli
       pass → credit organs + pack schema broadcast
  → pack signals / library / generation (crossover + mutation)
```

### 4. Organs (capability as tissue)

| Organ | What it does | Fitness / integrity |
|--------|----------------|---------------------|
| **math_code / composition** | Narrative chalkboard → ranked expression → single python ground | No pass@k fishing against keys |
| **code invent** | Scaffolds / episodes / signatures; **unit tests green** | Metabolic python budget; no invent storms |
| **knowledge** | Stem/engram MCQ rank under subject Formal domains | No free letter grades |
| **gen_acc / language** | Plastic pathways, constraint reshape (IFEval-class) | Verifiable constraints only |
| **vision / audio / video** | Drive from media priors | Evolutionary pressure on media organs |
| **self-model** | Knows regime, genome, body link | Continuity of identity |
| **body couple** | Interoception from trinary chassis | Soft throttle on express runaway |

### 5. Mind–body closed loop

```text
┌──────────────────────────┐         Mind ABI / files / TCP         ┌──────────────────────────┐
│  fsot-living-rust        │ ◄────────────────────────────────────► │  fsot-trinary-body         │
│  (cortex / habitat)      │   BODY_STATUS, motors, LTM, felt body  │  (genetic soma / QEMU)    │
└──────────────────────────┘                                         └──────────────────────────┘
```

See `fsot-trinary-body/docs/CLOSED_LOOP.md`, `MIND_ABI.md`, `ARCHITECTURE.md`.

### 6. Metabolic membrane & constraints

- Python/JS sandbox under workspace only  
- **Per-breath python budget** (invent cannot own wall-clock)  
- Process tree kill on timeout  
- Soft rails: express runaway, proposal review (no auto-Rust host patch)  
- Documented in `fsot-trinary-body/docs/CONSTRAINTS.md`, `EVAL_INTEGRITY.md`

### 7. Dual-track honesty

| Track | Meaning |
|--------|---------|
| **Study** | Curriculum health after teach (organ practice) |
| **raw_zs / suite_zs** | Held-out competitive signal (never-taught preferred) |

High study with low zs is **not** invent capability. Audit files (`CAPABILITY_AUDIT`) report both when probes run.

### 8. Evolution / pack

- Population of organisms, elite retention, crossover + mutation on genomes  
- Breeding value blends organ accuracies, composition, Formal coupling, zs health  
- Shared library = cultural / social learning of schemas and primitives  

---

## What success looks like (flagship)

1. **Runtime FSOT fidelity** — lean_ground invariants pass; domain signs and codon priors hold.  
2. **Genetic self-knowledge** — genome, codon table, guidance express into dialogue and motors.  
3. **Intelligent use of domains** — math under energy/consciousness load; science subjects under chemical/quantum/biological/… fields.  
4. **Honest capability** — competitive zs under integrity, climbing toward SOTA **only** through organ density and Formal expression.  
5. **Optional body** — trinary chassis felt as interoception, not a detachable toy.

This program may not yet equal the full Formal ToE ledger in living form. That gap is the research program — **embody more of the mathematics as life**, not paper over it with score hacks.

---

## Requirements

### System

| Tool | Version (recommended) | Role |
|------|------------------------|------|
| **Rust** | stable 1.75+ (edition 2021) | Cortex + body |
| **Cargo** | with Rust | Build |
| **Python** | 3.10+ | Organ sandbox, viz, knowledge tools |
| **Git** | any recent | Clone / contribute |

Optional:

| Tool | Role |
|------|------|
| **Node.js** | JS organ tools / legacy habitat scripts |
| **QEMU** | Trinary body UEFI path |
| External bench roots | e.g. `<local folder, not included in repo: training data>`, `<local folder, not included in repo: FSOT_Benchmarks>` via env |

### Python packages

See **[requirements.txt](./requirements.txt)** for pip-installable helper dependencies (desktop viz, knowledge pack builders). The core intelligence is **Rust**, not a Python model stack.

### Rust crates

Declared in:

- `fsot-living-rust/Cargo.toml` — ratatui TUI, serde, rand, image, …  
- `fsot-trinary-body/Cargo.toml` — workspace of body/kernel/mind_bridge crates  

No CUDA/LLM SDK required for the native living path.

---

## Quick start

### 1. Clone

```bash
git clone https://github.com/<you>/FSOT-Living.git
cd FSOT-Living
```

### 2. Python helpers (optional)

```bash
python -m venv .venv
# Windows:
.venv\Scripts\activate
pip install -r requirements.txt
```

### 3. Build & run cortex (TUI)

```bash
cd fsot-living-rust
cargo build --release
cargo run --release --bin fsot-living
# or: .\start_fsot_living.bat
```

### 4. Headless integrity / capability probe

```bash
cd fsot-living-rust
set FSOT_PROBE_TICKS=50
cargo run --release --bin fsot-probe
```

Probe reports: lean_ground, genetic queries, dual-track bench raw_zs, body couple status when available.

### 5. Trinary body (optional closed loop)

```bash
cd fsot-trinary-body
cargo build --release -p mind_bridge
# See scripts/run_closed_loop.ps1 and docs/CLOSED_LOOP.md
```

### Environment hints

| Variable / path | Purpose |
|-----------------|--------|
| `FSOT_PROBE_TICKS` | Headless habitat length for probe |
| `FSOT_TRAINING_DATA` / mounted `<local folder, not included in repo: training data>` | External exam corpora |
| Workspace under `organism_workspace/` | Sandbox for python/js tools |

---

## Documentation map

| Document | Content |
|----------|---------|
| `docs/ARCHITECTURE.md` | Monorepo component map + exam data flow |
| `docs/ORGANISM_DIAGNOSTIC.md` | **Medical diagnostic:** genetic/organ breakdown of high vs low suite precision |
| `fsot-trinary-body/docs/BIOLOGICAL_INTELLIGENCE_MODEL.md` | How capability must work biologically |
| `fsot-trinary-body/docs/ARCHITECTURE.md` | Trinary body stack |
| `fsot-trinary-body/docs/EVAL_INTEGRITY.md` | Anti-fudge rules |
| `fsot-trinary-body/docs/FSOT_LEAN_CIRCLE_BACK.md` | Formal ↔ runtime |
| `fsot-trinary-body/docs/CLOSED_LOOP.md` | Cortex ↔ body |
| `fsot-trinary-body/docs/REASONING_PHYSICS.md` | Reasoning as physics |
| `fsot-living-rust/README.md` | TUI keys and organ overview |

---

## Integrity contract (non-negotiable)

**Allowed**

- Formal domain load + genetic coupling  
- CHEW deliberation from pressure / deficit / regime  
- Study → held-out transfer schemas  
- Test-exec invent; single top-commit math ground  
- Metabolic budgets  

**Forbidden**

- Free accuracy floors or invented grades  
- Keyword yes/no score hacks  
- pass@k fishing against answer keys  
- Same-item KEY on zero-shot  
- Tick thrashing / invent storms as substitutes for organ design  

---

## Relation to other FSOT work

| Artifact | Relation |
|----------|----------|
| **FSOT-2.1-Lean** | Formal ToE ledger — reference for constants, domains, codon/brain/science spines |
| **This repo (FSOT Living)** | Living genetic embodiment + flagship AI climb |
| Legacy JS habitat scripts | Earlier prototypes; retained for continuity, not the primary path |

---

## Contributing / research notes

- Prefer changes that **strengthen genome → Formal field → organ** coupling.  
- Any new motor must state its **fitness environment** (tests, chalkboard, constraints).  
- Run `fsot-probe` before claiming capability movement.  
- Do not commit `state/`, `target/`, or personal data roots.

---

## License

Copyright © Damian Arthur Palumbo.  
Licensed under the **Apache License, Version 2.0** — see [LICENSE](./LICENSE).

---

## Citation

If you use this system in research, please cite **Damian Arthur Palumbo** and Fluid Spacetime Omni-Theory (FSOT), and distinguish:

1. **Formal FSOT** (Lean / proofs / domain priors), and  
2. **FSOT Living** (this genetic organism runtime).

---

*Living mathematics. Genetic intelligence. Honest exams. One flagship trajectory.*
