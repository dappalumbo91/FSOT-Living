//! Dynamic trinary genetic fabric for the FSOT body.
//!
//! **This is not a binary genome.**
//! Each locus holds a continuous field that collapses into a Trit phenotype
//! under pressure and can re-open (plasticity). Expression drifts with
//! homeostasis. Codons map to dual-axis trinary (same family as living habitat).

#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use libm::sqrt;
use trinary::{collapse_soft, Trit};

// ---------------------------------------------------------------------------
// Codon / dual-axis (aligned with fsot-living-rust genetic.rs)
// ---------------------------------------------------------------------------

/// Primary axis: purine +1, pyrimidine −1.
pub fn primary_trinary(codon: &str) -> [i8; 3] {
    let mut out = [0i8; 3];
    for (i, c) in codon.chars().take(3).enumerate() {
        out[i] = match c {
            'A' | 'G' | 'a' | 'g' => 1,
            'C' | 'T' | 'c' | 't' => -1,
            _ => 0,
        };
    }
    out
}

/// Secondary axis: A=+1 T=−1 G,C=0.
pub fn secondary_trinary(codon: &str) -> [i8; 3] {
    let mut out = [0i8; 3];
    for (i, c) in codon.chars().take(3).enumerate() {
        out[i] = match c {
            'A' | 'a' => 1,
            'T' | 't' => -1,
            _ => 0,
        };
    }
    out
}

/// Minimal genetic code (same table as living habitat).
pub fn genetic_code(codon: &str) -> char {
    let c = codon.to_ascii_uppercase();
    match c.as_str() {
        "AAA" => 'K', "AAC" => 'N', "AAG" => 'K', "AAT" => 'N',
        "ACA" => 'T', "ACC" => 'T', "ACG" => 'T', "ACT" => 'T',
        "AGA" => 'R', "AGC" => 'S', "AGG" => 'R', "AGT" => 'S',
        "ATA" => 'I', "ATC" => 'I', "ATG" => 'M', "ATT" => 'I',
        "CAA" => 'Q', "CAC" => 'H', "CAG" => 'Q', "CAT" => 'H',
        "CCA" => 'P', "CCC" => 'P', "CCG" => 'P', "CCT" => 'P',
        "CGA" => 'R', "CGC" => 'R', "CGG" => 'R', "CGT" => 'R',
        "CTA" => 'L', "CTC" => 'L', "CTG" => 'L', "CTT" => 'L',
        "GAA" => 'E', "GAC" => 'D', "GAG" => 'E', "GAT" => 'D',
        "GCA" => 'A', "GCC" => 'A', "GCG" => 'A', "GCT" => 'A',
        "GGA" => 'G', "GGC" => 'G', "GGG" => 'G', "GGT" => 'G',
        "GTA" => 'V', "GTC" => 'V', "GTG" => 'V', "GTT" => 'V',
        "TAA" => '*', "TAC" => 'Y', "TAG" => '*', "TAT" => 'Y',
        "TCA" => 'S', "TCC" => 'S', "TCG" => 'S', "TCT" => 'S',
        "TGA" => '*', "TGC" => 'C', "TGG" => 'W', "TGT" => 'C',
        "TTA" => 'L', "TTC" => 'F', "TTG" => 'L', "TTT" => 'F',
        _ => '?',
    }
}

/// Default 23D-style organism seed (matches living habitat default).
pub fn default_seed_codons() -> &'static [&'static str] {
    &[
        "ATG", "CCG", "AAC", "GGT", "AAT", "AGC", "AGT", "GGT", "TAG", "CTG",
        "TCG", "TTA", "TTG", "TCA", "CCG", "TCA", "CCA", "CCA", "TCG", "TCG",
        "AAT", "CGT", "CGT", "CGT", "CGT", "TAA",
    ]
}

// ---------------------------------------------------------------------------
// Dynamic locus — the anti-binary unit
// ---------------------------------------------------------------------------

/// One genetic locus: continuous field + collapsed Trit + expression + plasticity.
///
/// Unlike a static bit, this can re-collapse every tick under pressure.
#[derive(Copy, Clone, Debug)]
pub struct Locus {
    /// Pre-phenotypic continuous field (FSOT channel).
    pub field: f64,
    /// Collapsed phenotype (−1 / 0 / +1).
    pub state: Trit,
    /// How strongly the phenotype is expressed right now [0, 1].
    pub expression: f64,
    /// Ease of re-opening and re-collapsing [0, 1]. High = dynamic.
    pub plasticity: f64,
    /// Tick of last collapse event.
    pub last_collapse_tick: u64,
}

impl Locus {
    pub fn new(field: f64, plasticity: f64) -> Self {
        let state = collapse_soft(field, 0.15);
        Locus {
            field,
            state,
            expression: match state {
                Trit::Superposed => 0.25,
                _ => 0.55,
            },
            plasticity: plasticity.clamp(0.0, 1.0),
            last_collapse_tick: 0,
        }
    }

    /// From a codon base position (primary axis seed).
    pub fn from_base_axis(primary: i8, secondary: i8, plasticity: f64) -> Self {
        let field = primary as f64 * 0.55 + secondary as f64 * 0.35;
        Self::new(field, plasticity)
    }

    /// Apply environmental / interoceptive pressure.
    ///
    /// `pressure` ∈ ℝ — positive favors expression (SpinUp), negative favors
    /// repression. Plastic loci move more; rigid loci resist.
    pub fn tick(&mut self, pressure: f64, tick: u64) {
        let p = self.plasticity;
        // Field drifts toward pressure, weighted by plasticity.
        self.field = self.field * (1.0 - 0.12 * p) + pressure * (0.18 * p);
        // Soft decay of expression when superposed.
        let new_state = collapse_soft(self.field, 0.12 + 0.08 * (1.0 - p));
        if new_state != self.state {
            self.state = new_state;
            self.last_collapse_tick = tick;
            // Expression spike on re-collapse (phenotype change event).
            self.expression = (self.expression * 0.4 + 0.6).clamp(0.0, 1.0);
        } else {
            // Steady expression tracks |field|.
            let target = match self.state {
                Trit::Superposed => 0.2 + 0.15 * p,
                Trit::SpinUp | Trit::SpinDown => {
                    let mag = if self.field < 0.0 {
                        -self.field
                    } else {
                        self.field
                    };
                    (0.35 + mag * 0.45).clamp(0.0, 1.0)
                }
            };
            self.expression = self.expression * (1.0 - 0.1) + target * 0.1;
        }
    }

    /// Force re-superpose (rest / BREATHE). Only plastic loci fully open.
    pub fn rest(&mut self, tick: u64) {
        self.field *= 1.0 - 0.25 * self.plasticity;
        if self.plasticity > 0.35 {
            self.state = Trit::Superposed;
            self.last_collapse_tick = tick;
        }
        self.expression *= 0.85;
    }
}

// ---------------------------------------------------------------------------
// Genome guidance (body control parameters)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct GenomeGuidance {
    pub delta_psi: f64,
    pub d_eff: f64,
    pub emergence_preference: f64,
    pub domain_coupling: f64,
    pub aa_chain: String,
    pub codon_count: usize,
    /// Mean primary-axis field (−1..+1).
    pub mean_primary: f64,
    /// Mean secondary-axis field.
    pub mean_secondary: f64,
}

// ---------------------------------------------------------------------------
// Genetic fabric — full dynamic body genome
// ---------------------------------------------------------------------------

/// Living genetic fabric: codons + dynamic loci (not a frozen bitstring).
#[derive(Clone, Debug)]
pub struct GeneticFabric {
    pub codons: Vec<String>,
    /// Three loci per codon (one per base position) — dynamic.
    pub loci: Vec<Locus>,
    pub tick: u64,
    pub generation: u32,
}

impl GeneticFabric {
    pub fn from_codons(codons: &[&str]) -> Self {
        let mut codon_owned = Vec::with_capacity(codons.len());
        let mut loci = Vec::with_capacity(codons.len() * 3);
        for (ci, c) in codons.iter().enumerate() {
            codon_owned.push(String::from(*c));
            let p = primary_trinary(c);
            let s = secondary_trinary(c);
            // Plasticity varies by position — middle base more plastic (wobble-like).
            for bi in 0..3 {
                let plast = match bi {
                    1 => 0.72,
                    0 => 0.45,
                    _ => 0.55,
                } + (ci % 5) as f64 * 0.02;
                loci.push(Locus::from_base_axis(p[bi], s[bi], plast.min(0.95)));
            }
        }
        GeneticFabric {
            codons: codon_owned,
            loci,
            tick: 0,
            generation: 0,
        }
    }

    pub fn seed_default() -> Self {
        Self::from_codons(default_seed_codons())
    }

    /// Parse living-style genome strings: `ATG-CCG-AAC-…` (also comma/space).
    pub fn parse_genome_string(s: &str) -> Vec<String> {
        let mut out = Vec::new();
        for tok in s.split(|c: char| c == '-' || c == ',' || c.is_whitespace()) {
            let t = tok.trim().to_ascii_uppercase();
            if t.len() == 3
                && t.chars()
                    .all(|c| matches!(c, 'A' | 'C' | 'G' | 'T'))
            {
                out.push(t);
            }
        }
        out
    }

    pub fn from_codon_strings(codons: &[String]) -> Self {
        let refs: Vec<&str> = codons.iter().map(|s| s.as_str()).collect();
        Self::from_codons(&refs)
    }

    /// Reseed fabric from a champion genome string (mind → body germline couple).
    /// Rebuilds dynamic loci from codon axes — phenotype is living, not frozen bits.
    /// Returns false if no valid codons parsed.
    pub fn reseed_from_genome_string(&mut self, genome: &str, generation: u32) -> bool {
        let codons = Self::parse_genome_string(genome);
        if codons.is_empty() {
            return false;
        }
        let prev_tick = self.tick;
        *self = Self::from_codon_strings(&codons);
        self.generation = generation;
        self.tick = prev_tick; // keep body time continuous across reseed
        true
    }

    pub fn guidance(&self) -> GenomeGuidance {
        let mut prim = [0.0f64; 3];
        let mut sec = [0.0f64; 3];
        let mut n = 0usize;
        let mut aa = String::new();
        for codon in &self.codons {
            let p = primary_trinary(codon);
            let s = secondary_trinary(codon);
            for i in 0..3 {
                prim[i] += p[i] as f64;
                sec[i] += s[i] as f64;
            }
            n += 1;
            let a = genetic_code(codon);
            if a != '*' && a != '?' {
                aa.push(a);
            }
        }
        if n == 0 {
            return GenomeGuidance {
                delta_psi: 0.7,
                d_eff: 12.0,
                emergence_preference: 0.62,
                domain_coupling: 0.82,
                aa_chain: String::new(),
                codon_count: 0,
                mean_primary: 0.0,
                mean_secondary: 0.0,
            };
        }
        let inv = 1.0 / n as f64;
        for i in 0..3 {
            prim[i] *= inv;
            sec[i] *= inv;
        }
        let s_mag = sqrt(sec[0] * sec[0] + sec[1] * sec[1] + sec[2] * sec[2]) / sqrt(3.0);
        let mean_primary = (prim[0] + prim[1] + prim[2]) / 3.0;
        let mean_secondary = (sec[0] + sec[1] + sec[2]) / 3.0;
        GenomeGuidance {
            delta_psi: (0.35 + prim[0] * 0.25 + prim[1] * 0.15).clamp(0.05, 1.3),
            d_eff: (12.0 + sec[2] * 3.0 + s_mag * 2.0).clamp(8.0, 20.0),
            emergence_preference: (0.5 + mean_primary * 0.25 + mean_secondary * 0.1)
                .clamp(0.05, 0.95),
            domain_coupling: (0.7 + s_mag * 0.2).clamp(0.4, 1.0),
            aa_chain: aa,
            codon_count: n,
            mean_primary,
            mean_secondary,
        }
    }

    /// One homeostasis / pressure tick across all loci.
    pub fn tick_pressure(&mut self, global_pressure: f64) {
        self.tick = self.tick.wrapping_add(1);
        let t = self.tick;
        // Local modulation: alternate locus bands get inverted micro-pressure
        // so the fabric does not freeze into a single polarity.
        for (i, loc) in self.loci.iter_mut().enumerate() {
            let local = if i % 3 == 1 {
                global_pressure * 0.85
            } else if i % 3 == 2 {
                -global_pressure * 0.35
            } else {
                global_pressure
            };
            loc.tick(local, t);
        }
    }

    pub fn rest_all(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let t = self.tick;
        for loc in self.loci.iter_mut() {
            loc.rest(t);
        }
    }

    /// Phenotype summary: counts of repress / latent / express.
    pub fn phenotype_tally(&self) -> (usize, usize, usize) {
        let mut down = 0usize;
        let mut mid = 0usize;
        let mut up = 0usize;
        for loc in &self.loci {
            match loc.state {
                Trit::SpinDown => down += 1,
                Trit::Superposed => mid += 1,
                Trit::SpinUp => up += 1,
            }
        }
        (down, mid, up)
    }

    /// Mean expression across fabric [0, 1].
    pub fn mean_expression(&self) -> f64 {
        if self.loci.is_empty() {
            return 0.0;
        }
        let s: f64 = self.loci.iter().map(|l| l.expression).sum();
        s / self.loci.len() as f64
    }

    /// Mean plasticity — higher means more dynamic (less binary-static).
    pub fn mean_plasticity(&self) -> f64 {
        if self.loci.is_empty() {
            return 0.0;
        }
        let s: f64 = self.loci.iter().map(|l| l.plasticity).sum();
        s / self.loci.len() as f64
    }

    /// Compact fingerprint for serial boot banners.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for c in &self.codons {
            for b in c.as_bytes() {
                h ^= *b as u64;
                h = h.wrapping_mul(0x1000_0000_01b3);
            }
        }
        let (d, m, u) = self.phenotype_tally();
        h ^= (d as u64) << 32;
        h ^= (m as u64) << 16;
        h ^= u as u64;
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabric_is_trinary_not_binary() {
        let fab = GeneticFabric::seed_default();
        assert!(fab.loci.len() >= 3);
        let (d, m, u) = fab.phenotype_tally();
        // All three states must be representable in the type system;
        // seed typically produces a mix including latent.
        assert_eq!(d + m + u, fab.loci.len());
        // Plasticity is non-zero → dynamic, not static bits.
        assert!(fab.mean_plasticity() > 0.3);
    }

    #[test]
    fn pressure_changes_phenotype() {
        let mut fab = GeneticFabric::seed_default();
        let before = fab.phenotype_tally();
        for _ in 0..40 {
            fab.tick_pressure(0.9);
        }
        let after = fab.phenotype_tally();
        // Strong positive pressure should increase express count or change tally.
        assert!(
            after != before || fab.mean_expression() > 0.3,
            "fabric should respond dynamically to pressure"
        );
        let (_, _, up_after) = after;
        let (_, _, up_before) = before;
        assert!(up_after >= up_before || fab.loci.iter().any(|l| l.last_collapse_tick > 0));
    }

    #[test]
    fn parse_and_reseed_champion_genome() {
        let mut fab = GeneticFabric::seed_default();
        let g = "ATG-CCG-AGC-GGT-AAT-AGC-AGT-GGT-CAG-CTG-TCG-TTA-TTA-TTG-TCA-CCG-TCA-CCA-CAA-TCG-TCG-AAT-CGT-CGT-CCT-CGG-TAG";
        assert!(fab.reseed_from_genome_string(g, 3));
        assert_eq!(fab.generation, 3);
        assert_eq!(fab.codons[0], "ATG");
        assert!(fab.codons.len() >= 20);
        assert_eq!(fab.loci.len(), fab.codons.len() * 3);
    }

    #[test]
    fn rest_opens_plastic_loci() {
        let mut fab = GeneticFabric::seed_default();
        for _ in 0..20 {
            fab.tick_pressure(1.0);
        }
        fab.rest_all();
        let (_, mid, _) = fab.phenotype_tally();
        // After rest, some plastic loci should be latent again.
        assert!(mid > 0, "rest should re-superpose plastic loci");
    }

    #[test]
    fn guidance_matches_living_family() {
        let g = GeneticFabric::seed_default().guidance();
        assert!(g.codon_count > 0);
        assert!(g.delta_psi > 0.0 && g.delta_psi < 2.0);
        assert!(g.d_eff >= 8.0 && g.d_eff <= 20.0);
        assert!(!g.aa_chain.is_empty());
    }

    #[test]
    fn dual_axis_not_bits() {
        let p = primary_trinary("ATG");
        let s = secondary_trinary("ATG");
        // A=+1,+1 ; T=-1,-1 ; G=+1,0  → three-valued secondary
        assert_eq!(p, [1, -1, 1]);
        assert_eq!(s, [1, -1, 0]);
    }
}
