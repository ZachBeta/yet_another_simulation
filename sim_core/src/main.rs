use sim_core::config::Config;
use sim_core::neat::config::EvolutionConfig;
use sim_core::neat::population::Population;
use sim_core::neat::runner::run_match_record;
use sim_core::Brain;
use sim_core::neat::brain::NeatBrain;
use std::env;
use std::fs;
use std::time::Instant;
use num_cpus;
use rayon::ThreadPoolBuilder;
use std::sync::atomic::Ordering;
use sim_core::neat::brain::{INFER_TIME_NS, INFER_COUNT};
use sim_core::neat::runner::{PHYS_TIME_NS, PHYS_COUNT};

fn main() {
    // Parallelism: configure Rayon thread pool (default: half of CPU cores)
    let args: Vec<String> = env::args().collect();
    let workers = args.windows(2)
        .find(|w| w[0] == "--workers")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or_else(|| num_cpus::get() / 2);
    ThreadPoolBuilder::new().num_threads(workers).build_global().unwrap();
    println!("Using {} worker threads", workers);

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
        let eval_start = Instant::now();
        // Evaluate and log stats
        population.evaluate(&sim_cfg, &evo_cfg);
        let eval_dur = eval_start.elapsed();
        println!(" Evaluation took: {:?}", eval_dur);
        let fitnesses: Vec<f32> = population.genomes.iter().map(|g| g.fitness).collect();
        let best = *fitnesses.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;
        println!("Gen {}: best = {:.2}, avg = {:.2}", gen, best, avg);
        // Hall of Fame
        println!("Hall of Fame (top {}):", evo_cfg.hof_size);
        for (i, g) in population.hof.iter().enumerate() {
            println!("  HoF {}: {:.2}", i, g.fitness);
        }
        // Replay champion vs second-best
        let gen_dir = format!("out/gen_{:03}", gen);
        fs::create_dir_all(&gen_dir).expect("Failed to create output dir");
        if population.hof.len() > 1 {
            let champ = population.hof[0].clone();
            let opp = population.hof[1].clone();
            let agents: Vec<(Box<dyn Brain>, u32)> = vec![
                (Box::new(NeatBrain(champ.clone())) as Box<dyn Brain>, 0),
                (Box::new(NeatBrain(opp.clone())) as Box<dyn Brain>, 1),
            ];
            let path = format!("{}/champ_replay.jsonl", gen_dir);
            let stats = run_match_record(&path, &sim_cfg, &evo_cfg, agents);
            println!("  Replay: ticks = {}, health = {:.2}", stats.ticks, stats.subject_team_health);
        }
        // Reproduce for next generation
        if gen < max_gens - 1 {
            population.reproduce(&evo_cfg);
        }
    }
    // Print cumulative profiling results
    let infer_time = INFER_TIME_NS.load(Ordering::Relaxed);
    let infer_count = INFER_COUNT.load(Ordering::Relaxed);
    let phys_time = PHYS_TIME_NS.load(Ordering::Relaxed);
    let phys_count = PHYS_COUNT.load(Ordering::Relaxed);
    println!("=== Profiling Summary ===");
    println!("Inference: {:.2} ms total over {} calls", infer_time as f64 / 1e6, infer_count);
    println!("Physics:   {:.2} ms total over {} steps", phys_time as f64 / 1e6, phys_count);
}
