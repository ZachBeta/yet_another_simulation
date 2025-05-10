use sim_core::config::Config;
use sim_core::neat::config::EvolutionConfig;
use sim_core::neat::population::Population;

fn main() {
    // Simulation config
    let sim_cfg = Config::default();
    // NEAT evolution config (override for quick test)
    let mut evo_cfg = EvolutionConfig::default();
    evo_cfg.pop_size = 10;
    evo_cfg.tournament_k = 2;
    evo_cfg.max_ticks = 200;
    evo_cfg.num_teams = 2;
    evo_cfg.team_size = 1;

    // Number of generations to run
    let max_gens = 10;
    // Initialize population
    let mut population = Population::new(&evo_cfg);
    for gen in 0..max_gens {
        println!("--- Generation {} ---", gen);
        // Evaluate and log stats
        population.evaluate(&sim_cfg, &evo_cfg);
        let fitnesses: Vec<f32> = population.genomes.iter().map(|g| g.fitness).collect();
        let best = *fitnesses.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;
        println!("Gen {}: best = {:.2}, avg = {:.2}", gen, best, avg);
        // Hall of Fame
        println!("Hall of Fame (top {}):", evo_cfg.hof_size);
        for (i, g) in population.hof.iter().enumerate() {
            println!("  HoF {}: {:.2}", i, g.fitness);
        }
        // Reproduce for next generation
        if gen < max_gens - 1 {
            population.reproduce(&evo_cfg);
        }
    }
}
