/**
 * Structured FSOT seed corpus for organism ingestion.
 * Drawn from FSOT 2.1 Lean domain ledger + verification scope.
 */

'use strict';

const { DOMAIN_PARAMS, computeForDomain, CONSTANTS } = require('./fsot_scalar');

/** Domain ledger facts — real FSOT parameterizations. */
function domainLedgerSeeds() {
  return Object.entries(DOMAIN_PARAMS).map(([name, p]) => {
    const r = computeForDomain(name);
    return {
      text:
        `FSOT domain ${name}: D_eff=${p.D_eff} delta_psi=${p.delta_psi} ` +
        `observed=${p.observed} recent_hits=${p.recent_hits} ` +
        `S=${r.S.toFixed(6)} raw_S=${r.raw_S.toFixed(6)} regime=${r.regime} ` +
        `term1=${r.term1.toFixed(4)} term2=${r.term2.toFixed(4)} term3=${r.term3.toExponential(3)}`,
      type: 'fsot_domain',
      domain: name,
      S: r.S,
      regime: r.regime,
      strength: 0.85 + Math.min(0.15, Math.abs(r.S) * 0.05),
    };
  });
}

/** Core constant / formula seeds. */
function mathSeeds() {
  return [
    {
      text:
        `FSOT universal scale k=${CONSTANTS.k.toFixed(8)} ` +
        `phi=${CONSTANTS.phi.toFixed(8)} psi_con=${CONSTANTS.psi_con.toFixed(8)} ` +
        `eta_eff=${CONSTANTS.eta_eff.toFixed(8)} coherence_efficiency=${CONSTANTS.coherence_efficiency.toFixed(8)}`,
      type: 'fsot_math',
      strength: 0.95,
    },
    {
      text:
        `FSOT S = k * (term1 + term2 + term3); term1 carries growth quirk_mod perceived_adjust; ` +
        `term3 beta-suppressed chaos poof suction acoustic; observer quirk_mod modulates term1`,
      type: 'fsot_math',
      strength: 0.92,
    },
    {
      text:
        `FSOT verified Wave-1: H0≈68.44 km/s/Mpc T_CMB≈2.725 n_s≈0.964 Omega_b_h2≈0.02236 ` +
        `alpha_s=1/(e*pi) r_d≈147.52 Mpc from cosmological and quantum domain scalars`,
      type: 'fsot_cosmology',
      strength: 0.94,
    },
    {
      text:
        `FSOT verification: 65 proved Lean claims 0 sorry 7941 strict-empirical formulas ` +
        `all within 5% 6921 within 2% 306680+ empirical records domain coupling 246 nodes`,
      type: 'fsot_verification',
      strength: 0.96,
    },
    {
      text:
        `FSOT emergence: positive raw_S means emergence regime; negative raw_S means damping. ` +
        `quantum physics nuclear positive; cosmological cmb dark_energy damping at canonical params`,
      type: 'fsot_math',
      strength: 0.9,
    },
    {
      text:
        `FSOT genomic: 64 codons 27 trinary patterns 20 amino acids dual-axis codon map; ` +
        `biological domain D_eff=12 delta_psi=0.05 emergence-positive`,
      type: 'fsot_biology',
      strength: 0.88,
    },
  ];
}

/** Full seed list for a training/ingest pass. */
function allSeeds() {
  return [...mathSeeds(), ...domainLedgerSeeds()];
}

module.exports = { domainLedgerSeeds, mathSeeds, allSeeds };
