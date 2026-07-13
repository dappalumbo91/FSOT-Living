//! Genetic code + dual-axis trinary + organism genome guidance

use rand::Rng;
use std::collections::HashMap;

/// Standard genetic code codon → amino acid
pub fn genetic_code(codon: &str) -> char {
    match codon {
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

/// Primary axis: A,G=+1 C,T=-1
pub fn primary_trinary(codon: &str) -> [i8; 3] {
    let b: Vec<i8> = codon
        .chars()
        .map(|c| match c {
            'A' | 'G' => 1,
            'C' | 'T' => -1,
            _ => 0,
        })
        .collect();
    [b[0], b[1], b[2]]
}

/// Secondary axis: A=+1 T=-1 G,C=0
pub fn secondary_trinary(codon: &str) -> [i8; 3] {
    let b: Vec<i8> = codon
        .chars()
        .map(|c| match c {
            'A' => 1,
            'T' => -1,
            _ => 0,
        })
        .collect();
    [b[0], b[1], b[2]]
}

/// FSOT 23D organism default genome
pub fn default_organism_genome() -> Vec<String> {
    "ATG-CCG-AAC-GGT-AAT-AGC-AGT-GGT-TAG-CTG-TCG-TTA-TTG-TCA-CCG-TCA-CCA-CCA-TCG-TCG-AAT-CGT-CGT-CGT-CGT-TAA"
        .split('-')
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone, Debug)]
pub struct GenomeGuidance {
    pub delta_psi: f64,
    pub d_eff: f64,
    pub emergence_preference: f64,
    pub domain_coupling: f64,
    pub aa_chain: String,
    pub codon_count: usize,
}

pub fn genome_to_guidance(sequence: &[String]) -> GenomeGuidance {
    let mut prim = [0.0f64; 3];
    let mut sec = [0.0f64; 3];
    let mut n = 0usize;
    let mut starts = 0i32;
    let mut stops = 0i32;
    let mut aa = String::new();

    for codon in sequence {
        let p = primary_trinary(codon);
        let s = secondary_trinary(codon);
        for i in 0..3 {
            prim[i] += p[i] as f64;
            sec[i] += s[i] as f64;
        }
        n += 1;
        if codon == "ATG" {
            starts += 1;
        }
        let a = genetic_code(codon);
        if a == '*' {
            stops += 1;
        } else if a != '?' {
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
        };
    }

    let inv = 1.0 / n as f64;
    for i in 0..3 {
        prim[i] *= inv;
        sec[i] *= inv;
    }
    let s_mag = (sec[0] * sec[0] + sec[1] * sec[1] + sec[2] * sec[2]).sqrt() / 3f64.sqrt();

    GenomeGuidance {
        delta_psi: (0.35 + prim[0] * 0.25 + prim[1] * 0.15).clamp(0.05, 1.3),
        d_eff: (12.0 + sec[2] * 3.0 + s_mag * 2.0).clamp(8.0, 20.0),
        emergence_preference: (0.55 + s_mag * 0.2).clamp(0.42, 0.82),
        domain_coupling: (0.75 + (starts - stops) as f64 * 0.05 + sec[1] * 0.1).clamp(0.4, 1.4),
        aa_chain: aa,
        codon_count: n,
    }
}

/// Dual-parent crossover — offspring genome mixes high-efficiency lineages.
pub fn crossover_genomes(a: &[String], b: &[String], rng: &mut impl Rng) -> Vec<String> {
    if a.is_empty() {
        return if b.is_empty() {
            default_organism_genome()
        } else {
            b.to_vec()
        };
    }
    if b.is_empty() {
        return a.to_vec();
    }
    let n = a.len().max(b.len());
    let cut = rng.gen_range(1..n);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let src = if i < cut { a } else { b };
        out.push(src[i % src.len()].clone());
    }
    if out.first().map(|c| c.as_str()) != Some("ATG") && rng.gen_bool(0.7) {
        out.insert(0, "ATG".into());
    }
    out
}

pub fn mutate_genome(sequence: &[String], rate: f64, rng: &mut impl Rng) -> Vec<String> {
    let bases = [b'A', b'C', b'G', b'T'];
    let mut next: Vec<String> = sequence
        .iter()
        .map(|codon| {
            if rng.gen::<f64>() > rate {
                return codon.clone();
            }
            let mut arr: Vec<u8> = codon.bytes().collect();
            if arr.len() == 3 {
                let i = rng.gen_range(0..3);
                arr[i] = bases[rng.gen_range(0..4)];
            }
            String::from_utf8(arr).unwrap_or_else(|_| codon.clone())
        })
        .collect();

    if rng.gen::<f64>() < rate * 0.5 && next.len() < 48 {
        let c = format!(
            "{}{}{}",
            bases[rng.gen_range(0..4)] as char,
            bases[rng.gen_range(0..4)] as char,
            bases[rng.gen_range(0..4)] as char
        );
        let idx = rng.gen_range(0..=next.len());
        next.insert(idx, c);
    } else if rng.gen::<f64>() < rate * 0.3 && next.len() > 8 {
        let idx = rng.gen_range(0..next.len());
        next.remove(idx);
    }
    next
}

pub fn all_codons() -> HashMap<String, char> {
    let mut m = HashMap::new();
    for a in ['A', 'C', 'G', 'T'] {
        for b in ['A', 'C', 'G', 'T'] {
            for c in ['A', 'C', 'G', 'T'] {
                let codon = format!("{a}{b}{c}");
                let aa = genetic_code(&codon);
                m.insert(codon, aa);
            }
        }
    }
    m
}
