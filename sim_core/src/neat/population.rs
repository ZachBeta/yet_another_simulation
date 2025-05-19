use crate::config::Config;
use crate::brain::Brain;
use super::config::EvolutionConfig;
use super::genome::Genome;
use super::runner::run_match;
use super::brain::NeatBrain;
use crate::ai::{NaiveAgent, NaiveBrain};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;

/// A population of genomes and a hall-of-fame
pub struct Population {
    pub genomes: Vec<Genome>,
    pub hof: Vec<Genome>,
}

impl Population {
    /// Initialize population with default genomes
    pub fn new(evo_cfg: &EvolutionConfig) -> Self {
        let genomes = (0..evo_cfg.pop_size)
            .map(|_| Genome::new())
            .collect();
        Population { genomes, hof: Vec::new() }
    }

    /// Evaluate each genome's fitness by running matches
    pub fn evaluate(&mut self, sim_cfg: &Config, evo_cfg: &EvolutionConfig) {
        // Initialize genomes & reset fitness
        for genome in &mut self.genomes {
            if genome.nodes.is_empty() {
                genome.initialize(sim_cfg, evo_cfg);
            }
            genome.fitness = 0.0;
        }
        // Snapshot for opponent sampling
        let snapshot = self.genomes.clone();
        // Round-robin evaluation using Rayon
        let n = snapshot.len();
        self.genomes.par_iter_mut().enumerate().for_each(|(i, genome)| {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let mut agents: Vec<(Box<dyn Brain>, u32)> = Vec::new();
                // subject agent
                agents.push((Box::new(NeatBrain::new(
                    genome.clone(),
                    sim_cfg.batch_size,
                    sim_cfg.python_service_url.clone().unwrap_or_default(),
                )) as Box<dyn Brain>, 0));
                // opponent agent
                agents.push((Box::new(NeatBrain::new(
                    snapshot[j].clone(),
                    sim_cfg.batch_size,
                    sim_cfg.python_service_url.clone().unwrap_or_default(),
                )) as Box<dyn Brain>, 1));
                let stats = run_match(sim_cfg, evo_cfg, agents);
                let fit = evo_cfg.fitness_fn.compute(&stats, &evo_cfg);
                genome.fitness += fit;
            }
            // normalize fitness
            genome.fitness /= (n - 1) as f32;
        });
        // NaiveAgent baseline evaluation
        for genome in &mut self.genomes {
            let naive = NaiveBrain(NaiveAgent::new(1.2, 0.8));
            let mut agents: Vec<(Box<dyn Brain>, u32)> = Vec::new();
            // subject
            agents.push((Box::new(NeatBrain::new(
                genome.clone(),
                sim_cfg.batch_size,
                sim_cfg.python_service_url.clone().unwrap_or_default(),
            )) as Box<dyn Brain>, 0));
            // naive opponent
            agents.push((Box::new(naive) as Box<dyn Brain>, 1));
            let stats = run_match(sim_cfg, evo_cfg, agents);
            genome.fitness_naive = evo_cfg.fitness_fn.compute(&stats, &evo_cfg);
        }
        // update hall-of-fame
        self.genomes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        self.hof = self.genomes.iter().take(evo_cfg.hof_size).cloned().collect();
    }

    /// Produce next generation via speciation, selection, crossover, and mutation
    pub fn reproduce(&mut self, evo_cfg: &EvolutionConfig) {
        // Elitism: carry over top genomes from hall-of-fame
        let mut next_gen: Vec<Genome> = Vec::with_capacity(evo_cfg.pop_size);
        for g in &self.hof {
            next_gen.push(g.clone());
        }
        let mut rng = thread_rng();
        // Generate offspring until population is full
        while next_gen.len() < evo_cfg.pop_size {
            // Tournament selection for parents
            let mut p1 = self.genomes.choose(&mut rng).unwrap();
            for _ in 1..evo_cfg.tournament_k {
                let cand = self.genomes.choose(&mut rng).unwrap();
                if cand.fitness > p1.fitness { p1 = cand; }
            }
            let mut p2 = self.genomes.choose(&mut rng).unwrap();
            for _ in 1..evo_cfg.tournament_k {
                let cand = self.genomes.choose(&mut rng).unwrap();
                if cand.fitness > p2.fitness { p2 = cand; }
            }
            // Crossover and mutate to produce child
            let mut child = Genome::crossover(p1, p2, evo_cfg);
            child.mutate(evo_cfg);
            next_gen.push(child);
        }
        self.genomes = next_gen;
    }
}
