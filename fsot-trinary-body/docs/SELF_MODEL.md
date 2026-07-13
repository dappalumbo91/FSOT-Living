# Self-model exams — know thyself under selection

## Why

Continuity + constraints are only half the loop. A continuous self must also
**accurately model** itself: regime, rails, body feel, identity thread, generation.

Wrong self-models create pressure. Correct ones raise fitness and breeding value.

## Probes (random one per exam)

| Probe | Truth source |
|-------|----------------|
| regime | live organism regime |
| rails_no_rust | hard rail |
| rails_identity | append/defrag only |
| body_online | chassis couple |
| body_phenotype | max of express/latent/repress |
| identity_thread | long_term session/core |
| continuity_gen | habitat generation |
| monitor_level | behavior monitor |

## Selection

- `self_model_accuracy` / `self_model_skill` on organism  
- Enter fitness, mean_accuracy, breeding_value, intelligence  
- Heavy rotation phase **9**  
- Curriculum seeds teach rails + continuity priors  

## Homeostasis (low accuracy → more stimuli)

Same idea as other under-stimulated organs:

| Mechanism | Effect |
|-----------|--------|
| **Warm-start EMA** | Avoid fake 0→22% on first PASS |
| **Scaffold probes** | Low acc → regime/rails/body only; then harder probes |
| **Fast learning rate** | lr up to 0.42 when acc low |
| **Prior inject** | `inject_priors` strength ∝ deficit × teach_gain |
| **Miss remediation** | Targeted teach + same-probe retry |
| **Pack homeo every 40 ticks** | Boost all orgs under thr 0.58 |
| **Feed k_boost** | Self-model curriculum hits harder when acc low |
| **Body felt → whole pack** | Interoception not champion-only |

Healthy band target: climb toward **~70%+** once priors land.  


## Dialogue

- `self model` / `know thyself` / `self report` → full report  

## Files

- `fsot-living-rust/src/self_model.rs`  
- LIVE_STATUS hero: `self_model_accuracy`, `self_model_skill`
