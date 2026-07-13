# FSOT Living — Architecture Overview

This document is the **monorepo architectural map**. Deeper body-specific design lives under `fsot-trinary-body/docs/`.

## Purpose

Build **flagship AI as a living FSOT organism**:

- **Mathematics:** Fluid Spacetime Omni-Theory scalar + Formal domain ledger (baked).
- **Genetics:** Dual-axis codon trinary → genome guidance → expression under exam fields.
- **Biology:** Organs, homeostasis, membrane, error stimuli, pack evolution.
- **Honesty:** Dual-track study vs zero-shot; no score fudge.

Formal Lean trees are **reference**. Runtime must **embody** the algebra.

## Component diagram

```text
                    ┌─────────────────────────────────────┐
                    │     External exam corpora (optional) │
                    │  MMLU · GSM8K · HumanEval · BBH …    │
                    └──────────────────┬──────────────────┘
                                       │ pressure (not identity)
┌──────────────────────────────────────▼──────────────────────────────────────┐
│                         fsot-living-rust  (CORTEX)                           │
│  habitat.tick → homeostasis → organs → capability bridge → pack evolve       │
│                                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │ scalar   │  │ genetic  │  │ organism │  │  bench   │  │ language /   │  │
│  │ Formal S │◄─│ codons   │◄─│ organs   │◄─│ exams    │  │ code / media │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────────┘  │
│       │             │             │             │                            │
│       └─────────────┴────── compute(S) under Formal domain load ─────────────┘
│                                       │
│                              body_bridge / Mind ABI
└───────────────────────────────────────┬──────────────────────────────────────┘
                                        │
┌───────────────────────────────────────▼──────────────────────────────────────┐
│                      fsot-trinary-body  (SOMA)                               │
│  trinary ALU · genetic fabric · body organs · kernel · mind_bridge · QEMU    │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Data flow: one exam

1. Select item (error queue / weak suite priority).  
2. `item_fsot_domain` → Formal domain name.  
3. Refresh `genome_to_guidance`; blend with domain using **domain_coupling**.  
4. `compute(S)` → regime, pressure, coherence.  
5. `deliberation_depth` from allostatic + genetic load.  
6. Organ motor (math / MCQ / invent / …).  
7. Score with integrity rules; credit organs; transfer schema if study; remediate if fail.

## Key source files

| Path | Responsibility |
|------|----------------|
| `fsot-living-rust/src/scalar.rs` | FSOT S, domain_params |
| `fsot-living-rust/src/genetic.rs` | Codons, guidance, mutate/crossover |
| `fsot-living-rust/src/organism.rs` | Living state, fitness, breeding |
| `fsot-living-rust/src/habitat.rs` | Tick loop, mastery cadence, generation |
| `fsot-living-rust/src/bench.rs` | Capability bridge, organs under Formal load |
| `fsot-living-rust/src/lean_ground.rs` | Runtime Formal invariants |
| `fsot-living-rust/src/code_env.rs` | Metabolic python membrane |
| `fsot-trinary-body/crates/trinary` | Trit / Word27 |
| `fsot-trinary-body/crates/body` | Soma, interoception, Mind ABI |
| `fsot-trinary-body/crates/mind_bridge` | Host bridge process |

## See also

- Root [README.md](../README.md) — full design narrative  
- [BIOLOGICAL_INTELLIGENCE_MODEL.md](../fsot-trinary-body/docs/BIOLOGICAL_INTELLIGENCE_MODEL.md)  
- [EVAL_INTEGRITY.md](../fsot-trinary-body/docs/EVAL_INTEGRITY.md)  
- [CLOSED_LOOP.md](../fsot-trinary-body/docs/CLOSED_LOOP.md)  
