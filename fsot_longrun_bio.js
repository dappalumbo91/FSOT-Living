/**
 * Long bio run — single organism guided by genetic genome + deep Lean ingest.
 * Default 8000 ticks.
 */

'use strict';

const { FSOTLivingScalar } = require('./fsot_living_core');
const { loadGeneticBundle, geneticSeeds, genomeToGuidance } = require('./fsot_genetic_data');
const { allLeanSeeds, findLeanRoot } = require('./fsot_lean_ingest');
const { computeForDomain } = require('./fsot_scalar');

const TICKS = parseInt(process.argv[2] || '8000', 10);

const genetic = loadGeneticBundle();
const leanSeeds = allLeanSeeds();
const geneSeeds = geneticSeeds(genetic);
const bio = computeForDomain('biological');

const entity = new FSOTLivingScalar('fsot-bio-long', false);
const genome = genetic.organism?.sequence || [
  'ATG', 'CCG', 'AAC', 'GGT', 'AAT', 'AGC', 'AGT', 'GGT', 'CTG', 'TCG', 'TTA', 'TAA',
];
entity.genome = genome.slice();
const guide = genomeToGuidance(genome, genetic.codons);
entity.params.D_eff = guide.D_eff;
entity.params.delta_psi = guide.delta_psi;
entity.params.emergencePreference = guide.emergencePreference;
entity.domains.domainCoupling = guide.domainCoupling;
entity.targetS = bio.S;
entity.genomeGuidance = guide;

console.log('=== FSOT Living Scalar — LONG BIO RUN ===\n');
console.log(`Ticks: ${TICKS}`);
console.log(`Genetics: codons=${genetic.loaded.codons} AA=${genetic.loaded.amino} genome=${genetic.loaded.genome} fasta=${genetic.loaded.fasta}`);
console.log(`Lean: ${findLeanRoot() || 'n/a'}`);
console.log(`Bio domain S=${bio.S.toFixed(6)}`);
console.log(`Genome guidance: D_eff=${guide.D_eff.toFixed(2)} δψ=${guide.delta_psi.toFixed(3)} AA=${guide.aaChain}\n`);

// Full knowledge ingest
const bath = [...leanSeeds, ...geneSeeds];
console.log(`Ingesting ${bath.length} seeds (Lean + genetic)...`);
for (const seed of bath) {
  entity.ingestInformation(seed);
}

console.log('Running...\n');
for (let i = 0; i < TICKS; i++) {
  if (i % 35 === 0) entity.ingestInformation(geneSeeds[i % geneSeeds.length]);
  if (i % 50 === 0) entity.ingestInformation(leanSeeds[i % leanSeeds.length]);
  if (i % 40 === 0) {
    // re-assert biological guidance
    const g = genomeToGuidance(entity.genome, genetic.codons);
    entity.params.D_eff = entity.params.D_eff * 0.7 + g.D_eff * 0.3;
    entity.params.delta_psi = entity.params.delta_psi * 0.7 + g.delta_psi * 0.3;
  }
  entity.tickOnce();

  if ((i + 1) % 1000 === 0) {
    const st = entity.getState();
    console.log(
      `t=${st.tick} S=${st.S.toFixed(4)} ${st.regime} c=${st.coherence.toFixed(3)} ` +
        `P=${st.vitalityPressure.toFixed(3)} D_eff=${st.params.D_eff.toFixed(2)} ` +
        `int=${st.informationIntegration.toFixed(3)} mut+${st.acceptedMutations}/-${st.rejectedMutations} ` +
        `rules=${st.selfModificationPolicySize}/${st.signalGenerationLogicSize}/${st.domainRegulationLogicSize}`
    );
  }
}

const saved = entity.saveState();
const finalState = entity.getState();
console.log('\n=== Final Bio-Guided State ===');
console.log({
  tick: finalState.tick,
  S: finalState.S,
  raw_S: finalState.raw_S,
  regime: finalState.regime,
  bioTarget: bio.S,
  D_eff: finalState.params.D_eff,
  delta_psi: finalState.params.delta_psi,
  coherence: finalState.coherence,
  pressure: finalState.vitalityPressure,
  integration: finalState.informationIntegration,
  mutations: { accepted: finalState.acceptedMutations, rejected: finalState.rejectedMutations },
  genome: entity.genome.join('-'),
  aaChain: guide.aaChain,
});
console.log(`Saved → ${saved}`);
