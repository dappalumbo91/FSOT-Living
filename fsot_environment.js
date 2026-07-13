/**
 * FSOT Autonomous Environment
 * You set the habitat once. The organism lives — no human feeding/exams/saves required.
 * Data river: Genetics, PublicData, Lean seeds, local labs — chewed continuously.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { allLeanSeeds } = require('./fsot_lean_ingest');
const { loadGeneticBundle, geneticSeeds } = require('./fsot_genetic_data');

/** Habitat parameters — the only human design surface. */
const DEFAULT_ENV = {
  // population dynamics
  mode: 'pack', // pack = multi-organism selection like natural evolution
  population: 6,
  eliteCount: 2,
  generationIntervalTicks: 180,
  genomeMutationRate: 0.1,

  // time
  tickMs: 70,
  maxTicks: 0, // 0 = endless

  // autonomous cognition / selection pressure
  examIntervalTicks: 90,
  examRounds: 14,
  saveIntervalTicks: 500,

  // information ecology
  feedIntervalTicks: 12,
  feedsPerPulse: 2,
  dataRiverMaxSeeds: 8000,
  stressAmplitude: 0.18,

  // survival / selection weights (environment law)
  weights: {
    cognitive: 2.0,
    intelligence: 1.4,
    bioAlign: 0.7,
    emergence: 0.3,
    pressurePenalty: 0.4,
    integration: 0.35,
  },

  // roots to mine (auto-skip missing)
  dataRoots: [
    path.join('C:', 'Users', 'damia', 'Desktop', 'Genetics'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'Genetics', 'data'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT-2.1-Lean-main'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT-2.1-Lean'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'Knowledge base'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT SMILES Lab'),
    path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT NeuroLab'),
    path.join('D:', 'FSOT-2.1-Lean-PublicData'),
    path.join('G:', 'FSOT-PublicData'),
  ],
};

function readTextSafe(file, max = 6000) {
  try {
    if (!fs.existsSync(file) || !fs.statSync(file).isFile()) return null;
    const st = fs.statSync(file);
    if (st.size > 2_000_000) return null; // skip huge binaries
    return fs.readFileSync(file, 'utf8').slice(0, max);
  } catch {
    return null;
  }
}

function walkFiles(root, exts, limit, depth = 0, out = []) {
  if (out.length >= limit || depth > 4) return out;
  let entries;
  try {
    entries = fs.readdirSync(root, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const ent of entries) {
    if (out.length >= limit) break;
    const full = path.join(root, ent.name);
    if (ent.name.startsWith('.') || ent.name === 'node_modules' || ent.name === '.venv' || ent.name === 'target') {
      continue;
    }
    if (ent.isDirectory()) {
      walkFiles(full, exts, limit, depth + 1, out);
    } else if (exts.some((e) => ent.name.toLowerCase().endsWith(e))) {
      out.push(full);
    }
  }
  return out;
}

function chunkToSeeds(text, source, type, strength = 0.75) {
  const seeds = [];
  const parts = text
    .split(/\n+/)
    .map((l) => l.trim())
    .filter((l) => l.length > 30 && l.length < 400);
  // sample up to 40 lines per file
  const step = Math.max(1, Math.floor(parts.length / 40));
  for (let i = 0; i < parts.length && seeds.length < 40; i += step) {
    seeds.push({
      text: `[${source}] ${parts[i]}`,
      type,
      strength,
      domain: type.includes('bio') || type.includes('codon') ? 'biological' : undefined,
    });
  }
  return seeds;
}

/**
 * Build the autonomous data river once at environment boot.
 */
function buildDataRiver(env = DEFAULT_ENV) {
  const river = [];
  const meta = { sources: [], lean: 0, genetic: 0, files: 0, rootsOk: [] };

  // Structured FSOT math + domains + genetics
  try {
    const lean = allLeanSeeds();
    river.push(...lean);
    meta.lean = lean.length;
    meta.sources.push(`lean_seeds:${lean.length}`);
  } catch (e) {
    meta.sources.push(`lean_error:${e.message}`);
  }

  try {
    const genetic = loadGeneticBundle();
    const gSeeds = geneticSeeds(genetic);
    river.push(...gSeeds);
    meta.genetic = gSeeds.length;
    meta.sources.push(
      `genetic:codons=${genetic.loaded.codons},aa=${genetic.loaded.amino},genome=${genetic.loaded.genome},fasta=${genetic.loaded.fasta}`
    );
  } catch (e) {
    meta.sources.push(`genetic_error:${e.message}`);
  }

  // Mine local/external text/json/md/csv/fasta/yaml
  const exts = ['.md', '.txt', '.json', '.jsonl', '.csv', '.tsv', '.fasta', '.fa', '.yaml', '.yml', '.lean'];
  for (const root of env.dataRoots) {
    if (!fs.existsSync(root)) continue;
    meta.rootsOk.push(root);
    const files = walkFiles(root, exts, 80);
    for (const file of files) {
      const text = readTextSafe(file);
      if (!text) continue;
      const base = path.basename(file);
      let type = 'fsot_env';
      if (/\.fa(sta)?$/i.test(base)) type = 'fsot_fasta';
      else if (/codon|amino|protein|gene|genome|dna/i.test(base)) type = 'fsot_biology';
      else if (/lean|scalar|domain/i.test(base)) type = 'fsot_math';
      else if (/verify|certificate|scope/i.test(base)) type = 'fsot_verification';
      const seeds = chunkToSeeds(text, base, type, 0.72 + Math.random() * 0.15);
      river.push(...seeds);
      meta.files++;
    }
  }

  // Cap river size (shuffle sample)
  if (river.length > env.dataRiverMaxSeeds) {
    for (let i = river.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [river[i], river[j]] = [river[j], river[i]];
    }
    river.length = env.dataRiverMaxSeeds;
  }

  meta.total = river.length;
  return { river, meta };
}

/** Cursor over the river — continuous autonomous exposure. */
class DataRiverCursor {
  constructor(river) {
    this.river = river.length ? river : [{ text: 'FSOT void habitat', type: 'fsot_env', strength: 0.5 }];
    this.i = 0;
  }
  next(n = 1) {
    const out = [];
    for (let k = 0; k < n; k++) {
      out.push(this.river[this.i % this.river.length]);
      this.i++;
    }
    return out;
  }
  get position() {
    return this.i;
  }
  get size() {
    return this.river.length;
  }
}

function loadEnvOverrides(filePath) {
  try {
    if (filePath && fs.existsSync(filePath)) {
      const raw = JSON.parse(fs.readFileSync(filePath, 'utf8'));
      return { ...DEFAULT_ENV, ...raw, weights: { ...DEFAULT_ENV.weights, ...(raw.weights || {}) } };
    }
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_ENV };
}

module.exports = {
  DEFAULT_ENV,
  buildDataRiver,
  DataRiverCursor,
  loadEnvOverrides,
};
