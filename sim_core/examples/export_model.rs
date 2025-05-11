use std::fs;
use sim_core::neat::population::Population;
use sim_core::neat::onnx_exporter::export_genome;
use sim_core::config::Config;
use sim_core::neat::config::EvolutionConfig;

fn main() {
    // Default config for simulation and evolution
    let sim_cfg = Config::default();
    let evo_cfg = EvolutionConfig::default();
    // Initialize a population and ensure its first genome is built
    let pop = Population::new(&evo_cfg);
    let mut genome = pop.genomes[0].clone();
    genome.initialize(&sim_cfg, &evo_cfg);
    // Export to ONNX bytes
    let bytes = export_genome(&genome);
    // Write the ONNX model next to the Python service
    fs::write("../python_onnx_service/model.onnx", bytes)
        .expect("Failed to write ONNX model to python_onnx_service/model.onnx");
    println!("Wrote python_onnx_service/model.onnx");
}
