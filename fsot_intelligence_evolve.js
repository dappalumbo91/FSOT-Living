/**
 * FSOT Intelligence Evolution
 * Population evolves under cognitive task fitness (not only scalar homeostasis).
 * Goal path: FSOT-native intelligence — structurally different from fixed-net AI.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { FSOTLivingScalar, STATE_DIR } = require('./fsot_living_core');
const {
  loadGeneticBundle,
  genomeToGuidance,
  mutateGenome,
  geneticSeeds,
} = require('./fsot_genetic_data');
const { allLeanSeeds, findLeanRoot } = require('./fsot_lean_ingest');
const { computeForDomain } = require('./fsot_scalar');
const { teach, runCognitiveBattery } = require('./fsot_cognition');

function applyGenome(entity, sequence, codonMap) {
  const g = genomeToGuidance(sequence, codonMap);
  entity.genome = sequence.slice();
  entity.genomeGuidance = g;
  entity.params.D_eff = entity.params.D_eff * 0.35 + g.D_eff * 0.65;
  entity.params.delta_psi = entity.params.delta_psi * 0.35 + g.delta_psi * 0.65;
  entity.params.emergencePreference =
    entity.params.emergencePreference * 0.4 + g.emergencePreference * 0.6;
  entity.domains.domainCoupling = Math.min(
    1.55,
    entity.domains.domainCoupling * 0.5 + g.domainCoupling * 0.5
  );
  entity.targetS = entity.targetS * 0.6 + 0.48 * 0.4;
  return g;
}

function cloneChild(parent, id, codonMap) {
  const child = new FSOTLivingScalar(id, false);
  child.params = JSON.parse(JSON.stringify(parent.params));
  child.domains = JSON.parse(JSON.stringify(parent.domains));
  child.selfModificationPolicy = JSON.parse(JSON.stringify(parent.selfModificationPolicy));
  child.signalGenerationLogic = JSON.parse(JSON.stringify(parent.signalGenerationLogic));
  child.domainRegulationLogic = JSON.parse(JSON.stringify(parent.domainRegulationLogic));
  child.recallPolicy = JSON.parse(JSON.stringify(parent.recallPolicy || []));
  child._codonTable = { ...(parent._codonTable || {}) };
  child.genome = mutateGenome(parent.genome || [], 0.1);
  applyGenome(child, child.genome, codonMap);
  return child;
}

function totalFitness(org, bioTargetS) {
  const sAlign = 1 - Math.min(1.2, Math.abs(org.S - bioTargetS));
  return (
    (org.intelligenceScore || 0) * 1.5 +
    (org.cognitiveAccuracy || 0) * 2.0 +
    sAlign * 0.6 +
    (org.regime === 'emergence' ? 0.25 : -0.1) -
    (org.vitalityPressure || 0) * 0.35 +
    (org.informationIntegration || 0) * 0.4
  );
}

function runIntelligenceEvolution(options = {}) {
  const popSize = options.popSize ?? 10;
  const generations = options.generations ?? 18;
  const ticksPerGen = options.ticksPerGen ?? 60;
  const eliteCount = options.eliteCount ?? 3;
  const batteryRounds = options.batteryRounds ?? 16;

  const genetic = loadGeneticBundle();
  const leanSeeds = allLeanSeeds();
  const geneSeeds = geneticSeeds(genetic);
  const bio = computeForDomain('biological');
  const bioTargetS = bio.S;
  const baseGenome =
    genetic.organism?.sequence ||
    ['ATG', 'CCG', 'AAC', 'GGT', 'AAT', 'AGC', 'AGT', 'CTG', 'TCG', 'TAA'];

  console.log('=== FSOT Intelligence Evolution ===\n');
  console.log(`Goal: evolve FSOT-native intelligence under cognitive task fitness`);
  console.log(
    `Genetics: codons=${genetic.loaded.codons} AA=${genetic.loaded.amino} genome=${genetic.loaded.genome}`
  );
  console.log(`Lean: ${findLeanRoot() || 'n/a'}`);
  console.log(
    `pop=${popSize} gens=${generations} ticks/gen=${ticksPerGen} battery=${batteryRounds}\n`
  );

  // Curriculum: lean + genetic knowledge
  const curriculum = [
    ...leanSeeds.filter((s) => s.type === 'fsot_domain' || s.type === 'fsot_math').slice(0, 30),
    ...geneSeeds.filter((s) => s.type === 'fsot_codon' || s.type === 'fsot_genome').slice(0, 70),
    ...geneSeeds.filter((s) => s.type === 'fsot_protein').slice(0, 20),
  ];

  let population = [];
  for (let i = 0; i < popSize; i++) {
    const org = new FSOTLivingScalar(`intel-${i}`, false);
    const genome = i === 0 ? baseGenome.slice() : mutateGenome(baseGenome, 0.12 + i * 0.015);
    applyGenome(org, genome, genetic.codons);
    teach(org, curriculum);
    population.push(org);
  }

  const history = [];
  let hallOfFame = null;

  for (let gen = 0; gen < generations; gen++) {
    for (const org of population) {
      for (let t = 0; t < ticksPerGen; t++) {
        if (t % 20 === 0) {
          teach(org, [curriculum[t % curriculum.length]]);
        }
        if (t % 25 === 0) applyGenome(org, org.genome, genetic.codons);
        org.tickOnce();
      }

      // Cognitive exam every generation
      runCognitiveBattery(org, {
        rounds: batteryRounds,
        codonMap: genetic.codons,
        bioTargetS,
      });
    }

    const ranked = population
      .map((org) => ({
        org,
        fit: totalFitness(org, bioTargetS),
        acc: org.cognitiveAccuracy || 0,
        intel: org.intelligenceScore || 0,
        S: org.S,
        regime: org.regime,
        mem: (org.workingMemory || []).length,
        recallN: (org.recallPolicy || []).length,
        aa: org.genomeGuidance?.aaChain?.slice(0, 20) || '',
      }))
      .sort((a, b) => b.fit - a.fit);

    const best = ranked[0];
    const avgAcc = ranked.reduce((s, r) => s + r.acc, 0) / ranked.length;
    const avgFit = ranked.reduce((s, r) => s + r.fit, 0) / ranked.length;

    history.push({
      gen,
      bestFit: best.fit,
      avgFit,
      bestAcc: best.acc,
      avgAcc,
      bestIntel: best.intel,
      bestS: best.S,
      bestRegime: best.regime,
      mem: best.mem,
    });

    if (!hallOfFame || best.fit > hallOfFame.fit) {
      hallOfFame = {
        gen,
        fit: best.fit,
        acc: best.acc,
        intel: best.intel,
        S: best.S,
        genome: (best.org.genome || []).join('-'),
        aa: best.aa,
        recallPolicy: best.org.recallPolicy,
      };
    }

    console.log(
      `Gen ${String(gen).padStart(2)} | fit=${best.fit.toFixed(3)} avg=${avgFit.toFixed(3)} ` +
        `acc=${(best.acc * 100).toFixed(1)}% avgAcc=${(avgAcc * 100).toFixed(1)}% ` +
        `intel=${best.intel.toFixed(3)} S=${best.S.toFixed(3)} ${best.regime} ` +
        `mem=${best.mem} recallRules=${best.recallN}`
    );

    // Elites + children
    const elites = ranked.slice(0, eliteCount).map((r) => r.org);
    const next = [];
    for (let i = 0; i < elites.length; i++) {
      const e = elites[i];
      e.id = `intel-elite-${gen + 1}-${i}`;
      applyGenome(e, e.genome, genetic.codons);
      next.push(e);
    }
    while (next.length < popSize) {
      const parent = elites[Math.floor(Math.random() * elites.length)];
      const child = cloneChild(parent, `intel-g${gen + 1}-${next.length}`, genetic.codons);
      // partial re-teach (inheritance of codon table + fresh curriculum dose)
      teach(child, curriculum.slice(0, 40));
      next.push(child);
    }
    population = next;
  }

  const final = population
    .map((org) => ({
      id: org.id,
      fitness: totalFitness(org, bioTargetS),
      cognitiveAccuracy: org.cognitiveAccuracy,
      intelligenceScore: org.intelligenceScore,
      S: org.S,
      regime: org.regime,
      D_eff: org.params.D_eff,
      delta_psi: org.params.delta_psi,
      workingMemory: (org.workingMemory || []).length,
      recallPolicy: org.recallPolicy,
      genome: (org.genome || []).join('-'),
      aaChain: org.genomeGuidance?.aaChain || '',
      codonTableSize: Object.keys(org._codonTable || {}).length,
      lastTasks: (org.lastCognitiveResults || []).slice(0, 8),
    }))
    .sort((a, b) => b.fitness - a.fitness);

  const outDir = path.join(STATE_DIR, 'intelligence');
  if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });
  const outPath = path.join(outDir, `intel-evo-${Date.now()}.json`);
  const report = {
    goal: 'FSOT-native intelligence evolution',
    bioTargetS,
    leanRoot: findLeanRoot(),
    genetic: genetic.loaded,
    generations,
    popSize,
    history,
    hallOfFame,
    finalRanked: final,
    champion: final[0],
  };
  fs.writeFileSync(outPath, JSON.stringify(report, null, 2));

  const champ = population.find((o) => o.id === final[0].id) || population[0];
  champ.id = 'fsot-intelligence-champion';
  const champPath = champ.saveState();

  console.log('\n=== Intelligence Champion ===');
  console.log({
    accuracy: `${(final[0].cognitiveAccuracy * 100).toFixed(1)}%`,
    intelligenceScore: final[0].intelligenceScore,
    fitness: final[0].fitness,
    S: final[0].S,
    regime: final[0].regime,
    codonTableSize: final[0].codonTableSize,
    memory: final[0].workingMemory,
    aaChain: final[0].aaChain,
  });
  console.log('\nHall of fame:', hallOfFame);
  console.log(`\nReport → ${outPath}`);
  console.log(`Champion → ${champPath}`);
  return report;
}

if (require.main === module) {
  const gens = parseInt(process.argv[2] || '16', 10);
  const pop = parseInt(process.argv[3] || '10', 10);
  const ticks = parseInt(process.argv[4] || '50', 10);
  runIntelligenceEvolution({ generations: gens, popSize: pop, ticksPerGen: ticks });
}

module.exports = { runIntelligenceEvolution, totalFitness };
