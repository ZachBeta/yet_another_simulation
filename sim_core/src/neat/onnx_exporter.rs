use prost::Message;
use super::onnx_minimal::{
    DataType, ModelProto, GraphProto, NodeProto, TensorProto, ValueInfoProto,
    TensorShapeProto, TensorShapeProto_Dimension, TypeProto, TypeProto_Tensor,
    tensor_shape_proto_dimension,
};
use super::genome::Genome;

/// Convert a strictly feed-forward Genome into ONNX bytes
pub fn export_genome(genome: &Genome) -> Vec<u8> {
    // 1. Initialize ModelProto and GraphProto
    let mut model = ModelProto::default();
    model.ir_version = 7;
    let mut graph = GraphProto::default();
    graph.name = "neat_model".into();

    // 2. Define input ValueInfo
    let in_dim = genome.input_size();
    let input_info = {
        let shape = TensorShapeProto {
            dim: vec![
                TensorShapeProto_Dimension { dim: Some(tensor_shape_proto_dimension::Dim::DimParam("batch".into())) },
                TensorShapeProto_Dimension { dim: Some(tensor_shape_proto_dimension::Dim::DimValue(in_dim as i64)) },
            ],
        };
        let tensor_type = TypeProto_Tensor { elem_type: DataType::Float as i32, shape: Some(shape) };
        let r#type = Some(TypeProto { tensor_type: Some(tensor_type) });
        ValueInfoProto { name: "X".into(), r#type }
    };
    graph.input.push(input_info);

    // 3. Build feed-forward layers
    let mut prev = "X".to_string();
    for (i, layer) in genome.layers().iter().enumerate() {
        let out_dim = layer.output_size();
        let in_dim = layer.input_size();

        // Weight initializer
        let mut w = TensorProto::default();
        w.name = format!("W{}", i);
        w.data_type = DataType::Float as i32;
        w.dims = vec![out_dim as i64, in_dim as i64];
        w.raw_data = layer.weight_bytes().clone();
        graph.initializer.push(w.clone());

        // Bias initializer
        let mut b = TensorProto::default();
        b.name = format!("B{}", i);
        b.data_type = DataType::Float as i32;
        b.dims = vec![out_dim as i64];
        b.raw_data = layer.bias_bytes().clone();
        graph.initializer.push(b.clone());

        // MatMul node
        let mat_out = format!("mat{}", i);
        graph.node.push(NodeProto {
            input: vec![prev.clone(), w.name.clone()],
            output: vec![mat_out.clone()],
            op_type: "MatMul".into(),
        });

        // Add node
        let add_out = format!("pre{}", i);
        graph.node.push(NodeProto {
            input: vec![mat_out, b.name.clone()],
            output: vec![add_out.clone()],
            op_type: "Add".into(),
        });

        // Activation node
        let act_out = format!("act{}", i);
        graph.node.push(NodeProto {
            input: vec![add_out],
            output: vec![act_out.clone()],
            op_type: "Relu".into(),
        });
        prev = act_out;
    }

    // 4. Define output ValueInfo
    let out_dim = genome.output_size();
    let output_info = {
        let shape = TensorShapeProto {
            dim: vec![
                TensorShapeProto_Dimension { dim: Some(tensor_shape_proto_dimension::Dim::DimParam("batch".into())) },
                TensorShapeProto_Dimension { dim: Some(tensor_shape_proto_dimension::Dim::DimValue(out_dim as i64)) },
            ],
        };
        let tensor_type = TypeProto_Tensor { elem_type: DataType::Float as i32, shape: Some(shape) };
        let r#type = Some(TypeProto { tensor_type: Some(tensor_type) });
        ValueInfoProto { name: prev.clone(), r#type }
    };
    graph.output.push(output_info);

    // 5. Finalize and encode
    model.graph = Some(graph);
    model.encode_to_vec()
}
