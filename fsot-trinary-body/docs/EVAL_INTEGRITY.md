# Eval integrity — no fudge frame

## Framing

This is a **living FSOT organism** (organs, homeostasis, exam pressure) — **not** an LLM and **not** scored by parameter count. Gaps or strengths are **tissue capability**. Do not frame soft spots as “expected for a small model.”

## Dual track (do not mix)

| Track | What it is | What it is **not** |
|-------|------------|---------------------|
| **STUDY** (`suite_acc`) | Teach → test / motor practice | Invent-from-blank |
| **ZERO-SHOT** (`suite_zs` EMA) | Held-out; same-item answer engrams blocked | KEY_STORE for that id |
| **RAW ZS** (`suites_zeroshot_raw`) | Fresh hard pass/fail counts | Sticky pack EMA |

## Guards (implemented)

1. **ZS same-item block** — retrieval skips lines containing this item’s id, KEY_STORE, BENCH_KEY/NUM for that id, REMEDIATE for that id.  
2. **Cross-item transfer allowed** — EPISODE_*, SCHEMA_*, primitives (biological transfer).  
3. **Code study** — may re-run reference under mode `code_study_ref_exec` (motor practice); **not** invent.  
4. **Code zs requires test exec** — no string-key free pass (removed).  
5. **GSM** — single top motor commit (no pass@k fishing against the answer).  
6. **BBH** — strict answer match (no “true” substring in a dump); boolean motor via python.  
7. **IFEval** — no free pass for “long enough”; only verifiable constraint checks.  
8. **Raw zs prefers never-taught ids** this process (`taught_ids`).  
9. **Audit `integrity` + `suites_zeroshot_raw`** in `CAPABILITY_AUDIT.json`.  
10. **Small n** listed — wide CI; do not over-claim on n&lt;15.

## What can still look “too high”

- **Study** near 100% (esp. code after reference exec) is **practice**, not invent.  
- **Pack zs EMA** can lag above true ability if pre-integrity history was inflated — trust **raw_zs**.  
- HellaSwag predictive rankers can be strong; still require **n≥15** before claiming.  
- **Loader traps:** first-N on blocked files (BBH was all `boolean_expressions`) inflates a motor pathway — use **stratified** loads.  
- Prefer **raw_zs + per-suite n** when comparing to external tables.

## Lean circle-back (FSOT-2.1-Lean)

Source: https://github.com/dappalumbo91/FSOT-2.1-Lean  

- Living `scalar::domain_params` follows **`FSOT.Formal.Scalar.get_domain_params`** (proved signs), not only the Float convenience map.  
- `lean_ground` checks: `k`, `S=k·(T1+T2+T3)`, domain signs (quantum>0, cosmo<0, chemical>0, neural>0, medical>0, electron>0, ai≤0), codon dual-axis 64/8/27/3.  
- Bench exams blend **suite → Formal domain** (code→energy, math→electron, BBH→ai, MCQ→molecular, HS/Wino→neural) into D_eff / observer / recent_hits before CHEW.

## Integrity audit findings (2026-07-12)

| Issue | Fix |
|-------|-----|
| Code zs string-key free pass | require unit-test exec |
| IFEval “long enough” free pass | only verifiable constraints |
| BBH answer in long pack dumps | strict final-answer match |
| BBH bank = first task only | stratified by `task` |
| Pack zs EMA sticky | prefer `suites_zeroshot_raw` |
| Python hang on timeout | kill process tree + unique scratch files |

## How we use numbers

- **Honest competitive signal:** `suites_zeroshot_raw` + bank suite zs (n≥15 preferred).  
- **Curriculum health:** study track.  
- **Dialogue:** query battery.  
