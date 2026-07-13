# Lean ground — one family of invariants the runtime obeys

## Claim (scoped, honest)

The living scalar engine implements the **same algebraic skeleton** formalized in
FSOT 2.1 Lean `FSOT/Scalar.lean`:

```
S = k · (T1 + T2 + T3)
k = φ · (perceived_param_base · √2) / ln(π) · 0.99
```

with regime labels by sign thresholds used in the Rust port.

We do **not** claim “Lean proved the whole organism.”  
We claim **circle-back**: formal core ⟷ numbers the habitat actually runs.

## Runtime checks (`fsot-living-rust/src/lean_ground.rs`)

| Check | Meaning |
|-------|---------|
| `k_matches_lean_scalar` | Rust `k()` vs reconstructed Lean `k` (rel err &lt; 1e-9) |
| `S_eq_k_times_sum_terms` | `raw ≈ T1+T2+T3` and `S ≈ k·raw` |
| `domain_ledger_finite_regimes` | Domain S finite; regime ∈ {emergence,damping,neutral} |
| `guidance_emergence_d_eff_bounds` | Genome guidance stays in documented clamps |
| `regime_thresholds_consistent` | Sign rule samples behave coherently |

On habitat boot / main: prints `lean_ground PASS n/n · k=…`.

## How to re-verify against Lean tree

```text
Desktop\FSOT-2.1-Lean\FSOT-2.1-Lean-main\FSOT-2.1-Lean-main\FSOT\Scalar.lean
```

```powershell
cd "…\fsot-living-rust"
cargo test -p fsot-living lean_ground -- --nocapture
```

When Lean `k` or term definitions change, update `scalar.rs` constants **and**
this check — that is the living contract.
