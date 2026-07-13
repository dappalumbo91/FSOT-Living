/**
 * FSOT Living Scalar — standard run (v3.0 core)
 * Ingests FSOT seed corpus + symbol sweep, then 2500 ticks.
 */

'use strict';

const { FSOTLivingScalar } = require('./fsot_living_core');
const { allSeeds } = require('./fsot_seed_corpus');
const { computeForDomain, CONSTANTS } = require('./fsot_scalar');

const entity = new FSOTLivingScalar('fsot-living-v30', false);

console.log('=== FSOT Living Scalar v3.0 (verified scalar engine) ===\n');
console.log(`k = ${CONSTANTS.k.toFixed(8)}`);
console.log(`Sample domains:`);
for (const d of ['quantum', 'biological', 'cosmological', 'neuroscience']) {
  const r = computeForDomain(d);
  console.log(`  ${d}: S=${r.S.toFixed(6)} regime=${r.regime}`);
}
console.log('');

// Structured FSOT knowledge
const seeds = allSeeds();
console.log(`Ingesting ${seeds.length} FSOT structured seeds...\n`);
for (const seed of seeds) {
  const result = entity.ingestInformation(seed);
  if (seed.domain) {
    console.log(
      `[${seed.domain}] S_org=${result.S.toFixed(4)} regime=${result.regime} ` +
        `pattern=[${result.generatedSignal.join(',')}]`
    );
  }
}

// Light symbol alphabet
const symbols = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
console.log(`\nIngesting ${symbols.length} symbol tokens...`);
for (const s of symbols) {
  entity.ingestInformation(s);
}

console.log('\nRunning 2500 ticks...\n');
for (let i = 0; i < 2500; i++) {
  if (i % 40 === 0 && seeds.length > 0) {
    entity.ingestInformation(seeds[i % seeds.length]);
  }
  entity.tickOnce();
  if ((i + 1) % 500 === 0) {
    const st = entity.getState();
    console.log(
      `t=${st.tick} S=${st.S.toFixed(4)} regime=${st.regime} c=${st.coherence.toFixed(3)} ` +
        `P=${st.vitalityPressure.toFixed(3)} mut+${st.acceptedMutations}/-${st.rejectedMutations} ` +
        `rules=${st.selfModificationPolicySize}/${st.signalGenerationLogicSize}/${st.domainRegulationLogicSize}`
    );
  }
}

const saved = entity.saveState();
const finalState = entity.getState();

console.log('\n=== Final State ===');
console.log({
  tick: finalState.tick,
  S: finalState.S,
  raw_S: finalState.raw_S,
  regime: finalState.regime,
  vitality: finalState.vitality,
  coherence: finalState.coherence,
  pressure: finalState.vitalityPressure,
  D_eff: finalState.params.D_eff,
  delta_psi: finalState.params.delta_psi,
  acceptedMutations: finalState.acceptedMutations,
  rejectedMutations: finalState.rejectedMutations,
  policySizes: {
    mod: finalState.selfModificationPolicySize,
    signal: finalState.signalGenerationLogicSize,
    domain: finalState.domainRegulationLogicSize,
  },
  integration: finalState.informationIntegration,
});

console.log('\nSelf-Modification Policy:', finalState.selfModificationPolicy);
console.log('Domain Regulation:', finalState.domainRegulationLogic);
console.log('\nRecent memory:');
console.log(
  entity.memory.slice(-5).map((m) => ({
    t: m.t,
    S: m.S,
    regime: m.regime,
    pressure: m.pressure,
    c: m.c,
    sizes: `${m.modPolicySize}/${m.signalLogicSize}/${m.domainLogicSize}`,
  }))
);
console.log(`\nSaved → ${saved}`);
