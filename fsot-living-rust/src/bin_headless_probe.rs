use std::time::Instant;
mod genetic; mod habitat; mod organism; mod query; mod scalar;
fn main() {
  let mut h = habitat::Habitat::new(7);
  let t0 = Instant::now();
  for _ in 0..400 { h.step(); }
  let hero = h.hero();
  println!("=== AFTER 400 TICKS ===");
  println!("tick={} gen={} fed={} exams={} births={}", h.tick, h.generation, h.fed, h.exams, h.births);
  println!("hero={} S={:.4} regime={} acc={:.1}% intel={:.3} fit={:.3}",
    hero.id, hero.s, hero.regime, hero.cognitive_accuracy*100.0, hero.intelligence, hero.fitness_score());
  println!("codons={}/64 mem={} lt={} mut+/-={}/{}",
    hero.codon_table.len(), hero.memory.len(), hero.long_term.len(), hero.accepted_mut, hero.rejected_mut);
  println!("AA={}", hero.guidance.aa_chain);
  println!("lineage_entries={}", h.lineage.len());
  for L in h.lineage.iter().take(8) {
    println!("  g{} {} acc={:.0}% fit={:.2} cod={}", L.gen, L.champion, L.acc*100.0, L.fit, L.codons);
  }
  let tests = [
    "hello how are you", "status", "codon ATG", "codon TTT", "domain quantum",
    "domain cosmological", "domain biological", "genome", "fit", "help",
    "what is FSOT", "nonsense xyzzy",
  ];
  println!("=== QUERY BATTERY ===");
  for q in tests {
    let r = query::query(hero, q);
    let preview: String = r.answer.chars().take(120).collect();
    println!("Q: {} -> kind={} | {}", q, r.kind, preview.replace('\n', " / "));
  }
  // domain sign check
  let q = scalar::compute_for_domain("quantum");
  let c = scalar::compute_for_domain("cosmological");
  println!("=== SCALAR === qS={:.4} cS={:.4} k={:.6}", q.s, c.s, scalar::constants::k());
  println!("elapsed_ms={}", t0.elapsed().as_millis());
}
