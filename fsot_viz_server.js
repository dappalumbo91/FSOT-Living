/**
 * FSOT Autonomous Habitat Server
 * Human sets env.json once. Organisms live, feed, exam, select, save — no buttons required.
 * Dashboard is observation-only.
 * Open http://localhost:3847
 */

'use strict';

const http = require('http');
const fs = require('fs');
const path = require('path');
const { FSOTLivingScalar, STATE_DIR } = require('./fsot_living_core');
const { loadGeneticBundle, genomeToGuidance, mutateGenome } = require('./fsot_genetic_data');
const { computeForDomain, CONSTANTS } = require('./fsot_scalar');
const { teach, runCognitiveBattery, queryOrganism } = require('./fsot_cognition');
const {
  loadEnvOverrides,
  buildDataRiver,
  DataRiverCursor,
} = require('./fsot_environment');

const PORT = parseInt(process.env.FSOT_VIZ_PORT || '3847', 10);
const ROOT = __dirname;
const ENV_PATH = path.join(ROOT, 'env.json');

// ---------- Boot habitat ----------
const env = loadEnvOverrides(ENV_PATH);
const genetic = loadGeneticBundle();
const bio = computeForDomain('biological');
const { river, meta: riverMeta } = buildDataRiver(env);
const cursor = new DataRiverCursor(river);

const baseGenome =
  genetic.organism?.sequence ||
  ['ATG', 'CCG', 'AAC', 'GGT', 'AAT', 'AGC', 'AGT', 'CTG', 'TCG', 'TAA'];

function applyGenome(org, sequence) {
  const g = genomeToGuidance(sequence, genetic.codons);
  org.genome = sequence.slice();
  org.genomeGuidance = g;
  org.params.D_eff = org.params.D_eff * 0.35 + g.D_eff * 0.65;
  org.params.delta_psi = org.params.delta_psi * 0.35 + g.delta_psi * 0.65;
  org.params.emergencePreference =
    org.params.emergencePreference * 0.4 + g.emergencePreference * 0.6;
  org.domains.domainCoupling = Math.min(
    1.55,
    org.domains.domainCoupling * 0.5 + g.domainCoupling * 0.5
  );
  org.targetS = bio.S;
  return g;
}

function makeCritter(id, genome) {
  const org = new FSOTLivingScalar(id, false);
  org.longTermKnowledge = [];
  org.domainMemory = {};
  applyGenome(org, genome);
  // Birth: structured river bites so knowledge can compound
  const birthSeeds = [];
  for (let i = 0; i < 50 && birthSeeds.length < 36; i++) {
    const s = cursor.next(1)[0];
    if (s) birthSeeds.push(s);
  }
  teach(org, birthSeeds);
  return org;
}

function fitness(org) {
  const w = env.weights;
  const sAlign = 1 - Math.min(1.2, Math.abs(org.S - bio.S));
  return (
    (org.cognitiveAccuracy || 0) * w.cognitive +
    (org.intelligenceScore || 0) * w.intelligence +
    sAlign * w.bioAlign +
    (org.regime === 'emergence' ? w.emergence : -w.emergence * 0.5) -
    (org.vitalityPressure || 0) * w.pressurePenalty +
    (org.informationIntegration || 0) * w.integration
  );
}

const world = {
  autonomous: true,
  running: true,
  env,
  tick: 0,
  generation: 0,
  events: [],
  pack: [],
  hero: null,
  lastBattery: null,
  riverMeta,
  riverSize: cursor.size,
  riverPos: 0,
  lineage: [],
  stats: {
    fed: 0,
    exams: 0,
    births: 0,
    autosaves: 0,
    generations: 0,
    extinctionsAvoided: 0,
  },
};

// Spawn initial population into the habitat
const pop = Math.max(2, env.population || 6);
world.pack = [];
for (let i = 0; i < pop; i++) {
  const g = i === 0 ? baseGenome.slice() : mutateGenome(baseGenome, 0.08 + i * 0.03);
  world.pack.push(makeCritter(`org-g0-${i}`, g));
}
world.hero = world.pack[0];

function pushEvent(msg) {
  world.events.unshift({ t: world.tick, msg, at: Date.now() });
  if (world.events.length > 50) world.events.pop();
}

pushEvent('Autonomous habitat online — observation only');
pushEvent(`Data river: ${cursor.size} seeds · roots ${riverMeta.rootsOk.length}`);
pushEvent(`Population ${pop} · mode ${env.mode} · gen every ${env.generationIntervalTicks} ticks`);

function snapshotOrg(org) {
  return {
    id: org.id,
    tick: org.tick,
    S: org.S,
    raw_S: org.raw_S,
    regime: org.regime,
    vitality: org.vitality,
    coherence: org.coherence,
    pressure: org.vitalityPressure,
    reward: org.rewardSignal,
    health: org.homeostaticHealth,
    integration: org.informationIntegration,
    adaptive: org.adaptiveEfficiency,
    cognitive: org.cognitiveAccuracy || 0,
    intelligence: org.intelligenceScore || 0,
    fitness: fitness(org),
    D_eff: org.params.D_eff,
    delta_psi: org.params.delta_psi,
    observed: org.observed,
    domains: org.domains,
    genome: (org.genome || []).join('-'),
    aaChain: org.genomeGuidance?.aaChain || '',
    memSize: (org.workingMemory || []).length,
    recallRules: (org.recallPolicy || []).length,
    policySizes: {
      mod: org.selfModificationPolicy.length,
      signal: org.signalGenerationLogic.length,
      domain: org.domainRegulationLogic.length,
      recall: (org.recallPolicy || []).length,
    },
    codonKnown: Object.keys(org._codonTable || {}).length,
    acceptedMut: org.acceptedMutations,
    rejectedMut: org.rejectedMutations,
    term1: org.term1,
    term2: org.term2,
    term3: org.term3,
  };
}

function worldSnapshot() {
  const pack = world.pack.map(snapshotOrg).sort((a, b) => b.fitness - a.fitness);
  return {
    autonomous: true,
    running: world.running,
    mode: env.mode,
    worldTick: world.tick,
    generation: world.generation,
    bioTargetS: bio.S,
    k: CONSTANTS.k,
    genetic: genetic.loaded,
    env: {
      population: env.population,
      tickMs: env.tickMs,
      feedIntervalTicks: env.feedIntervalTicks,
      examIntervalTicks: env.examIntervalTicks,
      generationIntervalTicks: env.generationIntervalTicks,
      saveIntervalTicks: env.saveIntervalTicks,
      stressAmplitude: env.stressAmplitude,
    },
    river: {
      size: world.riverSize,
      position: world.riverPos,
      sources: riverMeta.sources,
      rootsOk: riverMeta.rootsOk,
      filesMined: riverMeta.files,
    },
    hero: snapshotOrg(world.hero),
    pack,
    events: world.events.slice(0, 25),
    stats: world.stats,
    lastBattery: world.lastBattery,
    lineage: world.lineage.slice(-12),
  };
}

function autonomousExam(org) {
  const bat = runCognitiveBattery(org, {
    rounds: env.examRounds,
    codonMap: genetic.codons,
    bioTargetS: bio.S,
  });
  world.lastBattery = {
    accuracy: bat.accuracy,
    intelligenceScore: bat.intelligenceScore,
    n: bat.n,
    id: org.id,
  };
  world.stats.exams++;
  return bat;
}

function autonomousGeneration() {
  // Rank by fitness law of the habitat
  world.pack.sort((a, b) => fitness(b) - fitness(a));
  const elites = world.pack.slice(0, env.eliteCount);
  world.hero = elites[0];

  // Compounding evolutionary pathways — heritable reasoning signature
  const pathway = {
    gen: world.generation,
    champion: world.hero.id,
    S: world.hero.S,
    acc: world.hero.cognitiveAccuracy || 0,
    intel: world.hero.intelligenceScore || 0,
    fit: fitness(world.hero),
    aa: world.hero.genomeGuidance?.aaChain?.slice(0, 28) || '',
    genome: (world.hero.genome || []).join('-'),
    codonTable: Object.keys(world.hero._codonTable || {}).length,
    mem: (world.hero.workingMemory || []).length,
    recall: (world.hero.recallPolicy || []).map((r) => `${r.feature}:${Number(r.weight).toFixed(2)}`),
    structures: {
      mod: world.hero.selfModificationPolicy.length,
      signal: world.hero.signalGenerationLogic.length,
      domain: world.hero.domainRegulationLogic.length,
      recall: (world.hero.recallPolicy || []).length,
    },
    D_eff: world.hero.params.D_eff,
    delta_psi: world.hero.params.delta_psi,
  };
  world.lineage.push(pathway);
  // Persist pathway archive (compounding history of minds)
  try {
    const pdir = path.join(STATE_DIR, 'habitat', 'pathways');
    if (!fs.existsSync(pdir)) fs.mkdirSync(pdir, { recursive: true });
    fs.writeFileSync(
      path.join(pdir, `pathway-g${world.generation}.json`),
      JSON.stringify(pathway, null, 2)
    );
    fs.writeFileSync(
      path.join(STATE_DIR, 'habitat', 'lineage.json'),
      JSON.stringify(world.lineage, null, 2)
    );
  } catch (_) {
    /* non-fatal */
  }

  const next = [];
  for (let i = 0; i < elites.length; i++) {
    const e = elites[i];
    e.id = `elite-g${world.generation + 1}-${i}`;
    applyGenome(e, e.genome);
    next.push(e);
  }
  while (next.length < env.population) {
    const parent = elites[Math.floor(Math.random() * elites.length)];
    const child = makeCritter(
      `org-g${world.generation + 1}-${next.length}`,
      mutateGenome(parent.genome || baseGenome, env.genomeMutationRate)
    );
    child.recallPolicy = JSON.parse(JSON.stringify(parent.recallPolicy || []));
    child._codonTable = { ...(parent._codonTable || {}) };
    child.domainMemory = JSON.parse(JSON.stringify(parent.domainMemory || {}));
    child.longTermKnowledge = (parent.longTermKnowledge || []).slice(-120);
    // inherit partial working memory (cultural transmission)
    child.workingMemory = (parent.workingMemory || []).slice(-25);
    next.push(child);
    world.stats.births++;
  }
  world.pack = next;
  world.generation++;
  world.stats.generations++;
  pushEvent(
    `Gen ${world.generation} · champion ${world.hero.id} S=${world.hero.S.toFixed(3)} ` +
      `acc=${((world.hero.cognitiveAccuracy || 0) * 100).toFixed(1)}% fit=${fitness(world.hero).toFixed(2)}`
  );
}

function autonomousSave() {
  try {
    if (!fs.existsSync(STATE_DIR)) fs.mkdirSync(STATE_DIR, { recursive: true });
    const dir = path.join(STATE_DIR, 'habitat');
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

    if (world.hero) {
      // Trim heavy arrays before disk write (was a crash vector on long runs)
      if (Array.isArray(world.hero.history) && world.hero.history.length > 80) {
        world.hero.history = world.hero.history.slice(-80);
      }
      if (Array.isArray(world.hero.longTermKnowledge) && world.hero.longTermKnowledge.length > 200) {
        world.hero.longTermKnowledge = world.hero.longTermKnowledge.slice(-200);
      }
      world.hero.id = 'fsot-habitat-champion';
      world.hero.saveState();
    }

    const report = {
      tick: world.tick,
      generation: world.generation,
      stats: world.stats,
      lastBattery: world.lastBattery,
      champion: world.hero ? snapshotOrg(world.hero) : null,
      pack: world.pack.map(snapshotOrg),
      lineage: world.lineage.slice(-80),
      riverPos: world.riverPos,
      savedAt: new Date().toISOString(),
    };
    const p = path.join(dir, `habitat-${Date.now()}.json`);
    fs.writeFileSync(p, JSON.stringify(report, null, 2));
    fs.writeFileSync(path.join(dir, 'latest.json'), JSON.stringify(report, null, 2));
    // prune old habitat-*.json keep last 15
    const old = fs
      .readdirSync(dir)
      .filter((f) => f.startsWith('habitat-') && f.endsWith('.json'))
      .sort();
    while (old.length > 15) {
      try {
        fs.unlinkSync(path.join(dir, old.shift()));
      } catch (_) {
        break;
      }
    }
    world.stats.autosaves++;
    pushEvent(`Autosave · ${path.basename(p)}`);
  } catch (e) {
    logCrash('autonomousSave', e);
    pushEvent(`Autosave failed: ${e.message}`);
  }
}

function stepWorld() {
  if (!world.running) return;
  if (env.maxTicks > 0 && world.tick >= env.maxTicks) {
    world.running = false;
    pushEvent('maxTicks reached — habitat paused by env law');
    return;
  }

  world.tick++;
  const stress =
    Math.sin(world.tick / 27) * env.stressAmplitude +
    Math.sin(world.tick / 91) * env.stressAmplitude * 0.4;

  for (const org of world.pack) {
    // Autonomous information ecology — prefer structured learning bites
    if (world.tick % env.feedIntervalTicks === 0) {
      const bites = [];
      for (let i = 0; i < env.feedsPerPulse * 3 && bites.length < env.feedsPerPulse; i++) {
        const s = cursor.next(1)[0];
        if (!s) break;
        if (
          bites.length < env.feedsPerPulse - 1 &&
          (s.type === 'fsot_codon' ||
            s.type === 'fsot_domain' ||
            s.type === 'fsot_biology' ||
            /Codon\s+[ACGT]{3}/i.test(s.text || ''))
        ) {
          bites.push(s);
        } else if (bites.length >= env.feedsPerPulse - 1 || i > env.feedsPerPulse) {
          bites.push(s);
        }
      }
      while (bites.length < env.feedsPerPulse) bites.push(...cursor.next(1));
      teach(org, bites);
      world.stats.fed += bites.length;
      world.riverPos = cursor.position;
    }

    // Biological constraint re-asserted by environment
    if (world.tick % 28 === 0 && org.genome) {
      applyGenome(org, org.genome);
    }

    org.tickOnce(stress);
  }

  // Keep hero = current fittest for display
  world.pack.sort((a, b) => fitness(b) - fitness(a));
  world.hero = world.pack[0];

  // Autonomous cognitive exams (selection pressure, not a human quiz)
  if (world.tick % env.examIntervalTicks === 0) {
    for (const org of world.pack) {
      autonomousExam(org);
    }
    pushEvent(
      `Natural exam pulse · best acc ${((world.hero.cognitiveAccuracy || 0) * 100).toFixed(1)}% · river@${world.riverPos}`
    );
  }

  // Generational turnover
  if (env.mode === 'pack' && world.tick % env.generationIntervalTicks === 0 && world.tick > 0) {
    autonomousGeneration();
  }

  // Autosave lineage state
  if (world.tick % env.saveIntervalTicks === 0 && world.tick > 0) {
    autonomousSave();
  }

  if (world.tick % 60 === 0) {
    pushEvent(
      `t=${world.tick} live=${world.pack.length} hero S=${world.hero.S.toFixed(3)} ${world.hero.regime} ` +
        `P=${world.hero.vitalityPressure.toFixed(2)} mem=${(world.hero.workingMemory || []).length}`
    );
  }
}

// ---------- HTTP observation API ----------
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
};

const CRASH_LOG = path.join(ROOT, 'state', 'habitat', 'crash.log');

function logCrash(label, err) {
  try {
    const dir = path.dirname(CRASH_LOG);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    const line =
      `[${new Date().toISOString()}] ${label}: ${err && err.stack ? err.stack : err}\n`;
    fs.appendFileSync(CRASH_LOG, line);
    console.error(line);
  } catch (_) {
    console.error(label, err);
  }
}

process.on('uncaughtException', (err) => {
  logCrash('uncaughtException', err);
  // Keep habitat alive unless listen/boot failed hard
});
process.on('unhandledRejection', (err) => {
  logCrash('unhandledRejection', err);
});

const server = http.createServer((req, res) => {
  try {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  if (url.pathname === '/api/state') {
    try {
      const snap = worldSnapshot();
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(snap));
    } catch (e) {
      logCrash('worldSnapshot', e);
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: false, error: String(e.message || e) }));
    }
    return;
  }

  // Read-only probe of the mind (does not steer habitat laws / evolution)
  if (url.pathname === '/api/query' && req.method === 'POST') {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      try {
        const parsed = body ? JSON.parse(body) : {};
        const q = parsed.q || parsed.query || '';
        const targetId = parsed.id || null;
        const org =
          (targetId && world.pack.find((o) => o.id === targetId)) || world.hero;
        if (!org) {
          res.writeHead(503, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ ok: false, error: 'No organism available yet' }));
          return;
        }
        const result = queryOrganism(org, q, { codonMap: genetic.codons });
        result.organismId = org.id;
        result.tick = world.tick;
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(result));
      } catch (e) {
        logCrash('query', e);
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ ok: false, error: e.message }));
      }
    });
    return;
  }

  // Habitat laws stay autonomous — no feed/pause/exam control
  if (url.pathname === '/api/control') {
    res.writeHead(403, { 'Content-Type': 'application/json' });
    res.end(
      JSON.stringify({
        ok: false,
        error:
          'Habitat life is autonomous. Use POST /api/query to probe the mind. Edit env.json to change environment laws.',
      })
    );
    return;
  }

  let filePath =
    url.pathname === '/' ? path.join(ROOT, 'fsot_dashboard.html') : path.join(ROOT, url.pathname);
  if (!filePath.startsWith(ROOT) || !fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
    res.writeHead(404);
    res.end('Not found');
    return;
  }
  const ext = path.extname(filePath);
  res.writeHead(200, { 'Content-Type': MIME[ext] || 'application/octet-stream' });
  fs.createReadStream(filePath).pipe(res);
  } catch (e) {
    logCrash('http', e);
    try {
      res.writeHead(500);
      res.end('server error');
    } catch (_) {
      /* */
    }
  }
});

let timer = null;
function schedule() {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    try {
      stepWorld();
    } catch (e) {
      logCrash('stepWorld', e);
      try {
        pushEvent(`Recovered from tick error: ${e.message}`);
      } catch (_) {
        /* */
      }
    }
    schedule();
  }, env.tickMs);
}

server.on('error', (err) => {
  logCrash('server', err);
  if (err.code === 'EADDRINUSE') {
    console.error(`Port ${PORT} in use. Close other habitat or set FSOT_VIZ_PORT.`);
    process.exit(1);
  }
});

// Bind IPv4 explicitly so browsers using 127.0.0.1 always connect on Windows
server.listen(PORT, '0.0.0.0', () => {
  console.log(`\n  FSOT Autonomous Habitat`);
  console.log(`  → http://127.0.0.1:${PORT}`);
  console.log(`  → http://localhost:${PORT}`);
  console.log(`  Observation only · laws from env.json`);
  console.log(`  Data river: ${cursor.size} seeds · ${riverMeta.rootsOk.length} roots`);
  console.log(`  Crash log: ${CRASH_LOG}\n`);
  schedule();
});
