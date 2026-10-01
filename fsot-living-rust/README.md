# FSOT Living — Rust TUI

Native terminal. **No browser.** Evolutionary multimodal organism.

## Run

```powershell
cd "fsot-living-rust"
.\start_fsot_living.bat
```

## Keys

| Key | Action |
|-----|--------|
| **Type + Enter** | Ask the champion |
| **Esc** | Quit |
| **Tab** | Toggle input focus |
| **Space** (not typing) | Pause / resume |

## Queries

```
hello / status / organs
codon ATG
domain quantum | cosmological | biological
genome | fit | tools
py: 2+40
py: print(sum(range(10)))
see
see organism_workspace/vision_lib/warm_emergence.png
forge double | forge add | forge fsot_sign
help
```

## Organs under evolutionary pressure

| Organ | Pressure | Interaction |
|-------|----------|-------------|
| Cognition | codon / regime / bio exams | status, domain, genome |
| **Code** | Python sandbox challenges | `py: …` |
| **Vision** | image regime / color exams | `see …` |
| **Tools** | forge + verify scripts | `forge …` → `organism_workspace/tools/` |

Fitness = cognitive + **code_acc** + **vision_acc** + tools. Skills **inherit** across generations.

## Live review (for external agents)

While TUI runs: `../state/habitat-rust/LIVE_STATUS.json`

```powershell
cargo run --release --bin fsot-probe
```

## Design note

Gaps close under **selection pressure**. Python is the internal thought / practice language (interpretable). JS and other interpreters can plug into the same sandbox pattern next.
