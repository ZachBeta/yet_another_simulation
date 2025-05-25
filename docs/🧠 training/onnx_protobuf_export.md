# Exporting NEAT Genomes to ONNX using Protobuf (Pure Rust)

A step-by-step guide for a mid-level SWE to build a Rust-based exporter from your NEAT `Genome` to ONNX `ModelProto` using `onnx-proto` and `protobuf` crates.

## 1. Add Dependencies

In `Cargo.toml`:
```toml
[dependencies]
onnx-proto = "0.14"
protobuf   = "2.28"
```

## 2. Scaffold `onnx_exporter` Module

Create `src/neat/onnx_exporter.rs`:
```rust
use onnx_proto::{ModelProto, GraphProto, NodeProto, TensorProto, ValueInfoProto};
use onnx_proto::tensor_proto::DataType;
use onnx_proto::TensorShapeProto_Dimension;
use protobuf::{Message, RepeatedField};
use crate::neat::genome::Genome;

/// Convert a strictly feed-forward Genome into ONNX bytes
pub fn export_genome(genome: &Genome) -> Vec<u8> {
    // 1. Initialize ModelProto and GraphProto
    let mut model = ModelProto::default();
    model.set_ir_version(7);
    let mut graph = GraphProto::default();
    graph.set_name("neat_model");

    // 2. Define input ValueInfo (name "X", shape [batch, in_dim])
    let in_dim = genome.input_count();
    let mut input_info = ValueInfoProto::default();
    input_info.set_name("X".into());
    let mut ttype = input_info.mut_type().mut_tensor_type();
    ttype.set_elem_type(DataType::FLOAT.into());
    let mut shape = ttype.mut_shape();
    shape.mut_dim().push({ let mut d = TensorShapeProto_Dimension::default(); d.set_dim_param("batch".into()); d });
    shape.mut_dim().push({ let mut d = TensorShapeProto_Dimension::default(); d.set_dim_value(in_dim as i64); d });
    graph.input.push(input_info);

    // 3. Build each feed-forward layer
    let mut prev = "X".to_string();
    for (i, layer) in genome.layers().iter().enumerate() {
        let out_dim = layer.output_size();
        let in_dim = layer.input_size();
        // a) Weights initializer W_i
        let mut w = TensorProto::default();
        w.set_name(format!("W{}", i));
        w.set_data_type(DataType::FLOAT.into());
        w.set_dims(vec![out_dim as i64, in_dim as i64].into());
        w.set_raw_data(layer.weight_bytes().clone());
        graph.initializer.push(w.clone());
        // b) Bias initializer B_i
        let mut b = TensorProto::default();
        b.set_name(format!("B{}", i));
        b.set_data_type(DataType::FLOAT.into());
        b.set_dims(vec![out_dim as i64].into());
        b.set_raw_data(layer.bias_bytes().clone());
        graph.initializer.push(b.clone());
        // c) MatMul node
        let mut mm = NodeProto::default();
        mm.set_op_type("MatMul".into());
        mm.set_input(RepeatedField::from_vec(vec![prev.clone(), w.get_name().to_string()]));
        mm.set_output(RepeatedField::from_vec(vec![format!("mat{}", i)]));
        graph.node.push(mm);
        // d) Add node
        let mut add = NodeProto::default();
        add.set_op_type("Add".into());
        add.set_input(RepeatedField::from_vec(vec![format!("mat{}", i), b.get_name().to_string()]));
        add.set_output(RepeatedField::from_vec(vec![format!("pre{}", i)]));
        graph.node.push(add);
        // e) Activation node
        let mut act = NodeProto::default();
        act.set_op_type("Relu".into());
        act.set_input(RepeatedField::from_vec(vec![format!("pre{}", i)]));
        act.set_output(RepeatedField::from_vec(vec![format!("act{}", i)]));
        graph.node.push(act);
        prev = format!("act{}", i);
    }

    // 4. Define output ValueInfo (shape [batch, out_dim])
    let out_dim = genome.output_count();
    let mut output_info = ValueInfoProto::default();
    output_info.set_name(prev.clone());
    let mut otype = output_info.mut_type().mut_tensor_type();
    otype.set_elem_type(DataType::FLOAT.into());
    let mut oshape = otype.mut_shape();
    oshape.mut_dim().push({ let mut d = TensorShapeProto_Dimension::default(); d.set_dim_param("batch".into()); d });
    oshape.mut_dim().push({ let mut d = TensorShapeProto_Dimension::default(); d.set_dim_value(out_dim as i64); d });
    graph.output.push(output_info);

    // 5. Attach graph and serialize
    model.set_graph(graph);
    model.write_to_bytes().unwrap()
}

## 3. Hook into `Genome::to_onnx`

In `src/neat/genome.rs`:
```rust
impl Genome {
    pub fn to_onnx(&self) -> Vec<u8> {
        onnx_exporter::export_genome(self)
    }
}
```

## 4. Use in Evaluation Loop

When GPU-infer feature is enabled, batch-export and load ONNX models:
```rust
#[cfg(feature = "gpu-infer")]
let models: Vec<Vec<u8>> = self.genomes.iter().map(|g| g.to_onnx()).collect();
#[cfg(feature = "gpu-infer")]
let sessions: Vec<_> = models.iter().map(|m| InferenceSession::new(m)).collect();
```

## 5. Test and Iterate

1. Ensure a simple 2-layer Genome exports without error.  
2. Load the ONNX bytes in Python (`onnx.load_from_string`) or in Rust with `onnxruntime`.  
3. Compare CPU vs ONNX outputs for a test input tensor.  

## Next Steps

- Extend to support skip connections (multiple `MatMul`/`Concat` ops).  
- Cache initializers per generation to avoid reallocating.  
- Optimize serialization performance if large genome counts.

*Happy exporting!*
