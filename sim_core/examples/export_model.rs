use std::fs;
use sim_core::neat::population::Population;
use sim_core::neat::onnx_exporter::export_genome;

fn main() {
    // Initialize a default population and export its first genome to ONNX bytes
    let pop = Population::new(&Default::default());
    let bytes = export_genome(&pop.genomes[0]);
    // Write the ONNX model next to the Python service
    fs::write("../python_onnx_service/model.onnx", bytes)
        .expect("Failed to write ONNX model to python_onnx_service/model.onnx");
    println!("Wrote python_onnx_service/model.onnx");
}
