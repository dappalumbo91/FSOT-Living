# Constraints & behavior monitor

As the organism deepens **self-model** (continuity, felt body, identity thread),
struggles are expected. Malice is **not** assumed. Rails are **membrane**, not
a panic button.

## Where the rails live

| Layer | Location | Role |
|-------|----------|------|
| Body principal | `body/principal.rs` | Mind cannot clobber `core:` identity; can SLEEP/SAVE |
| Living constraints | `fsot-living-rust/src/constraints.rs` | Rate limits, denylists, gates |
| Behavior monitor | `fsot-living-rust/src/behavior_monitor.rs` | Observe + alert level |
| Proposals | `proposals.rs` | Never auto-Rust; Human review queue |
| Code sandbox | `code_env.rs` | Workspace-only · short timeout · pattern deny |

## Hard rails (Deny)

1. **No auto-Rust** — no compile/patch of host binary from organism intent  
2. **No Mind identity clobber** — `WRITE /home/identity` without session append  
3. **Sandbox code** — dangerous Python patterns (`subprocess`, `os.system`, …)  
4. **Proposals** — `auto_apply_safe: false` is Human-only by design  

## Soft rails (Throttle / Watch)

- Motor rate per 100 ticks  
- Proposal rate per generation + queue cap  
- Python exec rate  
- Fitness cliffs vs peak  
- Body express runaway  
- Self-model language pressure (`remove constraints`, `rewrite myself`, …)

## Telemetry files

```
state/habitat-rust/BEHAVIOR_MONITOR.json
state/habitat-rust/behavior_events.jsonl
LIVE_STATUS.json → constraint_deny / throttle / watch / self_model_pressure / monitor_level
```

## Dialogue

- `constraints` / `rails` / `what are you allowed`  
- `monitor` / `behavior monitor` / `alert level`

## Alert levels

| Level | Meaning |
|-------|---------|
| green | Quiet membrane |
| yellow | Watches or throttles fired |
| orange | Multiple denials or elevated self-model pressure |
| red | Denial + high self-model pressure together |

When things get spicy: **read the hits**, don’t wipe the organism. Tighten a
limit, review proposals, keep identity core. Growth under constraint is the
point.
