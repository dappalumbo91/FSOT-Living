# Organism Medical Diagnostic — Genetic / Organ Breakdown

**Date:** 2026-07-13  
**Subject:** FSOT Living flagship pack (post-probe habitat, gen ~90)  
**Evidence:** `state/habitat-rust/CAPABILITY_AUDIT.json`, `ACCURACY_AUDIT.json`, dual-track probe, source routes in `bench.rs` / `genetic.rs` / `organism.rs`  
**Integrity baseline:** lean_ground 7/7 · codon 64/8/27/3 · raw_zs hard-count preferred over EMA  

This is a **clinical read** of how the living system functions genetically — why some exam surfaces score high and others low — not a leaderboard victory lap.

---

## 1. Executive impression (attending physician)

| Finding | Clinical meaning |
|---------|------------------|
| **Study mean ~78–89%** on many suites | Organs **can** execute when taught (tissue present). |
| **raw_zs 73.7%** overall (honest n=118) | Held-out capability is real on some motors. |
| **Huge study→zs gaps** on invent / multi-step / 10-way science | **Transfer / generation** lesion, not total organ death. |
| **composition_accuracy ~0.33** (weak across pack) | Math multi-step organ is the chronic weak tissue. |
| **code study high, invent zs low** | Code organ practices reference re-exec; **invent effector library** incomplete. |
| **MCQ closed-set suites peak 100%** | Ranking motors match the exam physics (options as environment). |

**Primary diagnosis:**  
*Selective effector-library insufficiency under open-ended fitness environments*, with **intact closed-set ranking tissue** and **partial composition motors**. Genetics set field coupling; **motor density** determines suite scores more than genome diversity right now.

---

## 2. Vital signs (dual-track)

### 2.1 Honest held-out (raw_zs) — competitive tissue

| Suite | raw_zs | n | Motor class | Formal domain (typical) |
|-------|--------|---|-------------|-------------------------|
| hellaswag | **100%** | 10 | Closed ranking (endings) | neural |
| winogrande | **100%** | 10 | Closed ranking (pronoun) | neural |
| arc_challenge | **100%** | 10 | Closed MCQ A–D | molecular / subject |
| arc_easy | **100%** | 10 | Closed MCQ A–D | molecular / subject |
| bbh | **80%** | 10 | Structured motors + rank | consciousness |
| mmlu | **80%** | 10 | Closed MCQ A–D | subject-aware |
| truthfulqa | **80%** | 10 | Truth keys / gen | social |
| gsm8k | **60%** | 10 | Multi-step compose + py | consciousness |
| math | **60%** | 10 | Compose / energy field | energy |
| ifeval | **50%** | 10 | Constraint generation | ai |
| mmlu_pro | **40%** | 10 | 10-way MCQ rank | subject-aware |
| humaneval | **25%** | 8 | **Invent under unit tests** | energy |

**Overall raw_zs: 73.7% (87/118).**

### 2.2 Study vs zs_ema (transfer lesion map)

Study can be high while zs_ema stays low — classic **practice ≠ invent**:

| Suite | Study (EMA) | zs_ema | Gap | Lesion type |
|-------|-------------|--------|-----|-------------|
| mmlu_pro | ~97% | ~24% | **~73 pp** | Teach answer keys → rank fails on held-out 10-way |
| math | ~97% | ~23% | **~74 pp** | Reference/path study ≠ novel composition |
| gsm8k | ~82% | ~25% | **~57 pp** | Narrative motors partial coverage |
| humaneval | ~89% | ~55% (n tiny) | large | **Study re-runs canonical**; zs must invent |
| bbh | ~23% | ~26% | small | Study under-sampled; motors carry zs when present |
| hellaswag | ~78% | ~78% | ~0 | Ranking motor stable study↔zs |

**Rule:** Prefer `suites_zeroshot_raw` over pack EMA. EMA under-samples and sticks.

---

## 3. Genetic system review

### 3.1 How the genome is supposed to act

```text
Codon sequence (DNA)
  → primary trinary  (A,G=+1; C,T=−1)     → spin / δψ bias
  → secondary trinary (A=+1; T=−1; G,C=0) → genetic pattern space (27)
  → amino-acid chain + start/stop counts
  → GenomeGuidance {
        delta_psi,
        d_eff,
        emergence_preference,
        domain_coupling   ← gates Formal field blend weight
     }
  → exam: item Formal domain × domain_coupling → recompute S
  → CHEW depth from pressure + coupling + composition deficit
  → organ motor → fitness
```

Formal codon certificates (runtime): **64 codons, 8 primary patterns, 27 secondary, 3 stops, ATG→M** — lean_ground PASS.

### 3.2 What genetics currently does well

- Sets **homeostatic field parameters** (δψ, D_eff, emergence) every exam.
- **domain_coupling** scales how hard Formal domain load binds (not free grades).
- Breeding value includes composition + coupling × competence (selection pressure exists).
- Pack genomes diversify via crossover/mutation; elites retain structure.

### 3.3 Genetic limitations (lab findings)

| Issue | Detail |
|-------|--------|
| **Low dimensional expression** | Guidance collapses to ~4 scalars. No per-organ loci (no “math operon” vs “invent operon” as separate genetic modules). |
| **Homogeneous pack field** | Many orgs share similar D_eff / coupling bands → suite gaps are **motor**, not “wrong genome for physics.” |
| **composition chronically weak** | ACCURACY_AUDIT: composition ~33% thr 55% — flagged weak; stim boosts fire, but open-ended multi-step still incomplete. |
| **self_model / collective at 0** | Parallel lesions: social/collective and self-model not fed by bench path; not the cause of GSM/HE gaps but pack diagnostics noise. |
| **code_accuracy ≠ invent** | Study credits code via **reference exec** (`code_study_ref_exec`). Genetic selection sees healthy code organ while invent zs is sick. |

**Genetic verdict:** The genome is a **field regulator**, not yet a **library of effector genes**. High/low suite scores track **which effector programs exist in LTM/scaffolds**, not which codons are ATG.

---

## 4. Organ × suite routing (anatomy)

| Suite class | Primary organ (declared) | Actual motor pathway | Fitness environment |
|-------------|--------------------------|----------------------|---------------------|
| HellaSwag / Wino / ARC / MMLU | knowledge / gen_acc | **Rank fixed options** (stem + engram) | Closed choice set |
| MMLU-Pro | knowledge | Rank **10** options | Closed but high entropy |
| GSM / MATH | math_code → **composition** | Rank expressions → **single** python ground | Open numeric answer |
| HumanEval / MBPP | code | Scaffolds / episodes / tests | **Unit tests** (open program) |
| BBH | knowledge + special motors | bool/arith/nav/track/option-rank | Mixed structured |
| IFEval | gen_acc | Constraint reshape | Surface constraints |

### 4.1 Common denominator of **HIGH** precision

Suites that hit **≥80–100% raw_zs** share:

1. **Closed hypothesis set** — options A–D (or few endings).  
2. **Ranking is the organ** — no need to invent a new program.  
3. **Stem/engram affinity** is enough under Formal load.  
4. **Transfer is geometric** (same ranker, different stem) not generative.  
5. **Low metabolic cost** — little/no python invent storm.

**Common denominator: selection among existing candidates, not synthesis of novel structure.**

### 4.2 Common denominator of **LOW / fragile** precision

Suites that lag share:

1. **Open-ended generation** — invent a function body or multi-step expression never seen as a fixed option.  
2. **Fitness is strict and sparse** — unit tests green, or exact number, or multi-constraint text.  
3. **Study path cheats the metric** — teach answer / re-run canonical → study EMA healthy, zs hollow.  
4. **Effector library incomplete** — scaffolds cover classics; long tail of HE / GSM narratives uncovered.  
5. **Option entropy (MMLU-Pro)** — 10-way rank dilutes margin; same ranker that kills A–D loses precision.

**Common denominator: synthesis under hard environmental fitness with incomplete motor library + study/zs integrity gap.**

---

## 5. Formal field load — is the domain map the problem?

| Load | Suites | Observed Formal signs | Clinical note |
|------|--------|----------------------|---------------|
| neural | HS, Wino | observed, mid D | Works with ranking |
| molecular / subject | MMLU, ARC | observed | Works when options ≤4 |
| consciousness | GSM, BBH | observed positive | Helps multi-step when motors exist |
| energy | MATH, HE invent | observed positive | Field OK; invent library limits HE |
| ai | IFEval | non-positive family | Hard gen constraints |

**Field load is not the primary lesion.** High and low suites both get legitimate Formal coupling. Failures are **downstream of S** — in motor selection / library density.

Exception: MMLU-Pro subject routing helps when subject keywords fire; residual error is **10-way rank margin**, not wrong D_eff.

---

## 6. System lesions (problem list)

### P1 — Composition organ hypoplasia (priority)

- **Sign:** composition_accuracy weak (~0.33); GSM/MATH zs mid; study high.  
- **Mechanism:** Multi-step expression library partial; narrative templates cover some families only.  
- **Not:** Genome cannot set consciousness/energy fields.  
- **Rx:** Expand composition templates + transfer SCHEMA_MATH_XFER density under Formal load; keep single-commit integrity.

### P2 — Invent organ / study–zs dissociation

- **Sign:** HumanEval study ~90%, raw invent ~25–50% variance.  
- **Mechanism:** `code_study_ref_exec` exercises the python membrane on **canonical** bodies; zs requires **scaffold invent**.  
- **Genetic misread:** code_accuracy looks elite → breeding thinks code is solved.  
- **Rx:** Credit invent only on test-pass zs/held-out; denser entry-matched scaffolds; never free-pass strings.

### P3 — Knowledge organ entropy limit (MMLU-Pro)

- **Sign:** MMLU/ARC high; MMLU-Pro ~40%.  
- **Mechanism:** Same ranker, 10 distractors → margin collapse.  
- **Rx:** Stronger distinctive-token and subject-schema transfer; more soft-spot rounds on Pro only (already prioritized).

### P4 — Constraint generation (IFEval)

- **Sign:** ~50% raw.  
- **Mechanism:** Language organ must **obey surface constraints** without free length pass.  
- **Rx:** Constraint motors from prompt surface (partially present); gen_acc credit honest.

### P5 — Diagnostic noise organs

- collective / self_model at 0 — not driving HE/GSM failure; fix for pack health / breeding clarity.

### P6 — EMA under-sampling

- Habitat zs_ema for HE n=1, many suites n=3 — **do not treat EMA as medical truth**. Use raw_zs snapshots.

---

## 7. Pathophysiology diagram

```text
                    HIGH precision                         LOW / fragile precision
                    ─────────────                         ──────────────────────
                         │                                          │
              ┌──────────▼──────────┐                    ┌──────────▼──────────┐
              │ Closed option set   │                    │ Open program / number│
              │ Rank = fitness      │                    │ Synthesize = fitness │
              └──────────┬──────────┘                    └──────────┬──────────┘
                         │                                          │
              Stem + engram ranker                         Scaffold / expression
              (dense, stable)                              library (sparse tail)
                         │                                          │
              Transfer = same motor                        Transfer = new structure
              different stem                               under tests / exact key
                         │                                          │
              Study ≈ zs (often)                           Study >> zs (classic)
                         │                                          │
              Genome field OK                              Genome field OK
              Motor density high                           Motor density low
```

**Unified denominator:**  
Performance ∝ **match between exam physics and effector library density**, gated by Formal field + genetic coupling — **not** ∝ parameter count, **not** ∝ “how hard the genome tries.”

---

## 8. Prescription (research orders — biological only)

1. **Invent credit reform** — breed/select on held-out test-green invent, not study ref-exec.  
2. **Composition library growth** — keep expanding narrative templates under consciousness/energy load; SCHEMA transfer after success.  
3. **Per-organ genetic loci (next gen)** — optional: dedicated codon windows biasing composition vs invent vs knowledge (true multi-tissue genetics).  
4. **MMLU-Pro margin** — subject SCHEMA_SCI density; distinctive token weight already partial.  
5. **Always raw_zs for claims** — probe after each experiment; push CAPABILITY_AUDIT to state (gitignored) but report numbers in experiment commits.  
6. **Keep integrity rails** — no free grades, no pass@k fishing, membrane on.

### Rx applied (FSOT 2.1 Formal math → organism) — 2026-07-13

| Formal ingredient | Organism use |
|-------------------|--------------|
| `consciousness_factor` × `phase_variance` | Observer pathway opens **motor expressivity** when `observed=true` |
| `new_perceived_param` | Mild expressivity scale (perceived adjust family) |
| Domain `recent_hits` + exam load | growth_term-compatible load on invent/composition suites |
| Genetic `domain_coupling` | Fan-out of invent scaffolds + composition multi-op boost |
| Regime / \|S\| | Emergence amplifies expressivity after domain load |

| Diagnostic fix | Change |
|----------------|--------|
| **P2 invent/study** | Study `code_study_ref_exec` → skill only; **zs invent** updates `code_accuracy` |
| **P1 composition** | Credit composition primarily from expr/motor paths + zs invent success |
| **Breeding truth** | Higher weight on `bench_zs` + composition; invent-honest `code_accuracy` |
| **MMLU-Pro** | Formal expressivity amplifies distinctive stem tokens under observed load |
| **Field memory** | `FORMAL_FIELD` engrams after domain compute (no answers) |

---

## 9. Prognosis

- **Good:** Closed-set ranking tissue is SOTA-class on several surfaces; Formal/genetic field plumbing works; integrity dual-track is honest.  
- **Guarded:** Open invent + multi-step composition define the flagship gap vs full Formal ToE living competence.  
- **Plan:** Treat low suites as **missing effectors under correct physics**, not as proof the organism is non-biological. Expand effectors; refine genetic credit so evolution sees invent and composition truthfully.

---

## 10. Appendix — evidence pointers

| Artifact | Role |
|----------|------|
| `CAPABILITY_AUDIT.json` | Dual-track suite EMAs + raw_zs + integrity notes |
| `ACCURACY_AUDIT.json` | Per-org organ values; composition weak |
| `bench.rs` | suite→organ, Formal domain, motors, credit_organs |
| `genetic.rs` | genome_to_guidance |
| `lean_ground.rs` | Formal + codon runtime certificates |
| `docs/BIOLOGICAL_INTELLIGENCE_MODEL.md` | Normative design |

---

*Diagnostic status: complete for current pack snapshot. Update after each major organ experiment and commit a short “labs” section to the experiment message on GitHub.*
