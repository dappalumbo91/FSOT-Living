/**
 * FSOT Cognition Layer — task fitness for evolving intelligence
 * Domain signs, codon→AA, regime classification, working-memory recall.
 * Fundamentally FSOT-native: answers drawn from scalar + genetic substrate.
 */

'use strict';

const { computeForDomain, DOMAIN_PARAMS } = require('./fsot_scalar');
const { GENETIC_CODE } = require('./fsot_genetic_data');

const DOMAIN_NAMES = Object.keys(DOMAIN_PARAMS);

/** Tokenize text for crude associative memory keys. */
function tokens(text) {
  return String(text)
    .toLowerCase()
    .replace(/[^a-z0-9_\s.-]/g, ' ')
    .split(/\s+/)
    .filter((t) => t.length > 2)
    .slice(0, 24);
}

/**
 * Store an episode into working memory using evolvable recall policy weights.
 */
function storeWorkingMemory(entity, episode) {
  if (!entity.workingMemory) entity.workingMemory = [];
  if (!entity.recallPolicy) {
    entity.recallPolicy = [
      { feature: 'recency', weight: 1.0 },
      { feature: 'strength', weight: 0.9 },
      { feature: 'typeMatch', weight: 0.85 },
      { feature: 'tokenOverlap', weight: 1.1 },
      { feature: 'regimeMatch', weight: 0.7 },
    ];
  }

  const entry = {
    tick: entity.tick,
    text: (episode.text || '').slice(0, 200),
    type: episode.type || 'general',
    strength: episode.strength ?? 0.5,
    domain: episode.domain || null,
    S: episode.S ?? entity.S,
    regime: episode.regime || entity.regime,
    tokens: tokens(episode.text || ''),
    answer: episode.answer || null,
    tags: episode.tags || [],
  };

  entity.workingMemory.push(entry);
  const cap = Math.max(20, Math.min(120, Math.round(40 + (entity.domains.memory || 0.7) * 40)));
  while (entity.workingMemory.length > cap) entity.workingMemory.shift();
  return entry;
}

function policyWeight(entity, feature) {
  const rule = (entity.recallPolicy || []).find((r) => r.feature === feature);
  return rule ? rule.weight : 0.5;
}

/**
 * Associative recall ranked by evolvable recall policy.
 */
function recall(entity, query, topK = 3) {
  const mem = entity.workingMemory || [];
  if (!mem.length) return [];

  const qTokens = new Set(tokens(query.text || query));
  const qType = query.type || null;
  const qRegime = query.regime || entity.regime;
  const now = entity.tick || 0;

  const scored = mem.map((e) => {
    const recency = 1 / (1 + (now - e.tick) / 50);
    const strength = e.strength;
    const typeMatch = qType && e.type === qType ? 1 : qType ? 0.2 : 0.5;
    let overlap = 0;
    if (qTokens.size && e.tokens.length) {
      let hit = 0;
      for (const t of e.tokens) if (qTokens.has(t)) hit++;
      overlap = hit / Math.max(1, qTokens.size);
    }
    const regimeMatch = e.regime === qRegime ? 1 : 0.3;

    const score =
      recency * policyWeight(entity, 'recency') +
      strength * policyWeight(entity, 'strength') +
      typeMatch * policyWeight(entity, 'typeMatch') +
      overlap * policyWeight(entity, 'tokenOverlap') +
      regimeMatch * policyWeight(entity, 'regimeMatch');

    return { entry: e, score };
  });

  scored.sort((a, b) => b.score - a.score);
  return scored.slice(0, topK);
}

function mutateRecallPolicy(entity) {
  if (!entity.recallPolicy) return;
  const kind = Math.floor(Math.random() * 3);
  if (kind === 0 && entity.recallPolicy.length < 8) {
    const features = ['recency', 'strength', 'typeMatch', 'tokenOverlap', 'regimeMatch', 'Sproximity'];
    entity.recallPolicy.push({
      feature: features[Math.floor(Math.random() * features.length)],
      weight: 0.5 + Math.random() * 1.0,
    });
  } else if (kind === 1 && entity.recallPolicy.length > 2) {
    entity.recallPolicy.splice(Math.floor(Math.random() * entity.recallPolicy.length), 1);
  } else {
    const r = entity.recallPolicy[Math.floor(Math.random() * entity.recallPolicy.length)];
    r.weight = Math.max(0.2, Math.min(2.0, r.weight + (Math.random() - 0.5) * 0.3));
  }
}

// ---------- Cognitive tasks ----------

function taskDomainSign(entity) {
  const name = DOMAIN_NAMES[Math.floor(Math.random() * DOMAIN_NAMES.length)];
  const truth = computeForDomain(name);
  const trueSign = truth.S > 0 ? 'emergence' : truth.S < 0 ? 'damping' : 'neutral';

  // Organism "thinks" by aligning params toward domain then reading regime/S
  const before = {
    D_eff: entity.params.D_eff,
    delta_psi: entity.params.delta_psi,
    observed: entity.observed,
  };
  const dp = DOMAIN_PARAMS[name];
  entity.params.D_eff = entity.params.D_eff * 0.4 + dp.D_eff * 0.6;
  entity.params.delta_psi = entity.params.delta_psi * 0.4 + dp.delta_psi * 0.6;
  entity.observed = dp.observed;
  entity.computeScalar(0, { freezePhase: true });

  let predicted = entity.regime;
  // Memory assist: if we've seen this domain, bias toward stored regime
  const hits = recall(entity, { text: `domain ${name}`, type: 'fsot_domain' }, 2);
  if (hits.length && hits[0].score > 1.5 && hits[0].entry.regime) {
    predicted = hits[0].entry.regime;
  }

  // Restore body
  entity.params.D_eff = before.D_eff;
  entity.params.delta_psi = before.delta_psi;
  entity.observed = before.observed;
  entity.computeScalar(0, { freezePhase: true });

  const correct = predicted === trueSign;
  return {
    task: 'domain_sign',
    domain: name,
    predicted,
    truth: trueSign,
    correct,
    score: correct ? 1 : 0,
  };
}

function taskCodonAA(entity, codonMap) {
  const codons = Object.keys(codonMap || GENETIC_CODE);
  const codon = codons[Math.floor(Math.random() * codons.length)];
  const truth = (codonMap && codonMap[codon]?.aa) || GENETIC_CODE[codon] || '?';

  // Query memory first
  const hits = recall(entity, { text: `Codon ${codon}`, type: 'fsot_codon' }, 3);
  let predicted = null;
  for (const h of hits) {
    const m = h.entry.text.match(/Codon\s+([ACGT]{3})\s*→\s*AA\s+(\S+)/i);
    if (m && m[1] === codon) {
      predicted = m[2];
      break;
    }
  }

  // Genome / genetic code fallback (organism "knows" if exposed)
  if (!predicted && entity._codonTable && entity._codonTable[codon]) {
    predicted = entity._codonTable[codon];
  }
  if (!predicted) {
    // weak guess from genome composition
    predicted = truth === '*' ? 'M' : 'X';
  }

  const correct = predicted === truth;
  return {
    task: 'codon_aa',
    codon,
    predicted,
    truth,
    correct,
    score: correct ? 1 : 0,
  };
}

function taskRegimeSelf(entity) {
  entity.computeScalar(0, { freezePhase: true });
  const truth = entity.S > 0.02 ? 'emergence' : entity.S < -0.02 ? 'damping' : 'neutral';
  // Must report own regime correctly — metacognition probe
  const predicted = entity.regime;
  const correct = predicted === truth || (truth === 'neutral' && Math.abs(entity.S) < 0.05);
  return {
    task: 'regime_self',
    predicted,
    truth,
    correct,
    score: correct ? 1 : 0,
  };
}

function taskRecallFact(entity) {
  const mem = entity.workingMemory || [];
  if (mem.length < 3) {
    return { task: 'recall_fact', correct: false, score: 0, skipped: true };
  }
  const target = mem[Math.floor(Math.random() * mem.length)];
  const cue = target.tokens.slice(0, 3).join(' ') || target.type;
  const hits = recall(entity, { text: cue, type: target.type }, 1);
  const correct =
    hits.length > 0 &&
    (hits[0].entry.text === target.text ||
      (hits[0].entry.tokens[0] && target.tokens.includes(hits[0].entry.tokens[0])));
  return {
    task: 'recall_fact',
    cue,
    correct: !!correct,
    score: correct ? 1 : 0,
  };
}

function taskBioAlignment(entity, bioTargetS) {
  const err = Math.abs(entity.S - bioTargetS);
  const score = Math.max(0, 1 - err / 0.8);
  return {
    task: 'bio_alignment',
    S: entity.S,
    target: bioTargetS,
    score,
    correct: err < 0.15,
  };
}

/**
 * Parse text into durable knowledge structures (codons, domains, facts).
 * Long-term knowledge survives working-memory eviction.
 */
function internalizeText(entity, text) {
  if (!text) return;
  entity._codonTable = entity._codonTable || {};
  entity.longTermKnowledge = entity.longTermKnowledge || [];
  entity.domainMemory = entity.domainMemory || {};

  // Codon → AA (multiple formats from river / genetics)
  const codonRe =
    /Codon\s+([ACGT]{3})\s*(?:→|->|to|=|:)?\s*(?:AA\s+)?([A-Z*]|\?)/gi;
  let m;
  while ((m = codonRe.exec(text)) !== null) {
    entity._codonTable[m[1].toUpperCase()] = m[2].toUpperCase();
  }
  // AAA | [1,1,1] style lines + genetic code fallback later
  const codonLine = text.match(/\b([ACGT]{3})\b[^\n]{0,40}\bAA\s*([A-Z*])/i);
  if (codonLine) entity._codonTable[codonLine[1].toUpperCase()] = codonLine[2].toUpperCase();

  // Domain ledger lines
  const dom = text.match(
    /(?:Lean\s+)?domain\s+(\w+)[:\s]+.*?S=([-\d.]+).*?regime=(\w+)/i
  );
  if (dom) {
    entity.domainMemory[dom[1].toLowerCase()] = {
      S: parseFloat(dom[2]),
      regime: dom[3].toLowerCase(),
      text: text.slice(0, 240),
    };
  }

  // Durable fact if structured
  if (
    /FSOT|codon|domain|S=|raw_S|amino|genome|quantum|cosmolog/i.test(text) &&
    text.length > 40
  ) {
    entity.longTermKnowledge.push({
      tick: entity.tick || 0,
      text: text.slice(0, 280),
      tokens: tokens(text),
    });
    if (entity.longTermKnowledge.length > 400) {
      entity.longTermKnowledge.splice(0, entity.longTermKnowledge.length - 400);
    }
  }
}

/**
 * Interactive query — for development / probing the mind without steering life.
 * Returns structured answer from scalar, genome, memory, codon table, domains.
 */
function queryOrganism(entity, question, options = {}) {
  const q = String(question || '').trim();
  const lower = q.toLowerCase();
  const codonMap = options.codonMap || null;
  const hits = [];

  if (!q) {
    return { ok: false, error: 'Empty query', answer: '' };
  }

  // Always re-sync tables from memory
  for (const e of entity.workingMemory || []) internalizeText(entity, e.text);
  for (const e of entity.longTermKnowledge || []) internalizeText(entity, e.text);

  // Greetings / status / self (match anywhere — "hello how are you" must not fall into river dump)
  const isGreeting =
    /\b(hello|hi|hey|greetings|good\s+(morning|afternoon|evening))\b/i.test(q) ||
    /\bhow are you\b/i.test(q) ||
    /\bhow('s| is) it going\b/i.test(q) ||
    /\bwhat'?s up\b/i.test(q);
  const isStatus =
    /^(status|state|vitals?)\b/i.test(q) ||
    /\b(who are you|what are you|your status|your state|your regime)\b/i.test(q) ||
    /\bhow (am i|are we|is the organism)\b/i.test(q);

  if (isGreeting || isStatus) {
    const mood =
      entity.regime === 'emergence'
        ? 'in an emergence regime — vitality is constructive'
        : entity.regime === 'damping'
          ? 'in a damping regime — under pressure, stabilizing'
          : 'near neutral balance';
    const feel =
      (entity.vitalityPressure || 0) < 0.2
        ? 'pressure is low; I am stable'
        : (entity.vitalityPressure || 0) < 0.5
          ? 'I feel moderate evolutionary pressure'
          : 'pressure is high; I am adapting hard';
    const greeting = isGreeting
      ? `Hello. I am FSOT organism ${entity.id || 'unknown'}. `
      : '';
    const answer =
      greeting +
      `I am ${mood}. ${feel}. ` +
      `S=${Number(entity.S).toFixed(4)} (raw ${Number(entity.raw_S).toFixed(4)}), ` +
      `coherence=${Number(entity.coherence).toFixed(3)}, ` +
      `pressure=${Number(entity.vitalityPressure).toFixed(3)}, ` +
      `cognitive accuracy=${((entity.cognitiveAccuracy || 0) * 100).toFixed(1)}%, ` +
      `codons learned=${Object.keys(entity._codonTable || {}).length}/64, ` +
      `working memory=${(entity.workingMemory || []).length}, ` +
      `long-term facts=${(entity.longTermKnowledge || []).length}. ` +
      `D_eff=${Number(entity.params?.D_eff).toFixed(2)}, δψ=${Number(entity.params?.delta_psi).toFixed(3)}. ` +
      (isGreeting
        ? 'Ask me "status", "codon ATG", "domain quantum", or "genome" for specifics.'
        : '');
    return {
      ok: true,
      kind: isGreeting ? 'greeting' : 'status',
      answer,
      S: entity.S,
      regime: entity.regime,
    };
  }

  // Codon query
  const codonQ = q.match(/\b([ACGT]{3})\b/i);
  if (/codon/i.test(q) || (codonQ && /amino|aa|code|what/i.test(q))) {
    const codon = (codonQ ? codonQ[1] : q.match(/codon\s+([ACGT]{3})/i)?.[1] || '').toUpperCase();
    if (codon) {
      const learned = entity._codonTable?.[codon];
      const envTruth = codonMap?.[codon]?.aa || GENETIC_CODE[codon];
      const aa = learned || envTruth || '?';
      const source = learned ? 'learned' : envTruth ? 'genetic_code' : 'unknown';
      const answer = `Codon ${codon} → amino acid ${aa} (source: ${source}). ` +
        (entity.genome?.includes?.(codon)
          ? `Present in my genome.`
          : `Not in current genome sequence.`);
      hits.push({ codon, aa, source });
      return { ok: true, kind: 'codon', answer, codon, aa, source };
    }
  }

  // Domain query
  const domName = q.match(/domain\s+(\w+)/i)?.[1] ||
    DOMAIN_NAMES.find((d) => lower.includes(d.replace(/_/g, ' ')) || lower.includes(d));
  if (domName || /quantum|cosmolog|biolog|neural|chemical/i.test(q)) {
    const name = (domName || (lower.includes('quantum')
      ? 'quantum'
      : lower.includes('cosmo')
        ? 'cosmological'
        : lower.includes('bio')
          ? 'biological'
          : lower.includes('neural')
            ? 'neuroscience'
            : 'chemical')).toLowerCase();
    const mem = entity.domainMemory?.[name];
    let truth = null;
    try {
      if (DOMAIN_PARAMS[name]) truth = computeForDomain(name);
    } catch {
      /* */
    }
    const regime = mem?.regime || truth?.regime || '?';
    const S = mem?.S ?? truth?.S;
    const answer =
      `Domain ${name}: regime=${regime}` +
      (S != null ? ` S=${Number(S).toFixed(6)}` : '') +
      (mem ? ' (from my knowledge).' : truth ? ' (computed from FSOT ledger).' : ' (unknown to me).');
    return { ok: true, kind: 'domain', answer, domain: name, S, regime };
  }

  // Genome
  if (/genome|amino.?acid chain|what.*genes/i.test(q)) {
    const g = (entity.genome || []).join('-');
    const aa = entity.genomeGuidance?.aaChain || '';
    const answer = `Genome (${(entity.genome || []).length} codons): ${g || 'none'}. AA chain: ${aa || 'n/a'}.`;
    return { ok: true, kind: 'genome', answer, genome: g, aaChain: aa };
  }

  // Free recall — ignore stopwords so "how/are/you/hello" cannot dump the whole river
  const STOP = new Set([
    'the', 'and', 'for', 'are', 'you', 'how', 'what', 'who', 'why', 'when', 'where',
    'this', 'that', 'with', 'from', 'have', 'has', 'was', 'were', 'hello', 'hey',
    'please', 'tell', 'about', 'into', 'just', 'like', 'know', 'does', 'did',
  ]);
  const qTok = tokens(q).filter((t) => t.length > 2 && !STOP.has(t));
  if (qTok.length === 0) {
    const answer =
      `I heard you, but that message has no search terms I can ground. ` +
      `My state: S=${Number(entity.S).toFixed(4)} regime=${entity.regime}. ` +
      `Try: status · codon ATG · domain quantum · genome · or a keyword like FSOT, protein, codon.`;
    return { ok: true, kind: 'clarify', answer, S: entity.S, regime: entity.regime };
  }

  const memHits = recall(entity, { text: qTok.join(' ') }, 4).filter((h) => h.score > 0.8);
  const lt = (entity.longTermKnowledge || [])
    .map((e) => {
      let ov = 0;
      for (const t of e.tokens || []) if (qTok.includes(t) || qTok.some((qt) => t.includes(qt))) ov++;
      return { e, score: ov };
    })
    .filter((x) => x.score >= 1)
    .sort((a, b) => b.score - a.score)
    .slice(0, 3);

  const snippets = [
    ...memHits.map((h) => h.entry.text),
    ...lt.map((x) => x.e.text),
  ].filter(Boolean);

  if (snippets.length) {
    const answer =
      `From memory (${snippets.length} hits for [${qTok.join(', ')}]):\n` +
      snippets.map((s, i) => `${i + 1}. ${s.slice(0, 180)}`).join('\n');
    storeWorkingMemory(entity, {
      text: `Query: ${q} → ${snippets[0].slice(0, 100)}`,
      type: 'query',
      strength: 0.6,
    });
    return {
      ok: true,
      kind: 'recall',
      answer,
      hits: snippets.length,
      confidence: Math.min(1, snippets.length / 4),
    };
  }

  // Scalar reflection fallback
  entity.computeScalar?.(0, { freezePhase: true });
  const answer =
    `No strong memory match. My present FSOT state: S=${Number(entity.S).toFixed(4)} ` +
    `regime=${entity.regime}. I know ${Object.keys(entity._codonTable || {}).length}/64 codons ` +
    `and ${(entity.longTermKnowledge || []).length} long-term facts. ` +
    `Try: "status", "codon ATG", "domain quantum", "genome", or keywords from my data river.`;
  return { ok: true, kind: 'fallback', answer, S: entity.S, regime: entity.regime };
}

/**
 * Run a cognitive battery. Returns accuracy + per-task breakdown.
 * Builds codon table from working memory / genetic exposure.
 */
function runCognitiveBattery(entity, options = {}) {
  const rounds = options.rounds ?? 12;
  const codonMap = options.codonMap || null;
  const bioTargetS = options.bioTargetS ?? 0.48;

  entity._codonTable = entity._codonTable || {};
  for (const e of entity.workingMemory || []) internalizeText(entity, e.text);
  for (const e of entity.longTermKnowledge || []) internalizeText(entity, e.text);

  // Knowledge depth bonus: more codons internalized → better battery baseline
  const codonDepth = Object.keys(entity._codonTable).length / 64;

  const results = [];
  let scoreSum = 0;
  let n = 0;

  for (let i = 0; i < rounds; i++) {
    const pick = i % 5;
    let r;
    if (pick === 0) r = taskDomainSign(entity);
    else if (pick === 1) r = taskCodonAA(entity, codonMap);
    else if (pick === 2) r = taskRegimeSelf(entity);
    else if (pick === 3) r = taskRecallFact(entity);
    else r = taskBioAlignment(entity, bioTargetS);

    if (r.skipped) continue;
    results.push(r);
    scoreSum += r.score;
    n++;
  }

  const accuracy = n ? scoreSum / n : 0;
  entity.cognitiveAccuracy = accuracy;
  entity.lastCognitiveResults = results;
  entity.intelligenceScore =
    accuracy * 1.4 +
    codonDepth * 0.5 +
    (entity.informationIntegration || 0) * 0.35 +
    (entity.adaptiveEfficiency || 0) * 0.25 +
    Math.min(0.3, (entity.longTermKnowledge || []).length / 400) +
    (entity.regime === 'emergence' ? 0.12 : 0) -
    (entity.vitalityPressure || 0) * 0.2;

  if (accuracy > 0.55) {
    entity.rewardSignal = Math.min(0.88, (entity.rewardSignal || 0.35) + 0.04);
    entity.domains.informationProcessing = Math.min(
      1.55,
      (entity.domains.informationProcessing || 0.65) * 1.01
    );
    entity.domains.memory = Math.min(1.55, (entity.domains.memory || 0.72) * 1.012);
  } else if (accuracy < 0.35) {
    entity.vitalityPressure = Math.min(1.5, (entity.vitalityPressure || 0) + 0.05);
  }

  return {
    accuracy,
    intelligenceScore: entity.intelligenceScore,
    codonDepth,
    n,
    results,
  };
}

/**
 * Teach: ingest seeds + internalize durable knowledge.
 */
function teach(entity, seeds) {
  let n = 0;
  for (const seed of seeds) {
    const text = typeof seed === 'string' ? seed : seed.text || '';
    const r = entity.ingestInformation(seed);
    storeWorkingMemory(entity, {
      text,
      type: seed.type || 'fsot',
      strength: seed.strength ?? r.strength ?? 0.7,
      domain: seed.domain,
      S: seed.S ?? r.S,
      regime: seed.regime || r.regime,
      tags: [seed.type, seed.domain].filter(Boolean),
    });
    internalizeText(entity, text);
    n++;
  }
  return n;
}

module.exports = {
  storeWorkingMemory,
  recall,
  mutateRecallPolicy,
  runCognitiveBattery,
  teach,
  tokens,
  internalizeText,
  queryOrganism,
};
