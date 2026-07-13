// FSOT Genetic Organism - Designed to thrive as code
// Built using verified FSOT 2.1 principles (scalar engine, observer effects,
// emergence boundaries, coherence, domains-as-organs)

class FSOTOrganism {
  constructor(genome = null) {
    // Genome directly maps to verified scalar parameters
    this.genome = genome || {
      D_eff: 18 + Math.random() * 7,           // Effective dimensionality (mid-scale sweet spot)
      phase_sensitivity: 0.8 + Math.random() * 0.4,
      mutation_rate: 0.03 + Math.random() * 0.04,
      coherence_bias: 0.9 + Math.random() * 0.2,
      observer_strength: 0.6 + Math.random() * 0.5,
    };

    this.age = 0;
    this.vitality = 0;           // Main scalar-derived "health"
    this.coherence = 0;          // Stability / emergence balance
    this.observed = false;       // Triggers quirk_mod-style modulation
  }

  // Core vitality computation (mirrors verified S_D_chaotic structure)
  computeVitality(environmentStress = 0) {
    const g = this.genome;

    // Simplified but structurally faithful version of the verified scalar
    const base = g.D_eff / 25;
    const phaseTerm = Math.cos((this.age * 0.1) * g.phase_sensitivity);
    const coherenceTerm = g.coherence_bias * (1 - environmentStress * 0.3);

    // Observer effect (quirk_mod style)
    let observerMod = 1;
    if (this.observed) {
      observerMod = Math.exp(0.288 * (this.age % 5) * 0.1) * 
                    Math.cos((this.age * 0.15) + (this.age % 3));
    }

    // Emergence / damping balance (stabilization boundaries)
    const rawVitality = (base * phaseTerm * coherenceTerm * observerMod) + 
                        (g.coherence_bias - 0.5) * 0.4;

    // Apply universal scaling + clamp to healthy range (emergence boundaries)
    this.vitality = Math.max(-0.8, Math.min(1.2, rawVitality * 0.42));
    
    // Coherence as separate but related metric
    this.coherence = Math.max(0, Math.min(1, 
      (this.vitality + 0.6) * g.coherence_bias * (1 - Math.abs(environmentStress) * 0.2)
    ));

    return this.vitality;
  }

  // "Senses" environment or self → activates observer modulation
  observe(stressLevel = 0) {
    this.observed = true;
    this.computeVitality(stressLevel);
    
    // If vitality drops too low while observed, it learns to reduce sensitivity
    if (this.vitality < 0.1) {
      this.genome.observer_strength *= 0.92;
    }
  }

  // Mutation (respects stabilization boundaries)
  mutate() {
    const g = this.genome;
    const rate = g.mutation_rate;

    g.D_eff += (Math.random() - 0.5) * rate * 3;
    g.phase_sensitivity += (Math.random() - 0.5) * rate * 0.5;
    g.coherence_bias += (Math.random() - 0.5) * rate * 0.3;
    g.observer_strength += (Math.random() - 0.5) * rate * 0.4;

    // Clamp to reasonable biological/code ranges (emergence boundaries)
    g.D_eff = Math.max(8, Math.min(28, g.D_eff));
    g.phase_sensitivity = Math.max(0.3, Math.min(2.5, g.phase_sensitivity));
    g.coherence_bias = Math.max(0.6, Math.min(1.4, g.coherence_bias));
    g.observer_strength = Math.max(0.2, Math.min(1.8, g.observer_strength));
  }

  // Reproduction with inheritance + mutation
  reproduce() {
    const childGenome = JSON.parse(JSON.stringify(this.genome));
    
    // Slight inheritance bias toward higher coherence
    childGenome.coherence_bias = Math.max(0.7, 
      childGenome.coherence_bias * (0.95 + Math.random() * 0.1)
    );

    const child = new FSOTOrganism(childGenome);
    child.mutate();
    return child;
  }

  getStatus() {
    return {
      vitality: this.vitality.toFixed(4),
      coherence: this.coherence.toFixed(4),
      observed: this.observed,
      D_eff: this.genome.D_eff.toFixed(2),
      age: this.age
    };
  }
}

// ==================== Evolutionary Test Loop ====================

function runEvolution(generations = 30, populationSize = 12) {
  console.log("=== FSOT Genetic Organism Evolution ===\n");

  let population = Array.from({ length: populationSize }, () => new FSOTOrganism());

  for (let gen = 0; gen < generations; gen++) {
    // Environmental stress increases over time (tests robustness)
    const stress = Math.min(0.8, gen / generations * 0.7);

    // Evaluate
    population.forEach(org => {
      org.age++;
      org.computeVitality(stress);
      
      // Random observation events
      if (Math.random() < 0.35) {
        org.observe(stress * 0.8);
      }
    });

    // Selection: keep top performers (highest vitality + coherence product)
    population.sort((a, b) => 
      (b.vitality * b.coherence) - (a.vitality * a.coherence)
    );

    const survivors = population.slice(0, Math.floor(populationSize * 0.6));

    // Reproduction
    const nextGen = [...survivors];
    while (nextGen.length < populationSize) {
      const parent = survivors[Math.floor(Math.random() * survivors.length)];
      nextGen.push(parent.reproduce());
    }

    population = nextGen;

    // Report every few generations
    if (gen % 5 === 0 || gen === generations - 1) {
      const avgVitality = population.reduce((sum, o) => sum + o.vitality, 0) / population.length;
      const avgCoherence = population.reduce((sum, o) => sum + o.coherence, 0) / population.length;
      
      console.log(`Gen ${gen.toString().padStart(2)} | Avg Vitality: ${avgVitality.toFixed(4)} | Avg Coherence: ${avgCoherence.toFixed(4)} | Best: ${population[0].vitality.toFixed(4)}`);
    }
  }

  console.log("\n=== Final Best Organism ===");
  console.log(population[0].getStatus());
  console.log("Genome:", population[0].genome);
}

runEvolution(40, 14);