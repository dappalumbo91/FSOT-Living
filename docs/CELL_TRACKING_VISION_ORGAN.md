# Cell-tracking vision organ (Biohub v50)

Synced with [FSOT-2.1-Lean](https://github.com/dappalumbo91/FSOT-2.1-Lean) `vendor/kaggle_biohub_review/v50/`.

## Competitive pipeline (CPU Kaggle)

```
zarr → U-Net FT detect (conf-ranked NMS) → FSOT SequenceTracker → ILP/lite → submission.csv
```

## Living adaptive rule (`cell_tracking.rs`)

| Proxy | Behavior |
|-------|----------|
| ≥ 0.90 | Dormant — conf-ranked NMS only (math winning) |
| 0.62–0.90 | Neutral — no emergence over-prune |
| < 0.62 | accuracy_homeo emergence/damping on det/NMS |

## State file

`state/habitat-rust/KAGGLE_CELL_TRACKING_PROXY.json` — update after each Kaggle feedback round.

## Rust module

`fsot-living-rust/src/cell_tracking.rs` — canonical source; Python `fsot_living_emergence.py` ports from here.