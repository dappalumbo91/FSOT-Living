/**
 * FSOT Living Scalar Organism — shared core (v3.0)
 * Grounded in verified FSOT 2.1 scalar engine (fsot_scalar.js).
 * Pressure-driven structural evolution with mutation selection.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { computeS, CONSTANTS, DOMAIN_PARAMS, getDomainParams } = require('./fsot_scalar');

const STATE_DIR = path.join(__dirname, 'state');

function ensureStateDir() {
  if (!fs.existsSync(STATE_DIR)) fs.mkdirSync(STATE_DIR, { recursive: true });
}

function deepClone(obj) {
  return JSON.parse(JSON.stringify(obj));
}

class FSOTLivingScalar {
  constructor(id = 'fsot-entity-v30', loadFromFile = true) {
    this.id = id;
    this.tick = 0;
    this.version = '3.1.0';

    // FSOT scalar state
    this.S = 0;
    this.raw_S = 0;
    this.term1 = 0;
    this.term2 = 1;
    this.term3 = 0;
    this.quirk_mod = 1;
    this.regime = 'neutral';

    this.vitality = 0.5;
    this.coherence = 0.6;
    this.phase = 0;
    this.homeostaticHealth = 0.55;
    this.stabilitySignal = 0.5;
    this.rewardSignal = 0.35;
    this.baselineStability = 0.52;
    this.lowStateCounter = 0;
    this.recentConsistency = 1;
    this.vitalityPressure = 0;
    this.targetS = 0.35;

    this.informationIntegration = 0.5;
    this.patternDistinctionBalance = 0.5;
    this.adaptiveEfficiency = 0.5;
    this.crossCategoryGeneralization = 0.5;

    this.observed = false;
    this.observationCount = 0;
    this.recent_hits = 0;

    this.domains = {
      scalarCore: 1.0,
      observerLayer: 0.9,
      coherenceMaintenance: 1.0,
      domainCoupling: 0.82,
      selfModification: 0.58,
      memory: 0.72,
      informationProcessing: 0.65,
      signaling: 0.6,
      efficiency: 0.6,
    };

    // Maps onto FSOT computeS inputs
    this.params = {
      D_eff: 14,
      N: 1,
      P: 1,
      delta_psi: 0.7,
      delta_theta: 1.0,
      rho: 1.0,
      scale: 1.0,
      amplitude: 1.0,
      trend_bias: 0.0,
      phaseSensitivity: 1.05,
      mutationRate: 0.019,
      observerResponsiveness: 0.78,
      boundaryStrictness: 0.9,
      emergencePreference: 0.62,
    };

    this.selfModificationPolicy = [
      { type: 'pressureThreshold', value: 0.35, strength: 1.0 },
      { type: 'consistencyBonus', value: 2.0, strength: 0.8 },
      { type: 'vitalityFloor', value: -0.15, strength: 1.2 },
    ];

    this.signalGenerationLogic = [
      { feature: 'strength', weight: 1.0, influence: 0.8 },
      { feature: 'stability', weight: 0.9, influence: 0.7 },
      { feature: 'coherence', weight: 0.85, influence: 0.75 },
    ];

    this.domainRegulationLogic = [
      { condition: 'highCoherence', target: 'selfModification', boost: 0.018 },
      { condition: 'highPressure', target: 'efficiency', boost: 0.5 },
      { condition: 'lowVitality', target: 'coherenceMaintenance', boost: 1.02 },
    ];

    // Fourth evolvable subsystem — working memory / recall (intelligence organ)
    this.recallPolicy = [
      { feature: 'recency', weight: 1.0 },
      { feature: 'strength', weight: 0.9 },
      { feature: 'typeMatch', weight: 0.85 },
      { feature: 'tokenOverlap', weight: 1.1 },
      { feature: 'regimeMatch', weight: 0.7 },
    ];
    this.workingMemory = [];
    this.cognitiveAccuracy = 0;
    this.intelligenceScore = 0;
    this.lastCognitiveResults = [];
    this._codonTable = {};

    this.ingestedSignals = [];
    this.internalSignals = [];
    this.history = [];
    this.memory = [];
    this.recentIngestions = [];
    this.mutationLog = [];
    this.acceptedMutations = 0;
    this.rejectedMutations = 0;

    if (loadFromFile) this.loadState();
  }

  /**
   * Core vitality from verified FSOT scalar S_D_chaotic.
   * environmentStress damps rho slightly; phase modulates delta_psi.
   */
  computeScalar(environmentStress = 0, options = {}) {
    const freezePhase = options.freezePhase === true;
    const p = this.params;
    const domainFactor = Object.values(this.domains).reduce((a, b) => a + b, 0) / 9;

    let signalEffect = 0;
    if (this.ingestedSignals.length > 0) {
      signalEffect =
        this.ingestedSignals.reduce((sum, s) => sum + s.strength * s.decay, 0) /
        this.ingestedSignals.length;
    }

    const phaseShift = Math.sin(this.phase * p.phaseSensitivity) * 0.15;
    const delta_psi = Math.max(
      0.05,
      Math.min(1.5, p.delta_psi + phaseShift + (this.observed ? 0.05 : 0))
    );
    const rho = Math.max(0.3, p.rho - environmentStress * 0.25 + signalEffect * 0.12);
    const D_eff = Math.max(6, Math.min(28, Math.round(p.D_eff)));
    const recent_hits = Math.min(5, Math.max(0, Math.round(this.recent_hits)));

    const result = computeS({
      N: p.N,
      P: p.P,
      D_eff,
      recent_hits,
      delta_psi,
      delta_theta: p.delta_theta,
      rho,
      scale: p.scale * domainFactor,
      amplitude: p.amplitude,
      trend_bias: p.trend_bias + (p.emergencePreference - 0.5) * 0.1,
      observed: this.observed,
    });

    this.S = result.S;
    this.raw_S = result.raw_S;
    this.term1 = result.term1;
    this.term2 = result.term2;
    this.term3 = result.term3;
    this.quirk_mod = result.quirk_mod;
    this.regime = result.regime;

    // Map S into organism vitality (emergence-positive living target)
    this.vitality = Math.max(-0.55, Math.min(1.2, this.S * 1.15 + 0.12));

    const targetPressure = Math.max(0, this.targetS - this.S);
    this.vitalityPressure = this.vitalityPressure * 0.9 + targetPressure * 0.1;
    this.vitalityPressure = Math.max(0, Math.min(1.5, this.vitalityPressure));

    // Coherence tracks stability of S + formal coherence_efficiency influence
    const sStability = 1 - Math.min(1, Math.abs(this.S - this.targetS));
    const cohPull =
      CONSTANTS.coherence_efficiency * 0.08 * sStability +
      (this.regime === 'emergence' ? 0.04 : -0.02);
    this.coherence = Math.max(
      0.22,
      Math.min(
        1.0,
        this.coherence * 0.93 +
          sStability * 0.12 +
          cohPull +
          (this.vitality + 0.4) * p.boundaryStrictness * 0.06
      )
    );

    this.homeostaticHealth = Math.max(
      0.22,
      Math.min(0.9, this.homeostaticHealth * 0.975 + (this.coherence - 0.5) * 0.07)
    );

    this.stabilitySignal = Math.max(
      0.22,
      Math.min(0.9, this.stabilitySignal * 0.965 + (this.coherence - 0.5) * 0.075)
    );

    this.updateDualSystem();
    this.updateEvolutionaryMetrics();

    if (!freezePhase) {
      this.phase += 0.1;
      if (this.recent_hits > 0 && this.tick % 12 === 0) {
        this.recent_hits = Math.max(0, this.recent_hits - 1);
      }
    }

    return this.vitality;
  }

  /** Snapshot used for mutation accept/reject. */
  fitnessSnapshot() {
    return {
      S: this.S,
      pressure: this.vitalityPressure,
      coherence: this.coherence,
      vitality: this.vitality,
      cognitive: this.cognitiveAccuracy || 0,
      intelligence: this.intelligenceScore || 0,
      score:
        this.S * 0.9 -
        this.vitalityPressure * 0.7 +
        this.coherence * 0.3 +
        this.adaptiveEfficiency * 0.25 +
        (this.cognitiveAccuracy || 0) * 1.1 +
        (this.intelligenceScore || 0) * 0.4,
    };
  }

  updateDualSystem() {
    const recentImprovement = this.S > 0.05 ? 0.085 : this.S > -0.1 ? 0.02 : -0.025;
    this.rewardSignal = Math.max(
      0.12,
      Math.min(0.88, this.rewardSignal * 0.94 + recentImprovement + (this.homeostaticHealth - 0.5) * 0.05)
    );

    const targetBaseline = 0.53;
    this.baselineStability = Math.max(
      0.28,
      Math.min(
        0.82,
        this.baselineStability * 0.987 +
          (targetBaseline - this.baselineStability) * 0.035 +
          (this.coherence - 0.5) * 0.02
      )
    );

    if (this.rewardSignal > 0.48) {
      const sustainBoost = 1 + (this.rewardSignal - 0.48) * 0.025;
      this.domains.selfModification = Math.min(1.55, this.domains.selfModification * sustainBoost);
      this.domains.informationProcessing = Math.min(
        1.45,
        this.domains.informationProcessing * (1 + (this.rewardSignal - 0.48) * 0.02)
      );
      this.domains.signaling = Math.min(1.42, this.domains.signaling * 1.012);
    }
  }

  updateEvolutionaryMetrics() {
    const recentWindow = this.recentIngestions.slice(-12);
    if (recentWindow.length > 0) {
      const uniqueTypes = new Set(recentWindow.map((s) => s.type)).size;
      const typeDiversity = Math.min(1.0, uniqueTypes / 4);
      const avgStrength = recentWindow.reduce((sum, s) => sum + s.strength, 0) / recentWindow.length;
      const newIntegration = typeDiversity * (0.6 + avgStrength * 0.4);
      this.informationIntegration = this.informationIntegration * 0.82 + newIntegration * 0.18;
    }

    const uniquePatterns = new Set(this.internalSignals.map((s) => JSON.stringify(s.pattern))).size;
    const totalSignals = Math.max(1, this.internalSignals.length);
    const distinctionRatio = uniquePatterns / totalSignals;
    const compressionPenalty = this.recentConsistency > 4 ? 0.15 : 0;
    this.patternDistinctionBalance = Math.max(
      0.1,
      Math.min(0.95, distinctionRatio * 0.7 + (1 - compressionPenalty) * 0.3)
    );

    const pressureUtilization =
      this.vitalityPressure > 0.1
        ? Math.min(1.0, (this.domains.efficiency + this.domains.selfModification) / 2.5)
        : 0.6;
    const vitalityStability = 1 - Math.abs(this.vitality - 0.4);
    this.adaptiveEfficiency =
      this.adaptiveEfficiency * 0.88 + pressureUtilization * vitalityStability * 0.12;

    const recentTypes = this.ingestedSignals.slice(-8).map((s) => s.type);
    const uniqueRecent = new Set(recentTypes).size;
    const generalizationBase = Math.min(1.0, uniqueRecent / 4);
    this.crossCategoryGeneralization =
      this.crossCategoryGeneralization * 0.9 + generalizationBase * 0.1;
  }

  shouldSelfModify() {
    let score = 0;
    const threshold = 0.6;

    for (const rule of this.selfModificationPolicy) {
      if (rule.type === 'pressureThreshold' && this.vitalityPressure > rule.value) {
        score += rule.strength * (this.vitalityPressure - rule.value);
      }
      if (rule.type === 'consistencyBonus' && this.recentConsistency >= rule.value) {
        score += rule.strength * 0.3;
      }
      if (rule.type === 'vitalityFloor' && this.vitality < rule.value) {
        score -= rule.strength * 0.8;
      }
    }

    const pressureMod = 1 + this.vitalityPressure * 0.4;
    const efficiencyMod = 1 + (this.adaptiveEfficiency - 0.5) * 0.5;
    return score * pressureMod * efficiencyMod > threshold;
  }

  internalSelfCheck() {
    this.ingestedSignals.forEach((s) => {
      s.decay = Math.max(0.1, s.decay * 0.965);
    });
    this.ingestedSignals = this.ingestedSignals.filter((s) => s.decay > 0.15);

    if (this.homeostaticHealth < 0.38 || this.coherence < 0.32 || this.regime === 'damping') {
      this.lowStateCounter++;
    } else {
      this.lowStateCounter = Math.max(0, this.lowStateCounter - 2);
    }

    if (this.lowStateCounter > 55) {
      this.domains.coherenceMaintenance = Math.min(1.58, this.domains.coherenceMaintenance * 1.026);
      this.domains.observerLayer = Math.min(1.48, this.domains.observerLayer * 1.018);
      this.domains.memory = Math.min(1.42, this.domains.memory * 1.012);
      this.homeostaticHealth = Math.min(0.9, this.homeostaticHealth * 1.02 + 0.016);
      this.stabilitySignal = Math.min(0.9, this.stabilitySignal * 1.016 + 0.012);
    }

    if (this.internalSignals.length > 0) {
      const recent = this.internalSignals.slice(-6);
      const avgConsistency =
        recent.reduce((sum, s) => sum + s.consistencyScore, 0) / recent.length;
      this.recentConsistency = Math.max(1, Math.min(5, avgConsistency));
    }
  }

  observe(environmentStress = 0) {
    this.observed = true;
    this.observationCount++;
    this.computeScalar(environmentStress);
  }

  generateInternalSignal(inputFeatures) {
    const stabilityFactor =
      (this.coherence + this.homeostaticHealth + this.baselineStability) / 3;

    const mods = { strength: 1, stability: 1, coherence: 1 };
    for (const rule of this.signalGenerationLogic) {
      if (mods[rule.feature] !== undefined) {
        mods[rule.feature] = rule.weight * rule.influence;
      }
    }

    let pattern = [
      Math.round((stabilityFactor * mods.stability + inputFeatures.strength * mods.strength) * 5) % 4,
      Math.round((this.rewardSignal * 0.6 + inputFeatures.strength * mods.strength) * 6) % 4,
      Math.round((this.stabilitySignal * mods.stability + inputFeatures.strength * mods.strength) * 5) % 4,
      Math.round((this.coherence * mods.coherence + inputFeatures.strength * mods.strength) * 5) % 4,
    ];

    if (inputFeatures.type === 'symbol' && inputFeatures.text) {
      const charCode = inputFeatures.text.charCodeAt(0) || 0;
      pattern[3] = (pattern[3] + (charCode % 3)) % 4;
    }

    // Fold regime into pattern
    if (this.regime === 'emergence') pattern[0] = (pattern[0] + 1) % 4;
    if (this.regime === 'damping') pattern[1] = (pattern[1] + 1) % 4;

    const signal = {
      id: `sig_${this.tick}`,
      pattern,
      strength: parseFloat(inputFeatures.strength.toFixed(3)),
      linkedTo: inputFeatures.type || 'general',
      tick: this.tick,
      consistencyScore: 1,
      S: this.S,
      regime: this.regime,
    };

    const similar = this.internalSignals.filter(
      (s) =>
        s.linkedTo === signal.linkedTo &&
        JSON.stringify(s.pattern) === JSON.stringify(signal.pattern)
    );

    if (similar.length > 0) {
      signal.consistencyScore = similar.length + 1;
      this.domains.signaling = Math.min(1.48, this.domains.signaling * 1.015);
    }

    this.internalSignals.push(signal);
    if (this.internalSignals.length > 50) this.internalSignals.shift();
    return signal;
  }

  /**
   * Ingest free text or structured seed objects { text, type?, strength?, domain? }.
   */
  ingestInformation(input = '') {
    let text = '';
    let forcedType = null;
    let forcedStrength = null;
    let domainHint = null;

    if (typeof input === 'object' && input !== null) {
      text = input.text || '';
      forcedType = input.type || null;
      forcedStrength = input.strength ?? null;
      domainHint = input.domain || null;
    } else {
      text = String(input || '');
    }

    if (!text) return { accepted: false, reason: 'Invalid input' };

    const hasNumbers = /\d/.test(text);
    const hasMathTerms = /(plus|minus|times|divide|square|root|equals|sum|product|term1|term3|phi|scalar)/i.test(text);
    const hasScientificTerms =
      /(force|energy|mass|motion|gravity|atom|molecule|quantum|cosmology|domain|FSOT|emergence|damping)/i.test(
        text
      );
    const isFsot =
      forcedType?.startsWith('fsot_') ||
      /FSOT|D_eff|raw_S|quirk_mod|delta_psi/i.test(text);
    const wordCount = text.split(/\s+/).length;

    let strength = forcedStrength ?? 0.45;
    if (forcedStrength == null) {
      if (hasNumbers) strength += 0.12;
      if (hasMathTerms) strength += 0.18;
      if (hasScientificTerms) strength += 0.15;
      if (isFsot) strength += 0.2;
      if (wordCount > 5) strength += 0.08;
      strength = Math.max(0.2, Math.min(1.0, strength));
    }

    const type =
      forcedType ||
      (isFsot
        ? 'fsot'
        : hasMathTerms
          ? 'math'
          : hasScientificTerms
            ? 'science'
            : 'symbol');

    // Align organism toward domain ledger when a domain is named
    if (domainHint || /domain\s+(\w+)/i.test(text)) {
      const name = domainHint || text.match(/domain\s+(\w+)/i)[1];
      if (Object.prototype.hasOwnProperty.call(DOMAIN_PARAMS, name)) {
        const dp = getDomainParams(name);
        this.params.D_eff = this.params.D_eff * 0.85 + dp.D_eff * 0.15;
        this.params.delta_psi = this.params.delta_psi * 0.85 + dp.delta_psi * 0.15;
      }
    }

    // Soft pull from reported S in structured seeds
    if (typeof input === 'object' && typeof input.S === 'number') {
      this.targetS = this.targetS * 0.92 + input.S * 0.08;
    }

    const inputFeatures = {
      hasNumbers,
      hasMathTerms,
      hasScientificTerms,
      wordCount,
      strength,
      type,
      text: text.substring(0, 160),
    };

    this.recentIngestions.push({ type, strength, tick: this.tick });
    if (this.recentIngestions.length > 20) this.recentIngestions.shift();

    this.homeostaticHealth = Math.max(
      0.22,
      Math.min(0.9, this.homeostaticHealth + (strength - 0.5) * 0.2)
    );
    this.stabilitySignal = Math.max(
      0.22,
      Math.min(0.9, this.stabilitySignal + (strength - 0.5) * 0.15)
    );
    this.domains.informationProcessing = Math.min(
      1.45,
      this.domains.informationProcessing * 1.015
    );
    this.recent_hits = Math.min(5, this.recent_hits + (isFsot ? 1 : 0));

    const generatedSignal = this.generateInternalSignal(inputFeatures);

    this.ingestedSignals.push({
      strength,
      type,
      tick: this.tick,
      decay: 1.0,
      original: text.substring(0, 160),
      linkedSignalId: generatedSignal.id,
    });

    this.observe(0);

    return {
      accepted: true,
      strength: parseFloat(strength.toFixed(3)),
      generatedSignal: generatedSignal.pattern,
      consistency: generatedSignal.consistencyScore,
      S: this.S,
      regime: this.regime,
      homeostaticHealth: this.homeostaticHealth,
      stabilitySignal: this.stabilitySignal,
    };
  }

  regulateDomains() {
    const c = this.coherence;
    const h = this.homeostaticHealth;
    const r = this.rewardSignal;
    const consistencyBonus = Math.min(0.12, (this.recentConsistency - 1) * 0.035);
    const pressureBonus = this.vitalityPressure * 0.08;
    const efficiencyBonus = this.adaptiveEfficiency * 0.06;

    for (const rule of this.domainRegulationLogic) {
      if (rule.condition === 'highCoherence' && c > 0.55 && h > 0.44) {
        if (this.domains[rule.target] !== undefined) {
          this.domains[rule.target] = Math.min(1.55, this.domains[rule.target] * (1 + rule.boost));
        }
      }
      if (rule.condition === 'highPressure' && this.vitalityPressure > 0.35) {
        if (this.domains[rule.target] !== undefined) {
          this.domains[rule.target] = Math.min(
            1.5,
            this.domains[rule.target] * (1 + rule.boost * 0.6)
          );
        }
      }
      if (rule.condition === 'lowVitality' && this.vitality < 0.1) {
        if (this.domains[rule.target] !== undefined) {
          this.domains[rule.target] = Math.min(1.6, this.domains[rule.target] * rule.boost);
        }
      }
    }

    if (c > 0.55 && h > 0.44) {
      const boost = 1 + r * 0.018 + consistencyBonus + pressureBonus + efficiencyBonus;
      this.domains.selfModification = Math.min(1.55, this.domains.selfModification * boost);
      this.domains.domainCoupling = Math.min(
        1.42,
        this.domains.domainCoupling * (1 + r * 0.012 + consistencyBonus * 0.4)
      );
      this.domains.efficiency = Math.min(
        1.4,
        this.domains.efficiency * (1 + pressureBonus * 0.5 + efficiencyBonus * 0.3)
      );
    } else if (c < 0.34 || h < 0.34) {
      this.domains.selfModification = Math.max(0.36, this.domains.selfModification * 0.985);
      this.domains.domainCoupling = Math.max(0.42, this.domains.domainCoupling * 0.99);
    }
  }

  /**
   * Propose mutation, evaluate short horizon, keep only if fitness improves.
   */
  selfModify() {
    if (!this.shouldSelfModify()) return;

    const before = this.fitnessSnapshot();
    const snapshot = {
      params: deepClone(this.params),
      selfModificationPolicy: deepClone(this.selfModificationPolicy),
      signalGenerationLogic: deepClone(this.signalGenerationLogic),
      domainRegulationLogic: deepClone(this.domainRegulationLogic),
      recallPolicy: deepClone(this.recallPolicy),
    };

    const consistencyMod = 1 + (this.recentConsistency - 2) * 0.06;
    const pressureMod = 1 + this.vitalityPressure * 0.5;
    const efficiencyMod = 1 + (this.adaptiveEfficiency - 0.5) * 0.4;
    const rate = this.params.mutationRate;
    const mutationScale = pressureMod > 1.2 ? 0.65 : 1.0;
    const scale = rate * consistencyMod * pressureMod * efficiencyMod * mutationScale;

    this.params.D_eff += (Math.random() - 0.5) * scale * 4.5;
    this.params.emergencePreference += (Math.random() - 0.5) * scale * 0.65;
    this.params.delta_psi += (Math.random() - 0.5) * scale * 0.4;
    this.params.D_eff = Math.max(6, Math.min(28, this.params.D_eff));
    this.params.emergencePreference = Math.max(0.42, Math.min(0.82, this.params.emergencePreference));
    this.params.delta_psi = Math.max(0.05, Math.min(1.5, this.params.delta_psi));

    let structuralKind = null;
    if (this.vitalityPressure > 0.32 && Math.random() < 0.12) {
      this.mutateSelfModificationPolicy();
      structuralKind = 'selfModificationPolicy';
    }
    if (this.vitalityPressure > 0.35 && Math.random() < 0.07) {
      this.mutateSignalGenerationLogic();
      structuralKind = structuralKind || 'signalGenerationLogic';
    }
    if (this.vitalityPressure > 0.38 && Math.random() < 0.06) {
      this.mutateDomainRegulationLogic();
      structuralKind = structuralKind || 'domainRegulationLogic';
    }
    if (
      (this.vitalityPressure > 0.3 || (this.cognitiveAccuracy || 0) < 0.5) &&
      Math.random() < 0.1
    ) {
      this.mutateRecallPolicy();
      structuralKind = structuralKind || 'recallPolicy';
    }

    // Recompute at same phase so selection is apples-to-apples
    this.computeScalar(0, { freezePhase: true });
    const after = this.fitnessSnapshot();
    const improved = after.score > before.score + 1e-6;

    if (!improved) {
      this.params = snapshot.params;
      this.selfModificationPolicy = snapshot.selfModificationPolicy;
      this.signalGenerationLogic = snapshot.signalGenerationLogic;
      this.domainRegulationLogic = snapshot.domainRegulationLogic;
      this.recallPolicy = snapshot.recallPolicy;
      this.computeScalar(0, { freezePhase: true });
      this.rejectedMutations++;
      this.mutationLog.push({
        t: this.tick,
        accepted: false,
        kind: structuralKind || 'params',
        before: before.score,
        after: after.score,
      });
    } else {
      this.acceptedMutations++;
      this.mutationLog.push({
        t: this.tick,
        accepted: true,
        kind: structuralKind || 'params',
        before: before.score,
        after: after.score,
      });
    }
    if (this.mutationLog.length > 80) this.mutationLog.shift();
  }

  mutateSelfModificationPolicy() {
    const mutationType = Math.floor(Math.random() * 3);
    if (mutationType === 0 && this.selfModificationPolicy.length < 7) {
      const newRuleTypes = ['pressureThreshold', 'consistencyBonus', 'vitalityFloor'];
      const newType = newRuleTypes[Math.floor(Math.random() * newRuleTypes.length)];
      this.selfModificationPolicy.push({
        type: newType,
        value: 0.3 + Math.random() * 0.4,
        strength: 0.6 + Math.random() * 0.8,
      });
    } else if (mutationType === 1 && this.selfModificationPolicy.length > 1) {
      this.selfModificationPolicy.splice(
        Math.floor(Math.random() * this.selfModificationPolicy.length),
        1
      );
    } else if (this.selfModificationPolicy.length > 0) {
      const rule =
        this.selfModificationPolicy[
          Math.floor(Math.random() * this.selfModificationPolicy.length)
        ];
      rule.value += (Math.random() - 0.5) * 0.15;
      rule.strength += (Math.random() - 0.5) * 0.2;
      rule.value = Math.max(0.05, Math.min(1.5, rule.value));
      rule.strength = Math.max(0.3, Math.min(2.0, rule.strength));
    }
  }

  mutateSignalGenerationLogic() {
    const mutationType = Math.floor(Math.random() * 3);
    if (mutationType === 0 && this.signalGenerationLogic.length < 6) {
      const featureTypes = ['strength', 'stability', 'coherence'];
      this.signalGenerationLogic.push({
        feature: featureTypes[Math.floor(Math.random() * featureTypes.length)],
        weight: 0.7 + Math.random() * 0.6,
        influence: 0.6 + Math.random() * 0.5,
      });
    } else if (mutationType === 1 && this.signalGenerationLogic.length > 1) {
      this.signalGenerationLogic.splice(
        Math.floor(Math.random() * this.signalGenerationLogic.length),
        1
      );
    } else if (this.signalGenerationLogic.length > 0) {
      const rule =
        this.signalGenerationLogic[
          Math.floor(Math.random() * this.signalGenerationLogic.length)
        ];
      rule.weight += (Math.random() - 0.5) * 0.2;
      rule.influence += (Math.random() - 0.5) * 0.15;
      rule.weight = Math.max(0.4, Math.min(2.0, rule.weight));
      rule.influence = Math.max(0.3, Math.min(1.5, rule.influence));
    }
  }

  mutateDomainRegulationLogic() {
    const mutationType = Math.floor(Math.random() * 3);
    if (mutationType === 0 && this.domainRegulationLogic.length < 6) {
      const conditions = ['highCoherence', 'highPressure', 'lowVitality'];
      const targets = [
        'selfModification',
        'efficiency',
        'coherenceMaintenance',
        'domainCoupling',
        'informationProcessing',
        'memory',
      ];
      this.domainRegulationLogic.push({
        condition: conditions[Math.floor(Math.random() * conditions.length)],
        target: targets[Math.floor(Math.random() * targets.length)],
        boost: 0.3 + Math.random() * 0.8,
      });
    } else if (mutationType === 1 && this.domainRegulationLogic.length > 1) {
      this.domainRegulationLogic.splice(
        Math.floor(Math.random() * this.domainRegulationLogic.length),
        1
      );
    } else if (this.domainRegulationLogic.length > 0) {
      const rule =
        this.domainRegulationLogic[
          Math.floor(Math.random() * this.domainRegulationLogic.length)
        ];
      rule.boost += (Math.random() - 0.5) * 0.25;
      rule.boost = Math.max(0.2, Math.min(1.8, rule.boost));
    }
  }

  mutateRecallPolicy() {
    const mutationType = Math.floor(Math.random() * 3);
    if (mutationType === 0 && this.recallPolicy.length < 8) {
      const features = [
        'recency',
        'strength',
        'typeMatch',
        'tokenOverlap',
        'regimeMatch',
        'Sproximity',
      ];
      this.recallPolicy.push({
        feature: features[Math.floor(Math.random() * features.length)],
        weight: 0.5 + Math.random(),
      });
    } else if (mutationType === 1 && this.recallPolicy.length > 2) {
      this.recallPolicy.splice(Math.floor(Math.random() * this.recallPolicy.length), 1);
    } else if (this.recallPolicy.length > 0) {
      const rule = this.recallPolicy[Math.floor(Math.random() * this.recallPolicy.length)];
      rule.weight = Math.max(0.2, Math.min(2.0, rule.weight + (Math.random() - 0.5) * 0.3));
    }
  }

  updateMemory() {
    if (this.tick % 8 === 0) {
      this.memory.push({
        t: this.tick,
        v: parseFloat(this.vitality.toFixed(4)),
        S: parseFloat(this.S.toFixed(4)),
        raw_S: parseFloat(this.raw_S.toFixed(4)),
        regime: this.regime,
        c: parseFloat(this.coherence.toFixed(4)),
        health: parseFloat(this.homeostaticHealth.toFixed(3)),
        reward: parseFloat(this.rewardSignal.toFixed(3)),
        pressure: parseFloat(this.vitalityPressure.toFixed(3)),
        integration: parseFloat(this.informationIntegration.toFixed(3)),
        distinction: parseFloat(this.patternDistinctionBalance.toFixed(3)),
        adaptive: parseFloat(this.adaptiveEfficiency.toFixed(3)),
        modPolicySize: this.selfModificationPolicy.length,
        signalLogicSize: this.signalGenerationLogic.length,
        domainLogicSize: this.domainRegulationLogic.length,
        recallPolicySize: this.recallPolicy.length,
        cognitive: parseFloat((this.cognitiveAccuracy || 0).toFixed(3)),
        intelligence: parseFloat((this.intelligenceScore || 0).toFixed(3)),
        D_eff: parseFloat(this.params.D_eff.toFixed(2)),
      });
      if (this.memory.length > 80) this.memory.shift();
    }
  }

  tickOnce(environmentStress = 0) {
    this.tick++;
    const stress = environmentStress + Math.sin(this.tick / 27) * 0.2;

    this.internalSelfCheck();

    if (Math.random() < this.params.observerResponsiveness * 0.42) {
      this.observe(stress);
    } else {
      this.observed = false;
      this.computeScalar(stress);
    }

    this.regulateDomains();
    this.selfModify();
    this.updateMemory();

    if (this.tick % 25 === 0) {
      this.history.push({
        tick: this.tick,
        vitality: parseFloat(this.vitality.toFixed(4)),
        S: parseFloat(this.S.toFixed(4)),
        coherence: parseFloat(this.coherence.toFixed(4)),
        regime: this.regime,
        observed: this.observed,
      });
      // Cap history — unbounded growth was crashing long habitat runs (OOM / save blowup)
      if (this.history.length > 200) this.history.splice(0, this.history.length - 200);
    }

    return this.getState();
  }

  getState() {
    return {
      id: this.id,
      version: this.version,
      tick: this.tick,
      S: this.S,
      raw_S: this.raw_S,
      term1: this.term1,
      term2: this.term2,
      term3: this.term3,
      quirk_mod: this.quirk_mod,
      regime: this.regime,
      k: CONSTANTS.k,
      vitality: this.vitality,
      coherence: this.coherence,
      homeostaticHealth: this.homeostaticHealth,
      stabilitySignal: this.stabilitySignal,
      rewardSignal: this.rewardSignal,
      baselineStability: this.baselineStability,
      observed: this.observed,
      domains: { ...this.domains },
      params: { ...this.params },
      activeSignals: this.ingestedSignals.length,
      internalSignals: this.internalSignals.length,
      recentConsistency: this.recentConsistency,
      vitalityPressure: this.vitalityPressure,
      informationIntegration: this.informationIntegration,
      patternDistinctionBalance: this.patternDistinctionBalance,
      adaptiveEfficiency: this.adaptiveEfficiency,
      selfModificationPolicySize: this.selfModificationPolicy.length,
      signalGenerationLogicSize: this.signalGenerationLogic.length,
      domainRegulationLogicSize: this.domainRegulationLogic.length,
      selfModificationPolicy: [...this.selfModificationPolicy],
      signalGenerationLogic: [...this.signalGenerationLogic],
      domainRegulationLogic: [...this.domainRegulationLogic],
      acceptedMutations: this.acceptedMutations,
      rejectedMutations: this.rejectedMutations,
      lowStateCounter: this.lowStateCounter,
      recallPolicySize: this.recallPolicy.length,
      recallPolicy: [...this.recallPolicy],
      workingMemorySize: (this.workingMemory || []).length,
      cognitiveAccuracy: this.cognitiveAccuracy || 0,
      intelligenceScore: this.intelligenceScore || 0,
    };
  }

  statePath() {
    ensureStateDir();
    return path.join(STATE_DIR, `${this.id}.json`);
  }

  saveState() {
    const data = {
      id: this.id,
      version: this.version,
      tick: this.tick,
      S: this.S,
      raw_S: this.raw_S,
      vitality: this.vitality,
      coherence: this.coherence,
      homeostaticHealth: this.homeostaticHealth,
      stabilitySignal: this.stabilitySignal,
      rewardSignal: this.rewardSignal,
      baselineStability: this.baselineStability,
      observed: this.observed,
      observationCount: this.observationCount,
      recent_hits: this.recent_hits,
      targetS: this.targetS,
      domains: this.domains,
      params: this.params,
      history: this.history,
      memory: this.memory,
      ingestedSignals: this.ingestedSignals,
      internalSignals: this.internalSignals,
      selfModificationPolicy: this.selfModificationPolicy,
      signalGenerationLogic: this.signalGenerationLogic,
      domainRegulationLogic: this.domainRegulationLogic,
      recallPolicy: this.recallPolicy,
      workingMemory: (this.workingMemory || []).slice(-80),
      _codonTable: this._codonTable,
      cognitiveAccuracy: this.cognitiveAccuracy,
      intelligenceScore: this.intelligenceScore,
      acceptedMutations: this.acceptedMutations,
      rejectedMutations: this.rejectedMutations,
      mutationLog: this.mutationLog,
      informationIntegration: this.informationIntegration,
      patternDistinctionBalance: this.patternDistinctionBalance,
      adaptiveEfficiency: this.adaptiveEfficiency,
      crossCategoryGeneralization: this.crossCategoryGeneralization,
      vitalityPressure: this.vitalityPressure,
      recentConsistency: this.recentConsistency,
      phase: this.phase,
    };
    fs.writeFileSync(this.statePath(), JSON.stringify(data, null, 2));
    return this.statePath();
  }

  loadState() {
    const filePath = this.statePath();
    if (fs.existsSync(filePath)) {
      try {
        const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
        Object.assign(this, data);
      } catch (_) {
        // fresh start if corrupt
      }
    }
  }
}

module.exports = { FSOTLivingScalar, STATE_DIR, CONSTANTS };
