// Example to verify CPU-only inference using Genome::feed_forward
use sim_core::config::Config;
use sim_core::neat::config::EvolutionConfig;
use sim_core::neat::population::Population;

fn main() {
    // Initialize simulation and evolution configs
    let sim_cfg = Config::default();
    let evo_cfg = EvolutionConfig::default();
    // Create a population and initialize its first genome
    let mut pop = Population::new(&evo_cfg);
    let mut genome = pop.genomes[0].clone();
    genome.initialize(&sim_cfg, &evo_cfg);
    // Prepare a dummy input of appropriate size
    let input_size = genome.input_size();
    let dummy_input = vec![0.5_f32; input_size];
    // Run CPU-only feed-forward inference
    let outputs = genome.feed_forward(&dummy_input);
    println!("feed_forward outputs: {:?}", outputs);
}
