/**
 * Deep FSOT Lean / verification ingest
 * Pulls domain ledger, README claims, Scalar constants, and optional
 * Desktop FSOT-2.1-Lean-main artifacts into seed objects.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { DOMAIN_PARAMS, computeForDomain, CONSTANTS } = require('./fsot_scalar');

const LEAN_CANDIDATES = [
  path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT-2.1-Lean-main'),
  path.join('C:', 'Users', 'damia', 'Desktop', 'FSOT-2.1-Lean'),
  path.join('D:', 'FSOT-2.1-Lean-PublicData'),
];

function findLeanRoot() {
  for (const p of LEAN_CANDIDATES) {
    if (fs.existsSync(p)) return p;
  }
  return null;
}

function safeRead(file, maxChars = 12000) {
  try {
    if (!fs.existsSync(file)) return null;
    return fs.readFileSync(file, 'utf8').slice(0, maxChars);
  } catch {
    return null;
  }
}

function domainSeeds() {
  return Object.keys(DOMAIN_PARAMS).map((name) => {
    const r = computeForDomain(name);
    const p = DOMAIN_PARAMS[name];
    return {
      text:
        `Lean domain ${name}: D_eff=${p.D_eff} δψ=${p.delta_psi} observed=${p.observed} ` +
        `hits=${p.recent_hits} S=${r.S.toFixed(6)} raw_S=${r.raw_S.toFixed(6)} ` +
        `regime=${r.regime} T1=${r.term1.toFixed(4)} T2=${r.term2.toFixed(4)} T3=${r.term3.toExponential(2)}`,
      type: 'fsot_domain',
      domain: name,
      S: r.S,
      regime: r.regime,
      strength: 0.92,
    };
  });
}

function constantSeeds() {
  return [
    {
      text:
        `FSOT Scalar.lean constants: k=${CONSTANTS.k} phi=${CONSTANTS.phi} ` +
        `psi_con=${CONSTANTS.psi_con} eta_eff=${CONSTANTS.eta_eff} ` +
        `coherence_efficiency=${CONSTANTS.coherence_efficiency} ` +
        `consciousness_factor=${CONSTANTS.consciousness_factor} ` +
        `phase_variance=${CONSTANTS.phase_variance} beta=${CONSTANTS.beta}`,
      type: 'fsot_math',
      strength: 0.97,
    },
    {
      text:
        `FSOT raw_S = term1 + term2 + term3; S = k * raw_S; ` +
        `term1 = base * perceived_adjust * quirk_mod; ` +
        `quirk_mod = exp(cf*pv)*cos(δψ+pv) when observed else 1; ` +
        `biological D_eff=12 δψ=0.05 emergence-positive guidance domain`,
      type: 'fsot_math',
      strength: 0.95,
    },
  ];
}

function fileSnippetSeeds(leanRoot) {
  const seeds = [];
  if (!leanRoot) return seeds;

  const files = [
    { rel: 'README.md', type: 'fsot_verification', strength: 0.9 },
    { rel: 'REPRODUCE.md', type: 'fsot_verification', strength: 0.88 },
    { rel: 'data/FSOT_VERIFIED_SCOPE.yaml', type: 'fsot_verification', strength: 0.94 },
    { rel: 'FSOT/Scalar.lean', type: 'fsot_math', strength: 0.93 },
    { rel: path.join('FSOT', 'Formal', 'Genomic.lean'), type: 'fsot_biology', strength: 0.95 },
    { rel: path.join('FSOT', 'Formal', 'CodonPriors.lean'), type: 'fsot_codon', strength: 0.94 },
    { rel: path.join('FSOT', 'Formal', 'ProteinPriors.lean'), type: 'fsot_protein', strength: 0.94 },
    { rel: path.join('FSOT', 'Formal', 'Domains.lean'), type: 'fsot_math', strength: 0.92 },
  ];

  for (const f of files) {
    const full = path.join(leanRoot, f.rel);
    const text = safeRead(full, 4000);
    if (!text) continue;
    // Chunk into digestible seeds
    const chunks = text
      .split(/\n{2,}/)
      .map((c) => c.replace(/\s+/g, ' ').trim())
      .filter((c) => c.length > 40 && c.length < 500)
      .slice(0, 12);
    for (const chunk of chunks) {
      seeds.push({
        text: `Lean:${f.rel} | ${chunk}`,
        type: f.type,
        strength: f.strength,
        domain: f.type.includes('bio') || f.type.includes('codon') || f.type.includes('protein')
          ? 'biological'
          : undefined,
      });
    }
  }

  seeds.push({
    text: `Lean root active: ${leanRoot}`,
    type: 'fsot_verification',
    strength: 0.7,
  });

  return seeds;
}

function allLeanSeeds() {
  const leanRoot = findLeanRoot();
  return [...constantSeeds(), ...domainSeeds(), ...fileSnippetSeeds(leanRoot)];
}

module.exports = {
  findLeanRoot,
  domainSeeds,
  constantSeeds,
  fileSnippetSeeds,
  allLeanSeeds,
};
