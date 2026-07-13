/**
 * FSOT Living Scalar — long run 5000 ticks (v3.0 core)
 */

'use strict';

const { FSOTLivingScalar } = require('./fsot_living_core');
const { allSeeds } = require('./fsot_seed_corpus');
const { CONSTANTS } = require('./fsot_scalar');

const entity = new FSOTLivingScalar('fsot-living-v30-long', false);
const seeds = allSeeds();

console.log('=== FSOT Living Scalar v3.0 LONG RUN (5000 ticks) ===\n');
console.log(`k=${CONSTANTS.k.toFixed(8)} | seeds=${seeds.length}\n`);

for (const seed of seeds) {
  entity.ingestInformation(seed);
}

const TICKS = 5000;
for (let i = 0; i < TICKS; i++) {
  if (i % 30 === 0) {
    entity.ingestInformation(seeds[i % seeds.length]);
  }
  entity.tickOnce();

  if ((i + 1) % 1000 === 0) {
    const st = entity.getState();
    console.log(
      `t=${st.tick} S=${st.S.toFixed(4)} ${st.regime} c=${st.coherence.toFixed(3)} ` +
        `P=${st.vitalityPressure.toFixed(3)} D_eff=${st.params.D_eff.toFixed(2)} ` +
        `mut+${st.acceptedMutations}/-${st.rejectedMutations} ` +
        `rules=${st.selfModificationPolicySize}/${st.signalGenerationLogicSize}/${st.domainRegulationLogicSize}`
    );
  }
}

const saved = entity.saveState();
const finalState = entity.getState();

console.log('\n=== Long-Run Final ===');
console.log({
  tick: finalState.tick,
  S: finalState.S,
  raw_S: finalState.raw_S,
  term1: finalState.term1,
  term2: finalState.term2,
  term3: finalState.term3,
  regime: finalState.regime,
  vitality: finalState.vitality,
  coherence: finalState.coherence,
  pressure: finalState.vitalityPressure,
  params: {
    D_eff: finalState.params.D_eff,
    delta_psi: finalState.params.delta_psi,
    emergencePreference: finalState.params.emergencePreference,
  },
  mutations: {
    accepted: finalState.acceptedMutations,
    rejected: finalState.rejectedMutations,
  },
  structures: {
    selfModificationPolicy: finalState.selfModificationPolicy,
    signalGenerationLogic: finalState.signalGenerationLogic,
    domainRegulationLogic: finalState.domainRegulationLogic,
  },
});

console.log('\nLast 5 memory samples:');
console.log(entity.memory.slice(-5));
console.log(`\nSaved → ${saved}`);
