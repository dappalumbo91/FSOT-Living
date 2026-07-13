# Closed loop — living cortex ↔ trinary body

## Architecture

```
┌─────────────────────────────┐     files      ┌──────────────────────────┐
│  fsot-living-rust (cortex)  │◄──────────────►│  mind_bridge / QEMU body │
│  body_bridge.rs             │  BODY_*        │  soma + Mind ABI         │
└─────────────────────────────┘                └──────────────────────────┘
         │                                              │
         │  LIVE_STATUS.json                            │  MIND telemetry
         ▼                                              ▼
    external tools                              serial MOTOR (QEMU)
```

## Files (`state/habitat-rust/`)

| File | Writer | Reader |
|------|--------|--------|
| `BODY_STATUS.json` | mind_bridge | living `body_bridge` |
| `BODY_TELEMETRY.jsonl` | mind_bridge | tools / review |
| `BODY_MOTOR.txt` | living cortex | mind_bridge (consume+clear) |
| `BODY_OUTBOX.txt` | mind_bridge | debug |
| `LIVE_STATUS.json` | living export | tools (includes `body_*`) |

## Quick start (TCP real-time)

**Terminal A — body host:**
```powershell
cd "...\fsot-trinary-body"
.\scripts\run_tcp_body.ps1
# or: cargo run --release -p mind_bridge -- --tcp
```

**Terminal B — living cortex:**
```powershell
cd "...\fsot-living-rust"
.\start_fsot_living_lean.bat   # 4GB / 2012 Mini
# or: .\start_fsot_living.bat
# In TUI: type "body status" + Enter
# Header body= should show "tcp" when linked
```

Cortex every ~25 ticks sends MOTOR policy over **TCP** (file fallback if TCP down).  
Body pressure couples into pack organism pressure.

See [TRANSPORT_TCP_VS_FILES.md](TRANSPORT_TCP_VS_FILES.md) and [MIN_SPEC_MAC_MINI_2012.md](MIN_SPEC_MAC_MINI_2012.md).

## QEMU interactive serial

```powershell
cd "...\fsot-trinary-body"
.\scripts\run_qemu_serial.ps1
# Then type:
MOTOR STATUS
MOTOR BREATHE
MOTOR ORGANS
MOTOR READ /body/genome
```

Requires `qemu-system-x86_64` and preferably `OVMF_PATH`.

## Dialogue

In living TUI ask:

- `body status`
- `body link`
- `chassis`
- `mind abi`
