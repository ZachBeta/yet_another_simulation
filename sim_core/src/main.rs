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
use clap::Parser;
use clap::ArgAction;
use sim_core::neat::genome::Genome;
use sim_core::domain::{WorldView, Vec2};
use reqwest::blocking::Client;
use serde_json::json;
use sim_core::neat::onnx_exporter::export_genome;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Opts {
    /// device: cpu or mps
    #[clap(long, default_value = "cpu")]
    device: String,
    /// number of runs
    #[clap(long, default_value_t = 10)]
    runs: usize,
    /// number of worker threads
    #[clap(long, default_value_t = num_cpus::get() / 2)]
    workers: usize,
    /// URL for Python service
    #[clap(long, default_value = "http://127.0.0.1:8000")]
    python_service_url: String,
    /// Export initial genome ONNX to path and exit
    #[clap(long)]
    export_model: Option<String>,
    /// toggle batch endpoint vs raw
    #[clap(long, action=ArgAction::Set, default_value_t = false)]
    batch: bool,
    /// batch size for Python service
    #[clap(long, default_value_t = 1)]
    batch_size: usize,
}

/// Run CPU or MPS inference bench and exit
fn bench_inference(sim_cfg: &Config, evo_cfg: &EvolutionConfig, runs: usize, batch: bool) {
    let mut genome = Genome::new();
    genome.initialize(sim_cfg, evo_cfg);
    let input_len = 2 + sim_cfg.nearest_k_enemies * 4
                  + sim_cfg.nearest_k_allies * 4
                  + sim_cfg.nearest_k_wrecks * 3;
    let input_row = vec![0.0f32; input_len];
    let mut total_ns: u128 = 0;
    if sim_cfg.use_python_service {
        // Test Python or batched service connectivity
        let url = sim_cfg.python_service_url.as_ref().unwrap();
        let client = Client::new();
        let endpoint = if batch { "infer_batch" } else { "infer" };
        let test_payload = json!({ "inputs": [input_row.clone()] });
        client.post(&format!("{}/{}", url, endpoint))
            .json(&test_payload)
            .send()
            .unwrap_or_else(|e| { eprintln!("Failed to connect to Python service at {}: {}", url, e); std::process::exit(1) });
        eprintln!("[bench_inference] Connected to Python service at {}", url);
        let mut runs_left = runs;
        while runs_left > 0 {
            let batch_sz = runs_left.min(sim_cfg.batch_size);
            let payload = json!({ "inputs": vec![input_row.clone(); batch_sz] });
            let start = Instant::now();
            let _ = client.post(&format!("{}/{}", url, endpoint))
                .json(&payload)
                .send().unwrap()
                .json::<serde_json::Value>().unwrap();
            total_ns += start.elapsed().as_nanos();
            runs_left -= batch_sz;
        }
    } else {
        for _ in 0..runs {
            let start = Instant::now();
            let _ = genome.feed_forward(&input_row);
            total_ns += start.elapsed().as_nanos();
        }
    }
    let avg_ms = total_ns as f64 / runs as f64 / 1e6;
    println!("Device={} runs={} avg_infer_ms={:.3}",
             if sim_cfg.use_python_service { "mps" } else { "cpu" },
             runs, avg_ms);
}

fn main() {
    let opts = Opts::parse();
    ThreadPoolBuilder::new()
        .num_threads(opts.workers)
        .build_global()
        .unwrap();
    println!("Using {} worker threads", opts.workers);

    // Simulation config
    let mut sim_cfg = Config::default();
    sim_cfg.batch_size = opts.batch_size;
    // Device selection: cpu or mps
    sim_cfg.use_python_service = false;
    sim_cfg.python_service_url = None;
    match opts.device.as_str() {
        "cpu" => {},
        "mps" => {
            sim_cfg.use_python_service = true;
            sim_cfg.python_service_url = Some(opts.python_service_url.clone());
        },
        d => panic!("Unknown device: {}", d),
    }
    println!("Device: {}", opts.device);
    if let Some(url) = &sim_cfg.python_service_url {
        println!("Python service URL: {}", url);
    }

    // NEAT evolution config (override for quick test)
    let mut evo_cfg = EvolutionConfig::default();
    evo_cfg.pop_size = 10;
    evo_cfg.tournament_k = 2;
    evo_cfg.max_ticks = 200;
    evo_cfg.num_teams = 2;
    evo_cfg.team_size = 1;

    // Export ONNX model if requested
    if let Some(path) = &opts.export_model {
        let mut genome = Genome::new();
        genome.initialize(&sim_cfg, &evo_cfg);
        let bytes = export_genome(&genome);
        fs::write(path, bytes).expect("Failed to write ONNX model");
        println!("Exported ONNX model to {}", path);
        return;
    }

    // If benchmarking CPU/MPS, run bench and exit
    if opts.device == "cpu" || opts.device == "mps" {
        bench_inference(&sim_cfg, &evo_cfg, opts.runs, opts.batch);
        return;
    }

    // Number of generations to run (mapped from --runs)
    let max_gens = opts.runs;
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
                (Box::new(NeatBrain::new(
                    champ.clone(),
                    sim_cfg.batch_size,
                    sim_cfg.python_service_url.clone().unwrap_or_default(),
                )) as Box<dyn Brain>, 0),
                (Box::new(NeatBrain::new(
                    opp.clone(),
                    sim_cfg.batch_size,
                    sim_cfg.python_service_url.clone().unwrap_or_default(),
                )) as Box<dyn Brain>, 1),
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
