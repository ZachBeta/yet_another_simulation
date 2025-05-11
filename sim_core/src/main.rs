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
use sim_core::neat::brain::{INFER_TIME_NS, INFER_COUNT, HTTP_TIME_NS, REMOTE_INFER_NS};
use sim_core::neat::runner::{PHYS_TIME_NS, PHYS_COUNT};
use sim_core::neat::onnx_exporter::export_genome;
use std::sync::Arc;

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
    let mut sim_cfg = Config::default();
    // Toggle ONNX GPU via CLI flag
    let use_onnx_gpu = args.contains(&"--onnx-gpu".to_string());
    sim_cfg.use_onnx_gpu = use_onnx_gpu;
    println!("ONNX GPU enabled: {}", use_onnx_gpu);
    // Toggle Python service via CLI flag
    let python_url = args.windows(2)
        .find(|w| w[0] == "--python-service-url")
        .map(|w| w[1].clone());
    let use_python_service = python_url.is_some();
    sim_cfg.use_python_service = use_python_service;
    sim_cfg.python_service_url = python_url;
    println!("Python service enabled: {}", use_python_service);
    if use_python_service {
        println!("Python service URL: {}", sim_cfg.python_service_url.as_ref().unwrap());
    }

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
    // Initialize ONNX GPU session if enabled
    if sim_cfg.use_onnx_gpu {
        // Export the first genome as a prototype superset model
        let model_bytes = export_genome(&population.genomes[0]);
        let sess = sim_cfg.onnx_env.as_ref().unwrap()
            .new_session_builder().unwrap()
            .with_cuda().unwrap()
            .with_model_from_memory(&model_bytes).unwrap();
        sim_cfg.onnx_session = Some(Arc::new(sess));
    }
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
    let http_time = HTTP_TIME_NS.load(Ordering::Relaxed);
    let remote_time = REMOTE_INFER_NS.load(Ordering::Relaxed);
    println!("HTTP:      {:.2} ms total", http_time as f64 / 1e6);
    println!("Remote:    {:.2} ms total", remote_time as f64 / 1e6);
}
