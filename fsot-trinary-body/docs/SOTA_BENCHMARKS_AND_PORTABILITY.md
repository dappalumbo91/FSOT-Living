# SOTA benchmarks we can still grab + portability (QEMU / Kaggle / HF / GitHub)

## Further benchmarks (available, high leverage)

You already run: **MMLU, GSM8K, HellaSwag, TruthfulQA, MBPP, ARC-Easy/Challenge (science CSV)**.

### Priority add-ons (downloadable / open)

| Benchmark | What it tests | Fit to FSOT organs | Where |
|-----------|---------------|--------------------|--------|
| **MMLU-Pro** | Harder multi-choice knowledge (10 options) | knowledge + CHEW MCQ | HF / paper data |
| **GPQA (Diamond)** | PhD-level science MCQ | knowledge + chew depth | HF `Idavidrein/gpqa` |
| **HumanEval / HumanEval+** | Function synthesis + tests | code scaffold path (loader ready) | openai/human-eval |
| **LiveCodeBench** | Fresh competitive coding | code motor + tests | GitHub LiveCodeBench |
| **MATH / MATH-500** | Competition math | GSM motor + schemas | HF datasets |
| **IFEval** | Instruction following | language plan→act | HF |
| **BBH (BIG-Bench Hard)** | Multi-step reasoning | composition + gen_acc | HF |
| **Winogrande** | Commonsense coref | language / gen | HF (like HS) |
| **SWE-bench (Lite)** | Real repo bugfix | **agent** loop — stretch | GitHub SWE-bench |
| **ARC-AGI / ARC Prize** | Abstract grid programs | **new organ** (not science ARC CSV) | Kaggle ARC Prize 2026 |
| **HLE** | Expert closed questions | hard knowledge | paper / HF when released |

### Kaggle-friendly note

Kaggle competitions often want: **notebook**, **fixed runtime**, **no long-lived OS**, **submission format**. Our habitat is a **continuous living process**; competition entry is usually a **frozen solver** distilled from it (export policies / code tools / schemas), not the full TUI+body.

---

## QEMU / body nerve vs Kaggle / GitHub / Hugging Face

### What is QEMU-coupled today?

| Layer | Needs QEMU? | Portability |
|-------|-------------|-------------|
| **fsot-living** (mind, exams, benches, organs) | **No** | Pure host Rust; runs on Windows/Linux with `cargo` |
| **mind_bridge / trinary body** TCP or files | Optional | Host TCP `7368` or **files** (`BODY_STATUS` / motors) |
| **QEMU serial nerve** (`FSOT_BODY_QEMU_TCP` :7370) | Only for **OS/chassis rehearsal** | Not required for capability benchmarks |
| **Node / Python** for JS & code organs | Host tools | Need installed runtimes in target env |

So: **benchmarking SOTA does not require QEMU.** QEMU is a *body* / nerve path, not the intelligence core.

### Ramifications by platform

#### GitHub
- **Easy:** publish crate, CI `cargo test` / probe smoke, docs, datasets as LFS or download scripts.
- **Hard:** huge training data on D:\ — use download scripts + `.gitignore`; don’t ship multi-GB dumps in the repo.
- **Body:** document “file mode” and optional TCP; QEMU as advanced optional path.

#### Hugging Face
- **Spaces:** typically Python Gradio/Streamlit — wrap probe as **subprocess** or re-export a thin API (`ask` / `bench_eval`).
- **Datasets hub:** publish *fact cards / schema packs / EPISODE_CODE* as HF datasets; load via `FSOT_TRAINING_DATA`.
- **Models hub:** FSOT is not a weight dump; publish as **“living agent runtime”** + small state checkpoints (`LIVE_STATUS`, germline), not a single `.safetensors` LLM.
- **No QEMU** on free Spaces — keep body offline; mind-only mode.

#### Kaggle
- **Notebook kernel:** Linux, internet often off for submission, time limits, no interactive TUI, no QEMU, no G:\ media drives.
- **Strategy:**
  1. **Offline pack:** bake a `bench_bank` snapshot or small jsonl into the notebook dataset.
  2. **Headless binary:** prebuild `fsot-probe` or a slim `fsot-eval` for Linux x86_64, or compile in-notebook if toolchain allowed.
  3. **Submission** = predictions file / writeup, not a 500-tick habitat loop (unless competition allows long jobs).
  4. **Strip:** disable audio/video drive indexes, body bridge, TUI; env `FSOT_LEAN=1`, file-only state.
- **ARC Prize specifically:** needs grid DSL / search agent — separate from current science-ARC CSV and from QEMU.

### Practical “port modes”

| Mode | Env | Use |
|------|-----|-----|
| **Lab full** | Host + optional body/QEMU + D:\ data | Daily evolution |
| **Headless probe** | Host, no TUI | CI / CAPABILITY_AUDIT |
| **Competition lean** | Linux binary + bundled jsonl, no body, no media | Kaggle / cloud |
| **HF Space** | API wrapper, mind-only | Demo chat + small exams |

### Risks if we ignore this
- Coupling identity/fitness to body TCP breaks demos when chassis offline (already soft-degraded).
- Absolute Windows paths (`D:\training data`, `G:\`) fail on Linux — already partially mitigated by `FSOT_TRAINING_DATA` / `FSOT_BENCH_ROOT`.
- Multi-minute 500-tick probes exceed notebook limits — need `fsot-eval --ticks 50 --suites hellaswag`.

### Recommended next engineering (when you want)
1. `fsot-eval` CLI: suite filter, tick cap, JSON report only.  
2. `scripts/fetch_benchmarks.sh` for GPQA / HumanEval / MMLU-Pro into `FSOT_TRAINING_DATA`.  
3. `PORTABLE.md` + GitHub Actions smoke (no QEMU).  
4. Optional Docker image: living + python + node, body file-mode.

---

**Bottom line:** Keep QEMU for body/OS research; ship **mind + bench** as portable. Kaggle/HF want a **lean solver surface**, not the full Omen+QEMU lab.

---

## Downloaded on D:\training data (2026-07-12)

| Pack | Path | N |
|------|------|---|
| HumanEval | `humaneval/HumanEval.jsonl` | 164 |
| MMLU-Pro | `mmlu_pro/test.jsonl` | 12032 |
| MATH-500 | `math/math.jsonl` | 500 |
| Winogrande | `winogrande/validation.jsonl` | 1267 |
| BBH mix | `bbh/bbh_mix.jsonl` | 2250 |
| IFEval | `ifeval/train.jsonl` | 541 |
| Catalog | `SOTA_PACKS.md` | — |

**Gated / later:** GPQA (needs HF auth). ARC-AGI grids (Kaggle Prize).

**Wired in `bench.rs`:** `mmlu_pro`, `math`, `winogrande`, `bbh`, `ifeval`, `humaneval` suites + CAPABILITY_AUDIT.
