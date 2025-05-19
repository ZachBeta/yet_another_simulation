use sim_core::config::Config;
use sim_core::neat::config::{EvolutionConfig, FitnessFn};
use sim_core::neat::population::Population;
use sim_core::neat::runner::{PHYS_TIME_NS, PHYS_COUNT, MATCH_TIME_NS, MATCH_COUNT, MatchStats};
use sim_core::neat::runner::run_match_record;
use sim_core::neat::runner::run_match;
use sim_core::Brain;
use sim_core::neat::brain::NeatBrain;
use std::env;
use std::fs;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use num_cpus;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::sync::atomic::Ordering;
use sim_core::neat::brain::{INFER_TIME_NS, INFER_COUNT, HTTP_TIME_NS, REMOTE_INFER_NS};
use clap::{Parser, Subcommand, Args};
use clap::ArgAction;
use sim_core::neat::genome::Genome;
use sim_core::domain::{WorldView, Vec2};
use reqwest::blocking::Client;
use serde_json::json;
use sim_core::neat::onnx_exporter::export_genome;
use serde_json;
use sim_core::ai::{NaiveAgent, NaiveBrain};
use std::collections::HashMap;
use indicatif::ParallelProgressIterator;
use std::collections::VecDeque;
use clap::ValueEnum;
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};

/// neat_train CLI with `bench`, `train`, and `tournament` subcommands
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run inference benchmarks (CPU or Python/ONNX service)
    Bench(BenchOpts),
    /// Train a NEAT population with snapshots and status
    Train(TrainOpts),
    /// Evaluate champions against a naive agent
    Tournament(TournamentOpts),
}

/// Options for the `bench` subcommand
#[derive(Args, Debug)]
struct BenchOpts {
    #[clap(long, default_value = "cpu")]
    device: String,
    #[clap(long, default_value_t = 10)]
    runs: usize,
    #[clap(long, default_value_t = num_cpus::get().saturating_sub(1))]
    workers: usize,
    #[clap(long, default_value = "http://127.0.0.1:8000")]
    python_service_url: String,
    #[clap(long, action=ArgAction::SetTrue)]
    batch: bool,
    #[clap(long, default_value_t = 1)]
    batch_size: usize,
    #[clap(long)]
    duration: Option<u64>,
    #[clap(long, action=ArgAction::SetTrue)]
    bench_verbose: bool,
    /// Export initial genome ONNX to path and exit
    #[clap(long)]
    export_model: Option<String>,
}

/// Options for the `train` subcommand
#[derive(Args, Debug)]
struct TrainOpts {
    #[clap(long, default_value = "cpu")]
    device: String,
    #[clap(long, default_value_t = num_cpus::get().saturating_sub(1))]
    workers: usize,
    /// fixed number of generations; overrides duration
    #[clap(long)]
    runs: Option<usize>,
    /// wall-clock limit in seconds
    #[clap(long)]
    duration: Option<u64>,
    /// snapshot every N generations
    #[clap(long, default_value_t = 5)]
    snapshot_interval: usize,
    /// number of generations between difficulty increases
    #[clap(long, default_value_t = 10)]
    difficulty_interval: usize,
    /// threshold of avg_naive to bump difficulty
    #[clap(long, default_value = "80.0")]
    difficulty_threshold: f32,
    #[clap(long, action=ArgAction::SetTrue)]
    verbose: bool,
    /// generations without improvement before triggering recovery
    #[clap(long, default_value_t = 20)]
    stagnation_window: usize,
    /// number of random genomes to inject when stagnated
    #[clap(long, default_value_t = 2)]
    inject_count: usize,
    /// scale factor to multiply mutation rates during recovery
    #[clap(long, default_value_t = 2.0)]
    mutation_scale: f32,
    /// Which fitness function to use
    #[clap(long, value_enum, default_value_t = FitnessFnArg::HealthPlusDamage)]
    fitness_fn: FitnessFnArg,
    /// Weight for time-to-win bonus (only for time-based fitness)
    #[clap(long, default_value_t = 0.1)]
    time_bonus_weight: f32,
    /// Weight for health in fitness
    #[clap(long, default_value_t = 1.0)]
    w_health: f32,
    /// Weight for damage in fitness
    #[clap(long, default_value_t = 1.0)]
    w_damage: f32,
    /// Weight for kills in fitness
    #[clap(long, default_value_t = 0.5)]
    w_kills: f32,
    /// Optional override for run ID
    #[clap(long)]
    run_id: Option<String>,
    /// Random seed for scenario randomization
    #[clap(long)]
    random_seed: Option<u64>,
    /// Max variation for map dimensions (±)
    #[clap(long, default_value_t = 0)]
    map_var: u32,
}

/// Options for the `tournament` subcommand
#[derive(Args, Debug)]
struct TournamentOpts {
    /// directory containing champion JSON files
    #[clap(long, default_value = "out")]
    pop_path: String,
    /// verbose per-match logs
    #[clap(long, action=ArgAction::SetTrue)]
    verbose: bool,
    /// include naive agent in tournament for Elo ranking
    #[clap(long, action=ArgAction::SetTrue)]
    include_naive: bool,
}

/// Available fitness function types
#[derive(ValueEnum, Clone, Debug)]
enum FitnessFnArg {
    HealthPlusDamage,
    HealthPlusDamageTime,
}

/// Run CPU or MPS inference bench and exit
fn bench_inference(sim_cfg: &Config, evo_cfg: &EvolutionConfig, runs: usize, batch: bool, verbose: bool) {
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
        // Parallel batched inference
        let inputs: Vec<_> = vec![input_row.clone(); runs];
        let batches: Vec<_> = inputs
            .chunks(sim_cfg.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        total_ns = batches
            .par_iter()
            .map(|batch_inputs| {
                let payload = json!({ "inputs": batch_inputs });
                let start = Instant::now();
                let _ = client.post(&format!("{}/{}", url, endpoint))
                    .json(&payload)
                    .send().unwrap()
                    .json::<serde_json::Value>().unwrap();
                start.elapsed().as_nanos()
            })
            .sum();
    } else {
        for _ in 0..runs {
            let start = Instant::now();
            let _ = genome.feed_forward(&input_row);
            total_ns += start.elapsed().as_nanos();
        }
    }
    let avg_ms = total_ns as f64 / runs as f64 / 1e6;
    if verbose {
        // timestamp
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        println!("[{}] Device={} runs={} avg_infer_ms={:.3}",
                 now,
                 if sim_cfg.use_python_service { "mps" } else { "cpu" },
                 runs, avg_ms);
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Bench(opts) => run_bench(&opts),
        Command::Train(opts) => run_train(&opts),
        Command::Tournament(opts) => run_tournament(&opts),
    }
}

/// Run the inference benchmark
fn run_bench(opts: &BenchOpts) {
    // reuse existing bench_inference logic with opts
    let mut sim_cfg = Config::default();
    sim_cfg.use_python_service = opts.device == "mps";
    sim_cfg.python_service_url = if opts.device == "mps" { Some(opts.python_service_url.clone()) } else { None };
    sim_cfg.batch_size = opts.batch_size;
    let mut evo_cfg = EvolutionConfig::default();
    ThreadPoolBuilder::new().num_threads(opts.workers).build_global().unwrap();
    if let Some(path) = &opts.export_model {
        let mut genome = Genome::new();
        genome.initialize(&sim_cfg, &evo_cfg);
        let bytes = export_genome(&genome);
        fs::write(path, bytes).expect("Failed to write ONNX model");
        println!("Exported ONNX model to {}", path);
        return;
    }
    // If duration mode, run until wall-clock >= duration
    if let Some(secs) = opts.duration {
        let dur = Duration::from_secs(secs);
        let start = Instant::now();
        (0..opts.workers)
            .into_par_iter()
            .for_each(|_| {
                while start.elapsed() < dur {
                    bench_inference(&sim_cfg, &evo_cfg, sim_cfg.batch_size, opts.batch, opts.bench_verbose);
                }
            });
    } else {
        // runs-based mode
        let base = opts.runs / opts.workers;
        let rem = opts.runs % opts.workers;
        (0..opts.workers)
            .into_par_iter()
            .enumerate()
            .for_each(|(i, _)| {
                let runs_i = base + if i < rem { 1 } else { 0 };
                bench_inference(&sim_cfg, &evo_cfg, runs_i, opts.batch, opts.bench_verbose);
            });
    }
}

/// Run the NEAT training loop with snapshots and status logs
fn run_train(opts: &TrainOpts) {
    ThreadPoolBuilder::new().num_threads(opts.workers).build_global().unwrap();
    fs::create_dir_all("out").unwrap();
    // generate run-specific ID and create output directory
    let ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let fn_name = opts.fitness_fn.to_possible_value().unwrap().get_name().to_string();
    let id = opts.run_id.clone().unwrap_or_else(|| format!(
        "{}-fn-{}-h{:.1}-d{:.1}-k{:.1}",
        ts, fn_name, opts.w_health, opts.w_damage, opts.w_kills
    ));
    let out_dir = format!("out/{}", id);
    fs::create_dir_all(&out_dir).unwrap();
    let mut sim_cfg = Config::default();
    sim_cfg.use_python_service = false;
    sim_cfg.python_service_url = None;
    let mut evo_cfg = EvolutionConfig::default();
    evo_cfg.pop_size = 10;
    evo_cfg.tournament_k = 2;
    evo_cfg.max_ticks = 200;
    evo_cfg.num_teams = 2;
    evo_cfg.team_size = 1;
    // upper bound on generations (usize::MAX if unlimited)
    let max_gens = opts.runs.unwrap_or(usize::MAX);
    let mut population = Population::new(&evo_cfg);
    let start = Instant::now();
    let mut gen = 0;
    // Track base sensor range for difficulty adjustments
    let base_scan_max_dist = sim_cfg.scan_max_dist;
    // keep original mutation rates for auto-recovery
    let orig_node_rate = evo_cfg.mutation_add_node_rate;
    let orig_conn_rate = evo_cfg.mutation_add_conn_rate;
    let mut recovery_active = false;
    let mut best_history: VecDeque<f32> = VecDeque::new();
    // RNG for scenario randomization
    let mut rng = match opts.random_seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    let base_map_w = evo_cfg.map_width;
    let base_map_h = evo_cfg.map_height;
    // run until generation or time limit
    while gen < max_gens && (opts.duration.map_or(true, |s| start.elapsed() < Duration::from_secs(s))) {
        // scenario randomization per generation
        if opts.map_var > 0 {
            let delta_w = rng.gen_range(-(opts.map_var as i32)..=(opts.map_var as i32));
            let delta_h = rng.gen_range(-(opts.map_var as i32)..=(opts.map_var as i32));
            evo_cfg.map_width = (base_map_w as i32 + delta_w).max(1) as u32;
            evo_cfg.map_height = (base_map_h as i32 + delta_h).max(1) as u32;
            println!("[{:.2}s] randomized map size → {}x{}", start.elapsed().as_secs_f32(), evo_cfg.map_width, evo_cfg.map_height);
        }
        // reset instrumentation counters
        PHYS_TIME_NS.store(0, Ordering::Relaxed);
        PHYS_COUNT.store(0, Ordering::Relaxed);
        MATCH_TIME_NS.store(0, Ordering::Relaxed);
        MATCH_COUNT.store(0, Ordering::Relaxed);
        INFER_TIME_NS.store(0, Ordering::Relaxed);
        INFER_COUNT.store(0, Ordering::Relaxed);
        HTTP_TIME_NS.store(0, Ordering::Relaxed);
        REMOTE_INFER_NS.store(0, Ordering::Relaxed);
        // Timestamped generation header
        println!("[{:.2}s] --- Generation {} ---", start.elapsed().as_secs_f32(), gen);
        let eval_start = Instant::now();
        // Evaluate and log stats
        population.evaluate(&sim_cfg, &evo_cfg);
        let eval_dur = eval_start.elapsed();
        println!(" Evaluation took: {:?}", eval_dur);
        // performance instrumentation
        let phys_ns = PHYS_TIME_NS.load(Ordering::Relaxed);
        let phys_ct = PHYS_COUNT.load(Ordering::Relaxed);
        let match_ns = MATCH_TIME_NS.load(Ordering::Relaxed);
        let match_ct = MATCH_COUNT.load(Ordering::Relaxed);
        let infer_ns = INFER_TIME_NS.load(Ordering::Relaxed);
        let infer_ct = INFER_COUNT.load(Ordering::Relaxed);
        let http_ns = HTTP_TIME_NS.load(Ordering::Relaxed);
        let remote_ns = REMOTE_INFER_NS.load(Ordering::Relaxed);
        println!(
            "  Perf: sim avg = {:.2}µs/tick ({} ticks); match avg = {:.2}ms/match ({} matches)",
            phys_ns as f64 / phys_ct as f64 / 1e3, phys_ct,
            match_ns as f64 / match_ct as f64 / 1e6, match_ct,
        );
        println!(
            "        infer avg = {:.2}µs ({}); http total = {:.2}ms; remote infer = {:.2}ms",
            infer_ns as f64 / infer_ct as f64 / 1e3, infer_ct,
            http_ns as f64 / 1e6, remote_ns as f64 / 1e6,
        );
        let fitnesses: Vec<f32> = population.genomes.iter().map(|g| g.fitness).collect();
        let best = *fitnesses.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;
        let naive_vals: Vec<f32> = population.genomes.iter().map(|g| g.fitness_naive).collect();
        let best_naive = *naive_vals.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg_naive = naive_vals.iter().sum::<f32>() / naive_vals.len() as f32;
        println!(
            "Gen {}: best = {:.2}, avg = {:.2}, naive_best = {:.2}, avg_naive = {:.2}",
            gen, best, avg, best_naive, avg_naive
        );
        println!("=== Profiling Summary ===");
        println!(
            "Inference: {:.2} ms total over {} calls", infer_ns as f64 / 1e6, infer_ct
        );
        println!(
            "Physics:   {:.2} ms total over {} steps", phys_ns as f64 / 1e6, phys_ct
        );
        println!(
            "HTTP:      {:.2} ms total", http_ns as f64 / 1e6
        );
        println!(
            "Remote:    {:.2} ms total", remote_ns as f64 / 1e6
        );
        // Bump difficulty if threshold & interval reached
        if gen > 0
           && gen % opts.difficulty_interval == 0
           && avg_naive >= opts.difficulty_threshold
           && sim_cfg.difficulty_level < sim_cfg.max_difficulty
        {
            sim_cfg.difficulty_level += 1;
            // Shrink sensor range by 10% per level
            sim_cfg.scan_max_dist = base_scan_max_dist * (1.0 - sim_cfg.difficulty_level as f32 * 0.1);
            println!("[{:.2}s] ↑ Difficulty → level {}, scan_max_dist={:.2}",
                     start.elapsed().as_secs_f32(), sim_cfg.difficulty_level, sim_cfg.scan_max_dist);
        }
        // Hall of Fame
        println!("Hall of Fame (top {}):", evo_cfg.hof_size);
        for (i, g) in population.hof.iter().enumerate() {
            println!("  HoF {}: {:.2}", i, g.fitness);
        }
        // Replay champion vs second-best
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
            let path = format!("{}/champ_replay.jsonl", out_dir);
            let stats = run_match_record(&path, &sim_cfg, &evo_cfg, agents);
            println!("  Replay: ticks = {}, health = {:.2}", stats.ticks, stats.subject_team_health);
        }
        // Snapshot champion weights for continued use
        {
            let champ = population.hof[0].clone();
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
            let total_duration = start.elapsed().as_secs_f32();
            let phys_ns = PHYS_TIME_NS.load(Ordering::Relaxed);
            let phys_ct = PHYS_COUNT.load(Ordering::Relaxed);
            let match_ns = MATCH_TIME_NS.load(Ordering::Relaxed);
            let match_ct = MATCH_COUNT.load(Ordering::Relaxed);
            let infer_ns = INFER_TIME_NS.load(Ordering::Relaxed);
            let infer_ct = INFER_COUNT.load(Ordering::Relaxed);
            let http_ns = HTTP_TIME_NS.load(Ordering::Relaxed);
            let remote_ns = REMOTE_INFER_NS.load(Ordering::Relaxed);
            let metadata = json!({
                "timestamp": timestamp,
                "duration_s": total_duration,
                "generation": gen,
                "config": {
                    "device": opts.device,
                    "workers": opts.workers,
                    "runs": opts.runs,
                    "duration_limit_s": opts.duration,
                    "snapshot_interval": opts.snapshot_interval,
                    "fitness_fn": fn_name,
                    "time_bonus_weight": opts.time_bonus_weight,
                    "w_health": opts.w_health,
                    "w_damage": opts.w_damage,
                    "w_kills": opts.w_kills,
                    "random_seed": opts.random_seed,
                    "map_var": opts.map_var,
                    "run_id": opts.run_id
                },
                "simulation_config": {
                    "nearest_k_enemies": sim_cfg.nearest_k_enemies,
                    "nearest_k_allies": sim_cfg.nearest_k_allies,
                    "nearest_k_wrecks": sim_cfg.nearest_k_wrecks,
                    "batch_size": sim_cfg.batch_size,
                    "use_python_service": sim_cfg.use_python_service,
                    "python_service_url": sim_cfg.python_service_url,
                    "map_width": evo_cfg.map_width,
                    "map_height": evo_cfg.map_height,
                    "difficulty_level": sim_cfg.difficulty_level,
                    "max_difficulty": sim_cfg.max_difficulty
                },
                "evolution_config": {
                    "pop_size": evo_cfg.pop_size,
                    "tournament_k": evo_cfg.tournament_k,
                    "max_ticks": evo_cfg.max_ticks,
                    "num_teams": evo_cfg.num_teams,
                    "team_size": evo_cfg.team_size,
                    "hof_size": evo_cfg.hof_size
                },
                "fitness_weights": {
                    "health": evo_cfg.w_health,
                    "damage": evo_cfg.w_damage,
                    "kills": evo_cfg.w_kills,
                    "time_bonus": evo_cfg.time_bonus_weight
                },
                "instrumentation": {
                    "sim_avg_us": phys_ns as f64 / phys_ct as f64 / 1e3,
                    "match_avg_ms": match_ns as f64 / match_ct as f64 / 1e6,
                    "infer_avg_us": infer_ns as f64 / infer_ct as f64 / 1e3,
                    "http_total_ms": http_ns as f64 / 1e6,
                    "remote_infer_ms": remote_ns as f64 / 1e6
                },
                // champion's baseline performance against NaiveAgent
                "champion_fitness_naive": champ.fitness_naive
            });
            let output = json!({
                "metadata": metadata,
                "genome": champ
            });
            let json_str = serde_json::to_string_pretty(&output).unwrap();
            fs::write(format!("{}/champion_latest.json", out_dir), &json_str).expect("Failed to write champion_latest");
            fs::write(format!("{}/champion_gen_{:03}.json", out_dir, gen), &json_str)
                .expect("Failed to write champion_gen file");
        }
        if gen % opts.snapshot_interval == 0 || gen + 1 == max_gens {
            let champ = &population.hof[0];
            let json = serde_json::to_string(champ).unwrap();
            fs::write(format!("{}/champion_gen_{:03}.json", out_dir, gen), &json).unwrap();
            fs::write(format!("{}/champion_latest.json", out_dir), &json).unwrap();
            if opts.verbose {
                eprintln!("[{:.1}s] ▶ snapshot champion → {}/champion_gen_{:03}.json", start.elapsed().as_secs_f32(), out_dir, gen);
            }
        }
        // detect stagnation over sliding window
        best_history.push_back(best);
        if best_history.len() > opts.stagnation_window {
            best_history.pop_front();
        }
        if best_history.len() == opts.stagnation_window
            && best_history.iter().all(|&v| (v - best_history[0]).abs() < f32::EPSILON)
        {
            println!("No improvement in {} gens; injecting {} random genomes and scaling mutation x{:.2}",
                     opts.stagnation_window, opts.inject_count, opts.mutation_scale);
            evo_cfg.mutation_add_node_rate = orig_node_rate * opts.mutation_scale;
            evo_cfg.mutation_add_conn_rate = orig_conn_rate * opts.mutation_scale;
            recovery_active = true;
        }
        if gen + 1 < max_gens {
            population.reproduce(&evo_cfg);
            // apply auto-recovery: inject random genomes and revert rates
            if recovery_active {
                population.genomes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
                for _ in 0..opts.inject_count {
                    population.genomes.pop();
                    population.genomes.push(Genome::new());
                }
                evo_cfg.mutation_add_node_rate = orig_node_rate;
                evo_cfg.mutation_add_conn_rate = orig_conn_rate;
                recovery_active = false;
            }
        }
        // apply selected fitness function and weight
        evo_cfg.fitness_fn = match opts.fitness_fn {
            FitnessFnArg::HealthPlusDamage => FitnessFn::HealthPlusDamage,
            FitnessFnArg::HealthPlusDamageTime => FitnessFn::HealthPlusDamageTime,
        };
        evo_cfg.time_bonus_weight = opts.time_bonus_weight;
        evo_cfg.w_health = opts.w_health;
        evo_cfg.w_damage = opts.w_damage;
        evo_cfg.w_kills = opts.w_kills;
        gen += 1;
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
    println!("Trained {} gens in {:.1}s → {:.2} gens/sec", gen, start.elapsed().as_secs_f32(), gen as f32 / start.elapsed().as_secs_f32());
}

/// Run a round-robin tournament among all champions, compute and dump Elo ratings
fn run_tournament(opts: &TournamentOpts) {
    // reset profiling counters
    PHYS_TIME_NS.store(0, Ordering::Relaxed);
    PHYS_COUNT.store(0, Ordering::Relaxed);
    INFER_TIME_NS.store(0, Ordering::Relaxed);
    INFER_COUNT.store(0, Ordering::Relaxed);
    HTTP_TIME_NS.store(0, Ordering::Relaxed);
    REMOTE_INFER_NS.store(0, Ordering::Relaxed);
    // configure Rayon thread pool to cpu_count-1 threads
    let threads = num_cpus::get().saturating_sub(1).max(1);
    ThreadPoolBuilder::new().num_threads(threads).build_global().expect("Failed to build global thread pool");
    // Ensure output dir exists
    fs::create_dir_all(&opts.pop_path).unwrap();
    // Simulation configs
    let mut sim_cfg = Config::default();
    sim_cfg.use_python_service = false;
    sim_cfg.batch_size = 1;
    sim_cfg.python_service_url = None;
    let mut evo_cfg = EvolutionConfig::default();
    evo_cfg.num_teams = 2;
    evo_cfg.team_size = 1;
    evo_cfg.max_ticks = 200;
    // Load champion genomes from JSON files
    let champions: Vec<(String, Genome)> = fs::read_dir(&opts.pop_path).unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                return None;
            }
            let fname = path.file_name()?.to_string_lossy().to_string();
            let data = fs::read_to_string(&path).ok()?;
            let g: Genome = serde_json::from_str(&data).ok()?;
            Some((fname, g))
        })
        .collect();
    if champions.is_empty() {
        println!("Need at least one champion in {}", opts.pop_path);
        return;
    }
    // Build participants list (champions and optional naive)
    let mut participants: Vec<(String, Option<Genome>)> =
        champions.into_iter().map(|(fname, g)| (fname, Some(g))).collect();
    if opts.include_naive {
        participants.push(("Naive".to_string(), None));
    }
    let total = participants.len();
    // Initialize Elo ratings at 1200
    let mut ratings: HashMap<String, f32> = participants.iter()
        .map(|(name, _)| (format!("{}/{}", opts.pop_path, name), 1200.0))
        .collect();
    let k_factor = 32.0;
    // Generate all unique pairs (i < j)
    let pairs: Vec<(usize, usize)> = (0..total)
        .flat_map(|i| ((i+1)..total).map(move |j| (i, j)))
        .collect();
    // Run matches in parallel and collect outcomes
    let total_pairs = pairs.len() as u64;
    println!("Running {} matchups…", total_pairs);
    let outcomes = pairs.into_par_iter()
        .progress_count(total_pairs)
        .map(|(i, j)| {
            // instantiate competitor brains
            let brain_i: Box<dyn Brain> = if let Some(ref gi) = participants[i].1 {
                Box::new(NeatBrain::new(gi.clone(), sim_cfg.batch_size, String::new()))
            } else {
                Box::new(NaiveBrain(NaiveAgent::new(sim_cfg.max_speed, 10.0)))
            };
            let brain_j: Box<dyn Brain> = if let Some(ref gj) = participants[j].1 {
                Box::new(NeatBrain::new(gj.clone(), sim_cfg.batch_size, String::new()))
            } else {
                Box::new(NaiveBrain(NaiveAgent::new(sim_cfg.max_speed, 10.0)))
            };
            let stats = run_match(&sim_cfg, &evo_cfg, vec![(brain_i, 0), (brain_j, 1)]);
            let win_i = stats.subject_team_health > 0.0;
            (i, j, win_i)
        }).collect::<Vec<_>>();
    println!(); // newline after progress bar
    // Sequentially update Elo ratings
    for (i, j, win_i) in outcomes {
        let pi = format!("{}/{}", opts.pop_path, participants[i].0);
        let pj = format!("{}/{}", opts.pop_path, participants[j].0);
        let ri = *ratings.get(&pi).unwrap();
        let rj = *ratings.get(&pj).unwrap();
        let expected_i = 1.0 / (1.0 + 10f32.powf((rj - ri) / 400.0));
        let expected_j = 1.0 / (1.0 + 10f32.powf((ri - rj) / 400.0));
        let score_i = if win_i { 1.0 } else { 0.0 };
        let score_j = 1.0 - score_i;
        *ratings.get_mut(&pi).unwrap() += k_factor * (score_i - expected_i);
        *ratings.get_mut(&pj).unwrap() += k_factor * (score_j - expected_j);
    }
    // Write Elo ratings to JSON
    let elo_path = format!("{}/elo_ratings.json", opts.pop_path);
    let out_list: Vec<_> = ratings.iter()
        .map(|(path, &elo)| json!({ "path": path, "elo": elo }))
        .collect();
    fs::write(&elo_path, serde_json::to_string_pretty(&out_list).unwrap())
        .expect("Failed to write elo_ratings.json");
    println!("Wrote Elo ratings to {}", elo_path);
    // Profiling summary
    let phys_ns = PHYS_TIME_NS.load(Ordering::Relaxed);
    let phys_count = PHYS_COUNT.load(Ordering::Relaxed);
    if phys_count > 0 {
        println!("Physics steps: {}, avg step time: {:.3} ms", phys_count, phys_ns as f64 / phys_count as f64 / 1e6);
    }
    let infer_ns = INFER_TIME_NS.load(Ordering::Relaxed);
    let infer_count = INFER_COUNT.load(Ordering::Relaxed);
    if infer_count > 0 {
        println!("Inference calls: {}, avg inference time: {:.3} ms", infer_count, infer_ns as f64 / infer_count as f64 / 1e6);
    }
    let http_ns = HTTP_TIME_NS.load(Ordering::Relaxed);
    if http_ns > 0 {
        println!("HTTP overhead total: {:.3} ms", http_ns as f64 / 1e6);
    }
    let remote_ns = REMOTE_INFER_NS.load(Ordering::Relaxed);
    if remote_ns > 0 {
        println!("Remote inference total: {:.3} ms", remote_ns as f64 / 1e6);
    }
}
