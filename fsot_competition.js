/**
 * Multi-organism competition — biological guidance via genomes + FSOT scalar fitness.
 * Population evolves genomes and structural policies under selection.
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

function applyGenomeGuidance(entity, sequence, codonMap) {
  const g = genomeToGuidance(sequence, codonMap);
  entity.genome = sequence.slice();
  entity.genomeGuidance = g;
  // Soft-hard blend: genome is biological guidance system
  entity.params.D_eff = entity.params.D_eff * 0.35 + g.D_eff * 0.65;
  entity.params.delta_psi = entity.params.delta_psi * 0.35 + g.delta_psi * 0.65;
  entity.params.emergencePreference =
    entity.params.emergencePreference * 0.4 + g.emergencePreference * 0.6;
  entity.domains.domainCoupling = Math.min(
    1.55,
    entity.domains.domainCoupling * 0.5 + g.domainCoupling * 0.5
  );
  // Living systems track biological domain emergence
  entity.targetS = entity.targetS * 0.7 + 0.35 * 0.3;
  return g;
}

function organismFitness(entity, bioTargetS) {
  // Prefer emergence near biological domain S, low pressure, high integration
  const sAlign = 1 - Math.min(1.5, Math.abs(entity.S - bioTargetS));
  const emergenceBonus = entity.regime === 'emergence' ? 0.35 : entity.regime === 'neutral' ? 0.1 : -0.25;
  return (
    entity.S * 0.9 +
    sAlign * 0.8 +
    emergenceBonus -
    entity.vitalityPressure * 0.7 +
    entity.coherence * 0.4 +
    entity.informationIntegration * 0.5 +
    entity.adaptiveEfficiency * 0.3 +
    (entity.genome?.length || 0) * 0.002
  );
}

function cloneOrganism(parent, id, codonMap, mutRate = 0.1) {
  const child = new FSOTLivingScalar(id, false);
  child.params = JSON.parse(JSON.stringify(parent.params));
  child.domains = JSON.parse(JSON.stringify(parent.domains));
  child.selfModificationPolicy = JSON.parse(JSON.stringify(parent.selfModificationPolicy));
  child.signalGenerationLogic = JSON.parse(JSON.stringify(parent.signalGenerationLogic));
  child.domainRegulationLogic = JSON.parse(JSON.stringify(parent.domainRegulationLogic));
  child.genome = mutateGenome(parent.genome || [], mutRate);
  applyGenomeGuidance(child, child.genome, codonMap);
  // slight param noise
  child.params.mutationRate = Math.max(
    0.01,
    Math.min(0.05, parent.params.mutationRate + (Math.random() - 0.5) * 0.01)
  );
  return child;
}

function runCompetition(options = {}) {
  const popSize = options.popSize ?? 10;
  const generations = options.generations ?? 20;
  const ticksPerGen = options.ticksPerGen ?? 120;
  const eliteCount = options.eliteCount ?? 3;
  const seedIngest = options.seedIngest !== false;

  const genetic = loadGeneticBundle();
  const leanSeeds = allLeanSeeds();
  const geneSeeds = geneticSeeds(genetic);
  const bioRef = computeForDomain('biological');
  const bioTargetS = bioRef.S;

  const baseGenome =
    genetic.organism?.sequence ||
    ['ATG', 'CCG', 'AAC', 'GGT', 'AAT', 'AGC', 'AGT', 'GGT', 'CTG', 'TCG', 'TTA', 'TAA'];

  console.log('=== FSOT Multi-Organism Competition (Biological Guidance) ===\n');
  console.log(`Genetics root: ${genetic.root}`);
  console.log(
    `Loaded: codons=${genetic.loaded.codons} AA=${genetic.loaded.amino} ` +
      `genome_codons=${genetic.loaded.genome} fasta=${genetic.loaded.fasta}`
  );
  console.log(`Lean root: ${findLeanRoot() || '(none)'}`);
  console.log(`Biological reference S=${bioTargetS.toFixed(6)} regime=${bioRef.regime}`);
  console.log(
    `Population=${popSize} generations=${generations} ticks/gen=${ticksPerGen}\n`
  );

  let population = [];
  for (let i = 0; i < popSize; i++) {
    const org = new FSOTLivingScalar(`comp-org-${i}`, false);
    const genome =
      i === 0
        ? baseGenome.slice()
        : mutateGenome(baseGenome, 0.15 + i * 0.02);
    applyGenomeGuidance(org, genome, genetic.codons);
    population.push(org);
  }

  // Shared knowledge bath
  if (seedIngest) {
    const bath = [...leanSeeds.slice(0, 40), ...geneSeeds.slice(0, 80)];
    console.log(`Ingesting knowledge bath: ${bath.length} seeds into each organism...\n`);
    for (const org of population) {
      for (const seed of bath) {
        org.ingestInformation(seed);
      }
    }
  }

  const history = [];

  for (let gen = 0; gen < generations; gen++) {
    // Live + occasional genetic re-feed
    for (const org of population) {
      for (let t = 0; t < ticksPerGen; t++) {
        if (t % 25 === 0 && geneSeeds.length) {
          org.ingestInformation(geneSeeds[t % geneSeeds.length]);
        }
        if (t % 40 === 0 && leanSeeds.length) {
          org.ingestInformation(leanSeeds[t % leanSeeds.length]);
        }
        // re-assert genome guidance periodically (biological constraint)
        if (t % 30 === 0) {
          applyGenomeGuidance(org, org.genome, genetic.codons);
        }
        org.tickOnce();
      }
    }

    // Score
    const ranked = population
      .map((org) => ({
        org,
        fitness: organismFitness(org, bioTargetS),
        S: org.S,
        regime: org.regime,
        pressure: org.vitalityPressure,
        integration: org.informationIntegration,
        genomeLen: org.genome?.length || 0,
        aa: org.genomeGuidance?.aaChain?.slice(0, 24) || '',
      }))
      .sort((a, b) => b.fitness - a.fitness);

    const best = ranked[0];
    const avgFit =
      ranked.reduce((s, r) => s + r.fitness, 0) / ranked.length;
    const avgS = ranked.reduce((s, r) => s + r.S, 0) / ranked.length;

    history.push({
      gen,
      bestFitness: best.fitness,
      avgFitness: avgFit,
      bestS: best.S,
      avgS,
      bestRegime: best.regime,
      bestGenomeLen: best.genomeLen,
      bestAA: best.aa,
    });

    console.log(
      `Gen ${String(gen).padStart(2)} | bestFit=${best.fitness.toFixed(3)} ` +
        `avgFit=${avgFit.toFixed(3)} bestS=${best.S.toFixed(4)} (${best.regime}) ` +
        `avgS=${avgS.toFixed(4)} genome=${best.genomeLen} aa=${best.aa || '-'} ` +
        `P=${best.pressure.toFixed(2)} int=${best.integration.toFixed(2)}`
    );

    // Selection + reproduction
    const elites = ranked.slice(0, eliteCount).map((r) => r.org);
    const next = [];
    // keep elites (re-apply guidance)
    for (let i = 0; i < elites.length; i++) {
      const e = elites[i];
      e.id = `comp-elite-${gen + 1}-${i}`;
      applyGenomeGuidance(e, e.genome, genetic.codons);
      next.push(e);
    }
    while (next.length < popSize) {
      const parent = elites[Math.floor(Math.random() * elites.length)];
      const child = cloneOrganism(
        parent,
        `comp-g${gen + 1}-c${next.length}`,
        genetic.codons,
        0.08 + Math.random() * 0.08
      );
      next.push(child);
    }
    population = next;
  }

  // Final ranking
  const finalRanked = population
    .map((org) => ({
      id: org.id,
      fitness: organismFitness(org, bioTargetS),
      S: org.S,
      raw_S: org.raw_S,
      regime: org.regime,
      D_eff: org.params.D_eff,
      delta_psi: org.params.delta_psi,
      genome: (org.genome || []).join('-'),
      aaChain: org.genomeGuidance?.aaChain || '',
      policySize: org.selfModificationPolicy.length,
      signalSize: org.signalGenerationLogic.length,
      domainSize: org.domainRegulationLogic.length,
      acceptedMutations: org.acceptedMutations,
      rejectedMutations: org.rejectedMutations,
      integration: org.informationIntegration,
      coherence: org.coherence,
      pressure: org.vitalityPressure,
    }))
    .sort((a, b) => b.fitness - a.fitness);

  const outDir = path.join(STATE_DIR, 'competition');
  if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });
  const outPath = path.join(outDir, `competition-${Date.now()}.json`);
  const report = {
    bioTargetS,
    leanRoot: findLeanRoot(),
    geneticLoaded: genetic.loaded,
    generations,
    popSize,
    ticksPerGen,
    history,
    finalRanked,
    champion: finalRanked[0],
  };
  fs.writeFileSync(outPath, JSON.stringify(report, null, 2));

  // Persist champion full state
  const champ = population.find((o) => o.id === finalRanked[0].id) || population[0];
  champ.id = 'fsot-champion';
  const champPath = champ.saveState();

  console.log('\n=== Champion ===');
  console.log(finalRanked[0]);
  console.log(`\nReport → ${outPath}`);
  console.log(`Champion state → ${champPath}`);

  return report;
}

if (require.main === module) {
  const gens = parseInt(process.argv[2] || '15', 10);
  const pop = parseInt(process.argv[3] || '8', 10);
  const ticks = parseInt(process.argv[4] || '100', 10);
  runCompetition({ generations: gens, popSize: pop, ticksPerGen: ticks });
}

module.exports = { runCompetition, applyGenomeGuidance, organismFitness, cloneOrganism };
