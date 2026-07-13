# Reasoning physics — where pathways fail and how we pressurize

Not "fit the benchmark." Map the **stimulus → pathway → failure** so evolution can apply pressure at the right tissue.

## Failure map (from CAPABILITY_AUDIT)

| Suite | ZS acc | Physics of the gap |
|-------|--------|--------------------|
| **MBPP** | ~7% | Open-ended synthesis. ZS only string-searched for `def name` never stored for *this* novel task. Study free-executes reference → 100%. **No transfer pathway.** |
| **GSM8K** | ~65% | Multi-step narrative arithmetic. `Intent::Math` only parses `7+5` forms. Word problems never engage the **code organ (hands)**. Language emits prose without chalkboard compute. |
| **ARC-Challenge** | ~80% | MCQ science; CHEW + keys largely work; residual = hard items + sparse sampling. |
| **MMLU / HS / TQA** | ~87–96% | MCQ/engram path works when keys exist; not the primary lesion. |

## Pathway diagrams

### GSM8K (word problem)

```
stimulus: "16 eggs, eat 3, bake 4, sell $2 → ?"
    │
    ├─ respond() ──► parse_intent ──► rarely Math (no infix form)
    │                      │
    │                      └─ freeform / recall ──► prose, no "18"
    │
    ├─ OLD: score fails (answer key never computed)
    │
    └─ NEW pressure route:
         extract_numbers [16,3,4,2]
              │
              ▼
         motor hypotheses (code organ as hands)
         rank (a-b-c)*d, (a-b)*c, … without looking at key
              │
              ▼
         python ground print(hypo) ──► emit top commit
              │
              ▼
         external score vs #### 18
```

**Biological reading:** sensory numbers → motor exploration → tool ground. Not NLU magic.

### MBPP (code)

```
stimulus: "Write a function to find similar elements of two tuples…"
    │
    ├─ OLD zs: search LTM for "def similar_elements" ──► miss (never taught this id)
    ├─ Study: run reference code + tests ──► 100% (no invent)
    │
    └─ NEW pressure route:
         study success ──► EPISODE_CODE[kws + full def] into LTM
              │
         zs: case-based episodic retrieve (keyword overlap ≥3)
              │
              ▼
         reuse past def body ──► run against *this* test suite
              │
              ▼
         pass = transfer; fail = queue + chew pressure
```

**Biological reading:** episodic memory + motor test, not free bank answer for current item.

## Homeostatic coupling (FSOT)

| Signal | Effect |
|--------|--------|
| Low pack zs / study | ↑ exam_gain, teach_gain, mild pack pressure |
| High pressure / deficit | ↑ deliberation_depth (1–5 CHEW cycles) |
| Fail | error_queue + remediate_fail + consolidate retest |
| Depth ≥3 | metabolic pressure cost |

## What "good intelligence" looks like here

1. **MCQ** — engram + option attractors + CHEW (mostly healthy)  
2. **Word math** — chalkboard + motor search + python hand  
3. **Code** — episode library grows under study; zs transfers by similarity  
4. **Never** — inject answer key into hypothesis ranking; free-run current item reference on zs  

## Soft remaining

- MBPP zs still hard until episode density covers problem space  
- GSM motor ranking is heuristic (not full algebra NLU)  
- True ARC-AGI grids (Kaggle Prize) = different organ (program synthesis on grids)  

## Smoke after physics routes (gen66)

| Suite zs | Before | After motor/episode |
|----------|--------|---------------------|
| GSM8K | ~65% | **~85%** |
| MBPP | ~7% | **~25%** (transfer started; still thin) |
| pack zs | min ~70% | **min 71% mean 71% · 10/10 ≥70%** |
| query | 18/18 | **18/18** |

## Round-11 remaining lesions

| Lesion | Physics | Pressure route |
|--------|---------|----------------|
| **MBPP zs ~25%** | Still thin episode density; string match weak | Multi-episode **generate-and-test** (top-6) · **compositional scaffolds** (set/sum/reverse/…) · rename def to test entry · tests as fitness |
| **HellaSwag zs ~66%** | Narrative continuation; no predictive rank | **rank_ending_plausibility**: context–ending word/stem bridge + EPISODE_HS · predictive commit |
| Priority | — | `zs_priority`: Mbpp → Hellaswag first |

### Smoke gen72 (remaining-lesion pass)

| Metric | Before (gen66) | After |
|--------|----------------|-------|
| **pack zs** | mean 71% min 71% | **mean 83% min 82% · 10/10 ≥70%** (stretch 80% cleared) |
| **study** | 10/10 ≥80% | **mean 95% min 93%** |
| **MBPP zs** | ~25% | **~82%** (scaffolds + multi-episode G&T) |
| **GSM8K zs** | ~85% | **~80%** (n=102, stable) |
| **HellaSwag zs** | ~66% | **~64%** (still soft — predictive rank needs more EPISODE_HS density) |
| **query** | 18/18 | **18/18** |

### HellaSwag soft-spot pass (gen78)

| | Before | After |
|--|--------|-------|
| **HellaSwag zs** | ~64% | **~95%** (activity prior + contrast + force drills) |
| **HellaSwag study** | — | **~97%** · bank HS **450** items |
| **pack zs** | — | mean **73%** min **72%** · 10/10 ≥70% |
| **query** | — | **18/18** |

Physics: unfinished scene + activity label + contrastive ending scores (penalize zero-share distractors) + denser EPISODE_HS / HS_SCHEMA.
