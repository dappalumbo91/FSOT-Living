/**
 * FSOT Genetic Data Layer — biological guidance substrate
 * Loads codon maps, amino-acid trinary, organism genomic sequence,
 * and sample FASTA from Desktop\Genetics.
 */

'use strict';

const fs = require('fs');
const path = require('path');

const GENETICS_ROOT = path.join('C:', 'Users', 'damia', 'Desktop', 'Genetics');
const CODONS_FILE = path.join(GENETICS_ROOT, '64_codon_trinary_map.txt');
const AA_FILE = path.join(GENETICS_ROOT, '20_amino_acid_fsot_map.txt');
const GENOME_FILE = path.join(
  GENETICS_ROOT,
  'black_hole_outgas_ai',
  'organism_genomic_sequence.txt'
);
const DATA_DIR = path.join(GENETICS_ROOT, 'data');

// Standard genetic code (codon → amino acid / stop)
const GENETIC_CODE = {
  AAA: 'K', AAC: 'N', AAG: 'K', AAT: 'N', ACA: 'T', ACC: 'T', ACG: 'T', ACT: 'T',
  AGA: 'R', AGC: 'S', AGG: 'R', AGT: 'S', ATA: 'I', ATC: 'I', ATG: 'M', ATT: 'I',
  CAA: 'Q', CAC: 'H', CAG: 'Q', CAT: 'H', CCA: 'P', CCC: 'P', CCG: 'P', CCT: 'P',
  CGA: 'R', CGC: 'R', CGG: 'R', CGT: 'R', CTA: 'L', CTC: 'L', CTG: 'L', CTT: 'L',
  GAA: 'E', GAC: 'D', GAG: 'E', GAT: 'D', GCA: 'A', GCC: 'A', GCG: 'A', GCT: 'A',
  GGA: 'G', GGC: 'G', GGG: 'G', GGT: 'G', GTA: 'V', GTC: 'V', GTG: 'V', GTT: 'V',
  TAA: '*', TAC: 'Y', TAG: '*', TAT: 'Y', TCA: 'S', TCC: 'S', TCG: 'S', TCT: 'S',
  TGA: '*', TGC: 'C', TGG: 'W', TGT: 'C', TTA: 'L', TTC: 'F', TTG: 'L', TTT: 'F',
};

function parseCodonMap(text) {
  const map = {};
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(
      /^([ACGT]{3})\s+\|\s+\[([-\d,\s]+)\]\s+\|\s+\[([-\d,\s]+)\]/
    );
    if (!m) continue;
    const codon = m[1];
    const primary = m[2].split(',').map((x) => parseInt(x.trim(), 10));
    const secondary = m[3].split(',').map((x) => parseInt(x.trim(), 10));
    map[codon] = {
      codon,
      primary,
      secondary,
      aa: GENETIC_CODE[codon] || '?',
      stop: GENETIC_CODE[codon] === '*',
    };
  }
  return map;
}

function parseAminoMap(text) {
  const map = {};
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(
      /^\s*([A-Z])\s+\|\s+\[\s*([-\d]+),\s*([-\d]+),\s*([-\d]+)\]\s+\|\s+(\w[\w\s]*?)\s+\|\s+([\d.]+)\s+\|\s+([\d.]+)\s+\|\s+([\d.]+)/
    );
    if (!m) continue;
    map[m[1]] = {
      letter: m[1],
      trinary: [parseInt(m[2], 10), parseInt(m[3], 10), parseInt(m[4], 10)],
      name: m[5].trim(),
      hydrophobicity: parseFloat(m[6]),
      volume: parseFloat(m[7]),
      dipole: parseFloat(m[8]),
    };
  }
  return map;
}

function parseOrganismGenome(text) {
  const components = [];
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(/^(.+?)\s+\|\s+([ACGT]{3})\s+\|\s+(.+)$/);
    if (!m) continue;
    if (m[1].includes('---') || m[1].includes('ORGANISM')) continue;
    components.push({
      component: m[1].trim(),
      codon: m[2],
      note: m[3].trim(),
    });
  }
  const pure = text.match(/PURE GENOMIC TRANSLATION[\s\S]*?([ACGT]{3}(?:-[ACGT]{3})+)/);
  const sequence = pure
    ? pure[1].split('-')
    : components.map((c) => c.codon);
  return { components, sequence, pureString: sequence.join('') };
}

function loadFastaFiles(dir, limit = 8) {
  if (!fs.existsSync(dir)) return [];
  const files = fs
    .readdirSync(dir)
    .filter((f) => f.endsWith('.fasta') || f.endsWith('.fa'))
    .slice(0, limit);
  return files.map((f) => {
    const raw = fs.readFileSync(path.join(dir, f), 'utf8');
    const lines = raw.split(/\r?\n/).filter((l) => l && !l.startsWith('>'));
    const seq = lines.join('').replace(/\s/g, '').toUpperCase();
    const header = (raw.split(/\r?\n/).find((l) => l.startsWith('>')) || `>${f}`).slice(1);
    return { id: f, header, sequence: seq, length: seq.length };
  });
}

function loadGeneticBundle() {
  const bundle = {
    root: GENETICS_ROOT,
    codons: {},
    aminoAcids: {},
    organism: null,
    fasta: [],
    loaded: { codons: false, amino: false, genome: false, fasta: 0 },
  };

  if (fs.existsSync(CODONS_FILE)) {
    bundle.codons = parseCodonMap(fs.readFileSync(CODONS_FILE, 'utf8'));
    bundle.loaded.codons = Object.keys(bundle.codons).length;
  }
  if (fs.existsSync(AA_FILE)) {
    bundle.aminoAcids = parseAminoMap(fs.readFileSync(AA_FILE, 'utf8'));
    bundle.loaded.amino = Object.keys(bundle.aminoAcids).length;
  }
  if (fs.existsSync(GENOME_FILE)) {
    bundle.organism = parseOrganismGenome(fs.readFileSync(GENOME_FILE, 'utf8'));
    bundle.loaded.genome = bundle.organism.sequence.length;
  }
  bundle.fasta = loadFastaFiles(DATA_DIR);
  bundle.loaded.fasta = bundle.fasta.length;

  // Fallback minimal codon set if files missing
  if (bundle.loaded.codons === 0 || bundle.loaded.codons === false) {
    const bases = ['A', 'C', 'G', 'T'];
    for (const a of bases) {
      for (const b of bases) {
        for (const c of bases) {
          const codon = a + b + c;
          const primary = [a, b, c].map((x) => (x === 'A' || x === 'G' ? 1 : -1));
          const secondary = [a, b, c].map((x) =>
            x === 'A' ? 1 : x === 'T' ? -1 : 0
          );
          bundle.codons[codon] = {
            codon,
            primary,
            secondary,
            aa: GENETIC_CODE[codon] || '?',
            stop: GENETIC_CODE[codon] === '*',
          };
        }
      }
    }
    bundle.loaded.codons = 64;
  }

  return bundle;
}

/**
 * Convert codon sequence → guidance vector for FSOT params.
 * primary mean → delta_psi bias; secondary mean → emergence preference;
 * start/stop structure → domain coupling weight.
 */
function genomeToGuidance(sequence, codonMap) {
  const prim = [0, 0, 0];
  const sec = [0, 0, 0];
  let n = 0;
  let stops = 0;
  let starts = 0;
  const aaChain = [];

  for (const codon of sequence) {
    const c = codonMap[codon];
    if (!c) continue;
    n++;
    for (let i = 0; i < 3; i++) {
      prim[i] += c.primary[i];
      sec[i] += c.secondary[i];
    }
    if (c.stop) stops++;
    if (codon === 'ATG') starts++;
    if (c.aa && c.aa !== '*') aaChain.push(c.aa);
  }

  if (n === 0) {
    return {
      delta_psi: 0.7,
      D_eff: 12,
      emergencePreference: 0.62,
      domainCoupling: 0.82,
      aaChain: '',
      codonCount: 0,
    };
  }

  const pMean = prim.map((x) => x / n);
  const sMean = sec.map((x) => x / n);
  const pMag = Math.sqrt(pMean[0] ** 2 + pMean[1] ** 2 + pMean[2] ** 2) / Math.sqrt(3);
  const sMag = Math.sqrt(sMean[0] ** 2 + sMean[1] ** 2 + sMean[2] ** 2) / Math.sqrt(3);

  // Biological domain ledger: D_eff≈12, delta_psi≈0.05 baseline, emergence-positive
  const delta_psi = Math.max(0.05, Math.min(1.3, 0.35 + pMean[0] * 0.25 + pMean[1] * 0.15));
  const D_eff = Math.max(8, Math.min(20, 12 + sMean[2] * 3 + pMag * 2));
  const emergencePreference = Math.max(0.42, Math.min(0.82, 0.55 + sMag * 0.2));
  const domainCoupling = Math.max(0.4, Math.min(1.4, 0.75 + (starts - stops) * 0.05 + sMean[1] * 0.1));

  return {
    delta_psi,
    D_eff,
    emergencePreference,
    domainCoupling,
    aaChain: aaChain.join(''),
    codonCount: n,
    primaryMean: pMean,
    secondaryMean: sMean,
    starts,
    stops,
  };
}

function mutateGenome(sequence, rate = 0.08) {
  const bases = ['A', 'C', 'G', 'T'];
  const next = sequence.map((codon) => {
    if (Math.random() > rate) return codon;
    const arr = codon.split('');
    const i = Math.floor(Math.random() * 3);
    arr[i] = bases[Math.floor(Math.random() * 4)];
    return arr.join('');
  });
  // occasional insert/delete of a codon
  if (Math.random() < rate * 0.5 && next.length < 48) {
    const c =
      bases[Math.floor(Math.random() * 4)] +
      bases[Math.floor(Math.random() * 4)] +
      bases[Math.floor(Math.random() * 4)];
    next.splice(Math.floor(Math.random() * next.length), 0, c);
  } else if (Math.random() < rate * 0.3 && next.length > 8) {
    next.splice(Math.floor(Math.random() * next.length), 1);
  }
  return next;
}

/** Build ingest seeds from genetic bundle for living organism. */
function geneticSeeds(bundle) {
  const seeds = [];
  seeds.push({
    text:
      `FSOT genetic code: ${bundle.loaded.codons} codons dual-axis trinary map ` +
      `primary A/G=+1 C/T=-1 secondary A=+1 T=-1 G/C=0; 20 amino acids [Charge,Polarity,Volume]`,
    type: 'fsot_biology',
    strength: 0.96,
  });

  for (const [codon, c] of Object.entries(bundle.codons).slice(0, 64)) {
    seeds.push({
      text:
        `Codon ${codon} → AA ${c.aa} primary=[${c.primary.join(',')}] ` +
        `secondary=[${c.secondary.join(',')}]`,
      type: 'fsot_codon',
      strength: 0.8,
      domain: 'biological',
    });
  }

  for (const [aa, a] of Object.entries(bundle.aminoAcids)) {
    seeds.push({
      text:
        `Amino acid ${aa} (${a.name}) trinary=[${a.trinary.join(',')}] ` +
        `hydrophobicity=${a.hydrophobicity} volume=${a.volume} dipole=${a.dipole}`,
      type: 'fsot_protein',
      strength: 0.85,
      domain: 'molecular',
    });
  }

  if (bundle.organism) {
    seeds.push({
      text:
        `FSOT 23D organism genome: ${bundle.organism.sequence.join('-')} ` +
        `(${bundle.organism.sequence.length} codons) biological guidance sequence`,
      type: 'fsot_genome',
      strength: 0.97,
      domain: 'biological',
    });
    for (const comp of bundle.organism.components.slice(0, 12)) {
      seeds.push({
        text: `Organism component ${comp.component} codon ${comp.codon}: ${comp.note}`,
        type: 'fsot_genome',
        strength: 0.88,
        domain: 'biological',
      });
    }
  }

  for (const fa of bundle.fasta) {
    const preview = fa.sequence.slice(0, 60);
    seeds.push({
      text: `FASTA ${fa.id} len=${fa.length} header=${fa.header.slice(0, 40)} seq=${preview}...`,
      type: 'fsot_fasta',
      strength: 0.9,
      domain: 'biological',
    });
  }

  return seeds;
}

module.exports = {
  GENETICS_ROOT,
  GENETIC_CODE,
  loadGeneticBundle,
  genomeToGuidance,
  mutateGenome,
  geneticSeeds,
  parseCodonMap,
  parseAminoMap,
};
