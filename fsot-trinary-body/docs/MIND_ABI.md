# Mind ABI — body ↔ cortex wire protocol

## Direction

| Direction | Line prefix | Role |
|-----------|-------------|------|
| Body → Cortex | `MIND …` | Telemetry (interoception + phenotype) |
| Cortex → Body | `MOTOR <verb> …` | Commands |
| Body → Cortex | `ACK verb=… detail=…` | Command ack |
| Body → Cortex | `DATA path=…` | Membrane read result |
| Body → Cortex | `ORGANS …` | Organ listing |
| Body → Cortex | `REPORT …` | Full sample dump |
| Body → Cortex | `BODY event=…` | Internal body events |

## MOTOR verbs

| Verb | Example | Effect |
|------|---------|--------|
| `STATUS` | `MOTOR STATUS` | Emit fresh `MIND` frame |
| `ATTEND` | `MOTOR ATTEND heap` | Spotlight channel (temp/tsc/heap/uptime/breath) |
| `BREATHE` | `MOTOR BREATHE` | Rest fabric — re-superpose plastic loci |
| `REPORT` | `MOTOR REPORT` | Detailed sample |
| `EXPRESS` | `MOTOR EXPRESS` | Motor bias +0.85 (emergence pressure) |
| `REPRESS` | `MOTOR REPRESS` | Motor bias −0.85 |
| `ORGANS` | `MOTOR ORGANS` | List membrane |
| `READ` | `MOTOR READ /body/genome` | Read organ content |
| `WRITE` | `MOTOR WRITE /organs/knowledge text` | Write organ |
| `NOTE` | `MOTOR NOTE hello` | Append to notes organ |

## MIND fields (selected)

- `seq`, `focus`, `breath` — interoception meta  
- `*_raw` — continuous sensors  
- `*_trit` — **trinary** collapsed channel (−1 / 0 / +1)  
- `pressure` — genetic fabric drive  
- `repress` / `latent` / `express` — phenotype tallies  
- `mean_expr`, `plast` — dynamics (not binary-static)  
- `genome_fp` — fingerprint  

## Organ membrane paths

| Path | Organ |
|------|--------|
| `/body/status` | Live status string |
| `/body/genome` | Genome snapshot |
| `/body/intero` | Latest intero summary |
| `/organs/knowledge` | Fact / knowledge buffer |
| `/organs/tools` | Tool registry stub |
| `/organs/notes` | Cortex notes |
| `/organs/bridge` | Living-habitat bridge mailbox |

## Host bridge files

`mind_bridge` (and living habitat) use:

```
state/habitat-rust/
  BODY_TELEMETRY.jsonl   # append-only MIND/ACK stream
  BODY_MOTOR.txt         # write MOTOR lines; bridge consumes
  BODY_STATUS.json       # snapshot for mind gym
  BODY_OUTBOX.txt        # last outbox
```

## Living habitat join

1. Keep `fsot-living-rust` running as mind gym.  
2. Run `cargo run -p mind_bridge -- 100` for body host cortex.  
3. Living side (later): read `BODY_STATUS.json` / telemetry; write `BODY_MOTOR.txt`.  
4. QEMU path: same `MOTOR`/`MIND` lines on serial.

## Design rules

1. **No opaque binary control** as primary protocol.  
2. Trit indices always present alongside continuous fields.  
3. Genes remain dynamic (BREATHE re-opens loci).  
4. Organs have Trit expression — not just byte bags.
