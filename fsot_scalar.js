/**
 * FSOT 2.1 Scalar Engine
 * Port of FSOT/Scalar.lean from https://github.com/dappalumbo91/FSOT-2.1-Lean
 * Damian Arthur Palumbo — Fluid Spacetime Omni-Theory
 */

'use strict';

// ============================================================
// FUNDAMENTAL & DERIVED CONSTANTS
// ============================================================

const phi = (1.0 + Math.sqrt(5.0)) / 2.0;
const e = Math.exp(1.0);
const pi = Math.PI;
const sqrt2 = Math.sqrt(2.0);
const log2 = Math.log(2.0);
const gamma_euler = 0.5772156649015329;
const catalan_G = 0.9159655941772190;

const alpha = Math.log(pi) / (e * Math.pow(phi, 13));
const psi_con = (e - 1.0) / e;
const eta_eff = 1.0 / (pi - 1.0);
const beta = 1.0 / Math.exp(Math.pow(pi, pi) + (e - 1.0));
const gamma = -log2 / phi;
const omega = Math.sin(pi / e) * sqrt2;
const theta_s = Math.sin(psi_con * eta_eff);
const poof_factor = Math.exp(-(Math.log(pi) / e) / (eta_eff * Math.log(phi)));
const acoustic_bleed = Math.sin(pi / e) * phi / sqrt2;
const phase_variance = -Math.cos(theta_s + pi);
const coherence_efficiency =
  (1.0 - poof_factor * Math.sin(theta_s)) *
  (1.0 + 0.01 * catalan_G / (pi * phi));
const bleed_in_factor = coherence_efficiency * (1.0 - Math.sin(theta_s) / phi);
const acoustic_inflow = acoustic_bleed * (1.0 + Math.cos(theta_s) / phi);
const suction_factor = poof_factor * (-Math.cos(theta_s - pi));
const chaos_factor = gamma / omega;
const perceived_param_base = gamma_euler / e;
const new_perceived_param = perceived_param_base * sqrt2;
const consciousness_factor = coherence_efficiency * new_perceived_param;
const k = (phi * (perceived_param_base * sqrt2) / Math.log(pi)) * 0.99;

// ============================================================
// INTERNAL TERMS
// ============================================================

function growth_term(N, recent_hits) {
  const Nf = N;
  const rhf = recent_hits;
  return Math.exp(alpha * (1.0 - rhf / Nf) * gamma_euler / phi);
}

function quirk_mod(observed, delta_psi) {
  if (observed) {
    return Math.exp(consciousness_factor * phase_variance) *
      Math.cos(delta_psi + phase_variance);
  }
  return 1.0;
}

function perceived_adjust(D_eff) {
  return 1.0 + new_perceived_param * Math.log(D_eff / 25.0);
}

function term3(N, P, D_eff, delta_psi, delta_theta) {
  const Deff = D_eff;
  return beta * Math.cos(delta_psi) *
    (N * P / Math.sqrt(Deff)) *
    (1.0 + chaos_factor * (Deff - 25.0) / 25.0) *
    (1.0 + poof_factor * Math.cos(theta_s + pi) + suction_factor * Math.sin(theta_s)) *
    (1.0 + acoustic_bleed * Math.pow(Math.sin(delta_theta), 2) / phi +
      acoustic_inflow * Math.pow(Math.cos(delta_theta), 2) / phi) *
    (1.0 + bleed_in_factor * phase_variance);
}

// ============================================================
// CORE COMPUTATION
// ============================================================

/**
 * Compute raw_S and scaled S = k * raw_S with term breakdown.
 * Matches FSOT.Scalar.compute_S_D_chaotic / compute_raw_S_D_chaotic.
 */
function computeS(opts = {}) {
  const N = opts.N ?? 1;
  const P = opts.P ?? 1;
  const D_eff = opts.D_eff ?? 25;
  const recent_hits = opts.recent_hits ?? 0;
  const delta_psi = opts.delta_psi ?? 1.0;
  const delta_theta = opts.delta_theta ?? 1.0;
  const rho = opts.rho ?? 1.0;
  const scale = opts.scale ?? 1.0;
  const amplitude = opts.amplitude ?? 1.0;
  const trend_bias = opts.trend_bias ?? 0.0;
  const observed = opts.observed ?? false;

  const gt = growth_term(N, recent_hits);
  const term1_base =
    (N * P / Math.sqrt(D_eff)) *
    Math.cos((psi_con + delta_psi) / eta_eff) *
    Math.exp(-alpha * recent_hits / N + rho + bleed_in_factor * delta_psi) *
    (1.0 + gt * coherence_efficiency);

  const pa = perceived_adjust(D_eff);
  const qm = quirk_mod(observed, delta_psi);
  const t1 = term1_base * pa * qm;
  const t2 = scale * amplitude + trend_bias;
  const t3 = term3(N, P, D_eff, delta_psi, delta_theta);
  const raw = t1 + t2 + t3;
  const S = raw * k;

  return {
    S,
    raw_S: raw,
    term1: t1,
    term2: t2,
    term3: t3,
    term1_base,
    perceived_adjust: pa,
    quirk_mod: qm,
    growth_term: gt,
    k,
    D_eff,
    delta_psi,
    delta_theta,
    observed,
    recent_hits,
    regime: S > 0 ? 'emergence' : S < 0 ? 'damping' : 'neutral',
  };
}

// Domain ledger (aligned with FSOT/Scalar.lean)
const DOMAIN_PARAMS = {
  quantum:         { D_eff: 6,  recent_hits: 0, delta_psi: 1.0,  delta_theta: 1.0, observed: true },
  biological:      { D_eff: 12, recent_hits: 0, delta_psi: 0.05, delta_theta: 1.0, observed: false },
  astronomical:    { D_eff: 20, recent_hits: 1, delta_psi: 1.0,  delta_theta: 1.0, observed: true },
  cosmological:    { D_eff: 25, recent_hits: 0, delta_psi: 1.0,  delta_theta: 1.0, observed: false },
  ai_tech:         { D_eff: 12, recent_hits: 1, delta_psi: 0.8,  delta_theta: 1.0, observed: true },
  engineering:     { D_eff: 15, recent_hits: 0, delta_psi: 0.5,  delta_theta: 1.0, observed: true },
  physics:         { D_eff: 8,  recent_hits: 0, delta_psi: 1.0,  delta_theta: 1.0, observed: true },
  chemistry:       { D_eff: 10, recent_hits: 0, delta_psi: 0.7,  delta_theta: 1.0, observed: false },
  earth_science:   { D_eff: 18, recent_hits: 1, delta_psi: 0.9,  delta_theta: 1.0, observed: true },
  neuroscience:    { D_eff: 14, recent_hits: 0, delta_psi: 0.1,  delta_theta: 1.0, observed: false },
  economics:       { D_eff: 16, recent_hits: 1, delta_psi: 0.6,  delta_theta: 1.0, observed: true },
  meteorology:     { D_eff: 17, recent_hits: 1, delta_psi: 0.4,  delta_theta: 1.0, observed: true },
  oceanography:    { D_eff: 19, recent_hits: 0, delta_psi: 0.3,  delta_theta: 1.0, observed: false },
  anomalies:       { D_eff: 22, recent_hits: 2, delta_psi: 1.5,  delta_theta: 1.0, observed: true },
  social:          { D_eff: 21, recent_hits: 1, delta_psi: 0.8,  delta_theta: 1.0, observed: true },
  planetary:       { D_eff: 23, recent_hits: 0, delta_psi: 1.0,  delta_theta: 1.0, observed: false },
  nuclear:         { D_eff: 9,  recent_hits: 0, delta_psi: 1.0,  delta_theta: 1.0, observed: true },
  atomic:          { D_eff: 7,  recent_hits: 0, delta_psi: 0.9,  delta_theta: 1.0, observed: true },
  molecular:       { D_eff: 11, recent_hits: 0, delta_psi: 0.6,  delta_theta: 1.0, observed: false },
  electromagnetic: { D_eff: 13, recent_hits: 1, delta_psi: 1.0,  delta_theta: 1.0, observed: true },
};

function getDomainParams(domain) {
  return { ...(DOMAIN_PARAMS[domain] || { D_eff: 25, recent_hits: 0, delta_psi: 1.0, delta_theta: 1.0, observed: false }) };
}

function computeForDomain(domainName, overrides = {}) {
  const base = getDomainParams(domainName);
  return computeS({ ...base, ...overrides });
}

const CONSTANTS = {
  phi, e, pi, sqrt2, log2, gamma_euler, catalan_G,
  alpha, psi_con, eta_eff, beta, gamma, omega, theta_s,
  poof_factor, acoustic_bleed, phase_variance, coherence_efficiency,
  bleed_in_factor, acoustic_inflow, suction_factor, chaos_factor,
  perceived_param_base, new_perceived_param, consciousness_factor, k,
};

module.exports = {
  CONSTANTS,
  growth_term,
  quirk_mod,
  perceived_adjust,
  term3,
  computeS,
  computeForDomain,
  getDomainParams,
  DOMAIN_PARAMS,
};
