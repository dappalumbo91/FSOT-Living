# Memory tiers — disk · RAM · nerves · CPU

Your model, implemented:

| Layer | Analogy | Substrate | What lives there |
|-------|---------|-----------|------------------|
| **LTM** | Long-term memory / notes on desk | **Disk** `BODY_LTM.txt` | Genome loci fields, organs, breath, scale checkpoint |
| **STM** | Short-term / working memory | **RAM** soma fabric + intero ring | Live expression, pressure, motor_bias |
| **Nerves** | Phone line / pipeline | **TCP** (+ serial) | `MIND` / `MOTOR` in real time |
| **CPU** | Neuron cluster | Tick loop | FSOT pressure → trit collapse → organs |

## Flow

```
DISK LTM  --wake-->  RAM working set  <==TCP==>  living cortex
   ^                      |
   +---- sleep/consolidate (plastic rest + save)
```

- **SLEEP** / auto-consolidate: rest plastic loci, write LTM  
- **SAVE**: checkpoint without full rest  
- **WAKE**: rehydrate from disk (trinary fields, not frozen bits)

## Goldfish rule (scale)

| Tier | When | Breath | Pack | Knowledge |
|------|------|--------|------|-----------|
| lean | &lt;6GB, no GPU | narrow | 4 | 2k |
| standard | typical desktop | medium | 6 | 12k |
| expanded | ≥8GB VRAM or ≥24GB RAM | wide | 8 | 40k |
| **max** | ≥12GB VRAM **and** ≥24GB RAM (Omen 35L + 5070 class) | widest | 10 | 80k |

Force: `FSOT_SCALE=lean|standard|expanded|max` or `FSOT_LEAN=1`.  
Override probes: `FSOT_RAM_MB`, `FSOT_VRAM_MB`.

## Commands

```text
MOTOR SCALE     # report tank size
MOTOR SLEEP     # consolidate to LTM now
MOTOR SAVE      # checkpoint
MOTOR WAKE      # host applies BODY_LTM.txt
```

Ask living: `scale` · `body status` · `goldfish`
