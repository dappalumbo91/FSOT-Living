/**
 * FSOT Habitat — LOCAL visual (no server, no URL)
 *
 * Runs the autonomous habitat in Node and rewrites a single HTML file
 * every tick with full state embedded. Open that file in any browser;
 * it auto-refreshes from disk. Zero localhost / API / port issues.
 *
 * Usage:
 *   node fsot_habitat_local.js
 *   npm run live
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const { FSOTLivingScalar, STATE_DIR } = require('./fsot_living_core');
const { loadGeneticBundle, genomeToGuidance, mutateGenome } = require('./fsot_genetic_data');
const { computeForDomain, CONSTANTS } = require('./fsot_scalar');
const { teach, runCognitiveBattery, queryOrganism } = require('./fsot_cognition');
const {
  loadEnvOverrides,
  buildDataRiver,
  DataRiverCursor,
} = require('./fsot_environment');

const ROOT = __dirname;
const ENV_PATH = path.join(ROOT, 'env.json');
const OUT_DIR = path.join(STATE_DIR, 'habitat');
const LIVE_HTML = path.join(OUT_DIR, 'LIVE.html');
const LIVE_JSON = path.join(OUT_DIR, 'LIVE.json');
const QUERY_IN = path.join(OUT_DIR, 'query_inbox.txt');
const QUERY_OUT = path.join(OUT_DIR, 'query_outbox.txt');

if (!fs.existsSync(OUT_DIR)) fs.mkdirSync(OUT_DIR, { recursive: true });

console.log('Booting habitat (loading data river — may take a few seconds)…');

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
  tick: 0,
  generation: 0,
  events: [],
  pack: [],
  hero: null,
  lastBattery: null,
  lastQuery: null,
  lineage: [],
  stats: { fed: 0, exams: 0, births: 0, autosaves: 0, generations: 0 },
  riverSize: cursor.size,
  riverPos: 0,
  riverMeta,
};

const pop = Math.max(2, env.population || 6);
for (let i = 0; i < pop; i++) {
  const g = i === 0 ? baseGenome.slice() : mutateGenome(baseGenome, 0.08 + i * 0.03);
  world.pack.push(makeCritter(`org-g0-${i}`, g));
}
world.hero = world.pack[0];

function pushEvent(msg) {
  world.events.unshift({ t: world.tick, msg });
  if (world.events.length > 40) world.events.pop();
}

pushEvent('Local habitat online — no server, file-based view');
pushEvent(`Data river ${cursor.size} seeds · ${riverMeta.rootsOk.length} roots`);

function snapshotOrg(org) {
  return {
    id: org.id,
    S: org.S,
    raw_S: org.raw_S,
    regime: org.regime,
    vitality: org.vitality,
    coherence: org.coherence,
    pressure: org.vitalityPressure,
    reward: org.rewardSignal,
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
    codonKnown: Object.keys(org._codonTable || {}).length,
    longTerm: (org.longTermKnowledge || []).length,
    policySizes: {
      mod: org.selfModificationPolicy.length,
      signal: org.signalGenerationLogic.length,
      domain: org.domainRegulationLogic.length,
      recall: (org.recallPolicy || []).length,
    },
    term1: org.term1,
    term2: org.term2,
    term3: org.term3,
  };
}

function worldSnapshot() {
  const pack = world.pack.map(snapshotOrg).sort((a, b) => b.fitness - a.fitness);
  return {
    mode: 'local-file',
    worldTick: world.tick,
    generation: world.generation,
    bioTargetS: bio.S,
    k: CONSTANTS.k,
    hero: snapshotOrg(world.hero),
    pack,
    events: world.events.slice(0, 20),
    stats: world.stats,
    lastBattery: world.lastBattery,
    lastQuery: world.lastQuery,
    lineage: world.lineage.slice(-12),
    river: {
      size: world.riverSize,
      position: world.riverPos,
      filesMined: riverMeta.files,
      roots: riverMeta.rootsOk.length,
    },
    env: {
      tickMs: env.tickMs,
      feedIntervalTicks: env.feedIntervalTicks,
      examIntervalTicks: env.examIntervalTicks,
      generationIntervalTicks: env.generationIntervalTicks,
      population: env.population,
    },
    paths: {
      liveHtml: LIVE_HTML,
      queryInbox: QUERY_IN,
      queryOutbox: QUERY_OUT,
    },
    updatedAt: new Date().toISOString(),
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
    id: org.id,
  };
  world.stats.exams++;
  return bat;
}

function autonomousGeneration() {
  world.pack.sort((a, b) => fitness(b) - fitness(a));
  const elites = world.pack.slice(0, env.eliteCount);
  world.hero = elites[0];
  world.lineage.push({
    gen: world.generation,
    champion: world.hero.id,
    S: world.hero.S,
    acc: world.hero.cognitiveAccuracy || 0,
    fit: fitness(world.hero),
    codonTable: Object.keys(world.hero._codonTable || {}).length,
  });
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
    child.workingMemory = (parent.workingMemory || []).slice(-25);
    next.push(child);
    world.stats.births++;
  }
  world.pack = next;
  world.generation++;
  world.stats.generations++;
  pushEvent(
    `Gen ${world.generation} · ${world.hero.id} S=${world.hero.S.toFixed(3)} ` +
      `acc=${((world.hero.cognitiveAccuracy || 0) * 100).toFixed(1)}%`
  );
}

function processQueryInbox() {
  try {
    if (!fs.existsSync(QUERY_IN)) return;
    const raw = fs.readFileSync(QUERY_IN, 'utf8').trim();
    if (!raw) return;
    // consume inbox
    fs.writeFileSync(QUERY_IN, '');
    const result = queryOrganism(world.hero, raw, { codonMap: genetic.codons });
    result.organismId = world.hero.id;
    result.tick = world.tick;
    result.asked = raw;
    world.lastQuery = result;
    fs.writeFileSync(
      QUERY_OUT,
      `[${result.organismId} · ${result.kind} · t${result.tick}]\n${result.answer || result.error}\n`
    );
    pushEvent(`Query: "${raw.slice(0, 40)}" → ${result.kind}`);
  } catch (e) {
    fs.writeFileSync(QUERY_OUT, `Query error: ${e.message}\n`);
  }
}

function stepWorld() {
  world.tick++;
  const stress =
    Math.sin(world.tick / 27) * env.stressAmplitude +
    Math.sin(world.tick / 91) * env.stressAmplitude * 0.4;

  for (const org of world.pack) {
    if (world.tick % env.feedIntervalTicks === 0) {
      const bites = cursor.next(env.feedsPerPulse);
      teach(org, bites);
      world.stats.fed += bites.length;
      world.riverPos = cursor.position;
    }
    if (world.tick % 28 === 0 && org.genome) applyGenome(org, org.genome);
    org.tickOnce(stress);
  }

  world.pack.sort((a, b) => fitness(b) - fitness(a));
  world.hero = world.pack[0];

  if (world.tick % env.examIntervalTicks === 0) {
    for (const org of world.pack) autonomousExam(org);
    pushEvent(
      `Exam · best ${((world.hero.cognitiveAccuracy || 0) * 100).toFixed(1)}% · river@${world.riverPos}`
    );
  }

  if (env.mode === 'pack' && world.tick % env.generationIntervalTicks === 0 && world.tick > 0) {
    autonomousGeneration();
  }

  if (world.tick % env.saveIntervalTicks === 0 && world.tick > 0) {
    try {
      if (world.hero.history?.length > 80) world.hero.history = world.hero.history.slice(-80);
      world.hero.id = 'fsot-habitat-champion';
      world.hero.saveState();
      world.stats.autosaves++;
      pushEvent('Autosave champion');
    } catch (e) {
      pushEvent(`Save err: ${e.message}`);
    }
  }

  if (world.tick % 50 === 0) {
    pushEvent(
      `t=${world.tick} S=${world.hero.S.toFixed(3)} ${world.hero.regime} ` +
        `c=${world.hero.coherence.toFixed(2)} codons=${Object.keys(world.hero._codonTable || {}).length}`
    );
  }

  processQueryInbox();
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function renderLiveHtml(snap) {
  const h = snap.hero;
  const rc =
    h.regime === 'emergence' ? '#3dffa8' : h.regime === 'damping' ? '#ff6b8a' : '#7ec8ff';
  const packRows = snap.pack
    .map(
      (p, i) =>
        `<tr><td>${i === 0 ? '★ ' : ''}${escapeHtml(p.id)}</td>` +
        `<td style="color:${p.regime === 'emergence' ? '#3dffa8' : '#ff6b8a'}">${p.S.toFixed(3)}</td>` +
        `<td>${(p.cognitive * 100).toFixed(0)}%</td>` +
        `<td>${p.fitness.toFixed(2)}</td>` +
        `<td>${p.codonKnown}</td></tr>`
    )
    .join('');
  const events = snap.events.map((e) => `<div>[t${e.t}] ${escapeHtml(e.msg)}</div>`).join('');
  const lineage = (snap.lineage || [])
    .slice()
    .reverse()
    .map(
      (L) =>
        `<div>g${L.gen} ${escapeHtml(L.champion)} S=${Number(L.S).toFixed(3)} ` +
        `acc=${(L.acc * 100).toFixed(0)}% fit=${Number(L.fit).toFixed(2)}</div>`
    )
    .join('');
  const lq = snap.lastQuery
    ? `<pre>${escapeHtml(`[${snap.lastQuery.organismId} · ${snap.lastQuery.kind} · t${snap.lastQuery.tick}]\n${snap.lastQuery.answer || ''}`)}</pre>`
    : '<p class="muted">Type a question in the box below and click Ask (writes a local file the habitat reads).</p>';

  // query form uses file protocol trick: user types, we can't POST without server —
  // so we use a downloadable approach: form can't write files from browser.
  // Instead show instructions + last answer, and provide a tiny companion:
  // the LIVE page includes an input that copies query to clipboard + path hint.
  // Real query: user edits query_inbox.txt OR uses the helper button that
  // downloads a query file — simplest is meta-refresh and external notepad.
  // Better: embed ActiveX? No.
  // Best local: HTML form with JavaScript that cannot write to disk on file://
  // So query works via Node watching QUERY_IN — user pastes into a visible path.
  // We'll add a simple approach: second file QUERY_IN opened instructions.

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta http-equiv="refresh" content="1"/>
<title>FSOT Habitat LIVE (local file)</title>
<style>
  body{margin:0;font-family:Segoe UI,system-ui,sans-serif;background:#0b0f14;color:#e8f0f7}
  header{padding:12px 16px;border-bottom:1px solid #1e2a36;display:flex;flex-wrap:wrap;gap:10px;align-items:center}
  h1{margin:0;font-size:1.05rem}
  .sub{color:#8aa0b5;font-size:.8rem}
  .badge{font-size:.72rem;padding:3px 8px;border-radius:999px;border:1px solid #1e2a36;color:#8aa0b5}
  .live{border-color:#3dffa8;color:#3dffa8}
  .grid{display:grid;grid-template-columns:1.2fr 1fr 1fr;gap:12px;padding:12px;max-width:1400px;margin:0 auto}
  @media(max-width:1000px){.grid{grid-template-columns:1fr}}
  .panel{background:#121a22;border:1px solid #1e2a36;border-radius:12px;padding:12px}
  h2{margin:0 0 8px;font-size:.72rem;text-transform:uppercase;letter-spacing:.08em;color:#8aa0b5}
  .meters label{display:flex;justify-content:space-between;font-size:.75rem;color:#8aa0b5}
  .bar{height:8px;background:#0a1018;border-radius:99px;overflow:hidden;border:1px solid #1e2a36;margin:3px 0 8px}
  .bar>i{display:block;height:100%;background:${rc}}
  table{width:100%;border-collapse:collapse;font-size:.78rem}
  td,th{padding:4px 6px;border-bottom:1px solid #15202b;text-align:left}
  .log{max-height:200px;overflow:auto;font-family:Consolas,monospace;font-size:.72rem;color:#8aa0b5}
  .genome{font-family:Consolas,monospace;font-size:.65rem;color:#c4a1ff;word-break:break-all}
  .aa{color:#5eead4;font-family:Consolas,monospace;font-size:.8rem}
  .big{font-size:1.4rem;color:${rc};font-weight:600}
  .muted{color:#8aa0b5;font-size:.8rem}
  pre{white-space:pre-wrap;color:#5eead4;font-size:.8rem;background:#0c1219;padding:10px;border-radius:8px;border:1px solid #1e2a36}
  .query-help{font-size:.78rem;color:#8aa0b5;line-height:1.4}
  code{color:#ffc857}
  .creature{width:100%;height:280px;border-radius:12px;background:radial-gradient(circle at 50% 45%,${rc}22, #070b10 70%);
    border:1px solid #1e2a36;display:flex;align-items:center;justify-content:center;position:relative}
  .orb{width:90px;height:90px;border-radius:50%;background:radial-gradient(circle at 35% 35%,${rc},#0a1520);
    box-shadow:0 0 40px ${rc}66;border:2px solid ${rc}}
  .ring{position:absolute;width:160px;height:160px;border-radius:50%;border:2px solid ${rc}44}
</style>
</head>
<body>
<header>
  <div>
    <h1>FSOT Habitat — LOCAL FILE VIEW</h1>
    <div class="sub">No server · this file rewrites itself every tick · auto-refreshes</div>
  </div>
  <span class="badge live">LOCAL LIVE</span>
  <span class="badge">t=${snap.worldTick}</span>
  <span class="badge">gen ${snap.generation}</span>
  <span class="badge">river ${snap.river.position}/${snap.river.size}</span>
</header>
<div class="grid">
  <div class="panel">
    <h2>Organism</h2>
    <div class="creature"><div class="ring"></div><div class="orb"></div></div>
    <p class="big">${escapeHtml(h.id)}</p>
    <p style="color:${rc}">${h.regime.toUpperCase()} · S=${h.S.toFixed(4)} · fit=${h.fitness.toFixed(2)}</p>
    <div class="aa">AA: ${escapeHtml(h.aaChain || '—')}</div>
    <div class="genome">${escapeHtml(h.genome || '—')}</div>
  </div>
  <div class="panel">
    <h2>Vitals</h2>
    <div class="meters">
      <label><span>S</span><span>${h.S.toFixed(4)}</span></label>
      <div class="bar"><i style="width:${Math.max(0, Math.min(100, ((h.S + 1) / 2) * 100))}%"></i></div>
      <label><span>Coherence</span><span>${h.coherence.toFixed(3)}</span></label>
      <div class="bar"><i style="width:${(h.coherence * 100).toFixed(0)}%;background:#7ec8ff"></i></div>
      <label><span>Pressure</span><span>${h.pressure.toFixed(3)}</span></label>
      <div class="bar"><i style="width:${Math.min(100, h.pressure * 100).toFixed(0)}%;background:#ffc857"></i></div>
      <label><span>Cognitive</span><span>${(h.cognitive * 100).toFixed(1)}%</span></label>
      <div class="bar"><i style="width:${(h.cognitive * 100).toFixed(0)}%;background:#5eead4"></i></div>
      <label><span>Codons known</span><span>${h.codonKnown}/64</span></label>
      <div class="bar"><i style="width:${((h.codonKnown / 64) * 100).toFixed(0)}%;background:#c4a1ff"></i></div>
    </div>
    <p class="muted">fed ${snap.stats.fed} · exams ${snap.stats.exams} · births ${snap.stats.births} · saves ${snap.stats.autosaves}</p>
    <p class="muted">D_eff=${h.D_eff.toFixed(2)} δψ=${h.delta_psi.toFixed(3)} mem=${h.memSize} LT=${h.longTerm}</p>
  </div>
  <div class="panel">
    <h2>Population</h2>
    <table><tr><th>id</th><th>S</th><th>acc</th><th>fit</th><th>cod</th></tr>${packRows}</table>
    <h2 style="margin-top:12px">Lineage</h2>
    <div class="log">${lineage || 'awaiting generations…'}</div>
  </div>
  <div class="panel" style="grid-column:1/-1">
    <h2>Query (local file — no server)</h2>
    <div class="query-help">
      Write your question into this file (one line), save it, wait 1 second:<br/>
      <code>${escapeHtml(QUERY_IN)}</code><br/>
      Answer appears here and in: <code>${escapeHtml(QUERY_OUT)}</code><br/>
      Examples: <code>hello how are you</code> · <code>status</code> · <code>codon ATG</code> · <code>domain quantum</code> · <code>genome</code>
    </div>
    <div style="margin-top:8px">${lq}</div>
  </div>
  <div class="panel" style="grid-column:1/-1">
    <h2>Habitat log</h2>
    <div class="log">${events}</div>
  </div>
</div>
<footer style="text-align:center;color:#8aa0b5;font-size:.72rem;padding:12px">
  File path: ${escapeHtml(LIVE_HTML)} · updated ${escapeHtml(snap.updatedAt)} · auto-refresh 1s
</footer>
</body>
</html>`;
}

function writeLive() {
  const snap = worldSnapshot();
  fs.writeFileSync(LIVE_JSON, JSON.stringify(snap, null, 2));
  fs.writeFileSync(LIVE_HTML, renderLiveHtml(snap), 'utf8');
}

// Ensure query inbox exists
if (!fs.existsSync(QUERY_IN)) {
  fs.writeFileSync(
    QUERY_IN,
    '# Type one question on the NEXT line, save file, wait 1s\nhello how are you\n'
  );
}
if (!fs.existsSync(QUERY_OUT)) fs.writeFileSync(QUERY_OUT, '');

// Initial render + open file
writeLive();
console.log('\n========================================');
console.log(' FSOT LOCAL HABITAT (no server)');
console.log(' Open this file in your browser:');
console.log(' ' + LIVE_HTML);
console.log(' It auto-refreshes every 1 second.');
console.log(' Query: edit ' + QUERY_IN);
console.log('========================================\n');

// Open in default browser as a FILE, not URL
try {
  if (process.platform === 'win32') {
    spawn('cmd', ['/c', 'start', '', LIVE_HTML], { detached: true, stdio: 'ignore' });
  } else if (process.platform === 'darwin') {
    spawn('open', [LIVE_HTML], { detached: true, stdio: 'ignore' });
  } else {
    spawn('xdg-open', [LIVE_HTML], { detached: true, stdio: 'ignore' });
  }
} catch (e) {
  console.log('Open the LIVE.html path manually if it did not pop up.');
}

let running = true;
function loop() {
  if (!running) return;
  try {
    stepWorld();
    writeLive();
    if (world.tick % 100 === 0) {
      console.log(
        `t=${world.tick} gen=${world.generation} S=${world.hero.S.toFixed(3)} ` +
          `${world.hero.regime} acc=${((world.hero.cognitiveAccuracy || 0) * 100).toFixed(1)}% ` +
          `codons=${Object.keys(world.hero._codonTable || {}).length}`
      );
    }
  } catch (e) {
    console.error('tick error', e.message);
    pushEvent('tick error: ' + e.message);
    try {
      writeLive();
    } catch (_) {
      /* */
    }
  }
  setTimeout(loop, env.tickMs);
}

process.on('uncaughtException', (e) => {
  console.error('uncaught', e);
  pushEvent('uncaught: ' + e.message);
});

loop();
