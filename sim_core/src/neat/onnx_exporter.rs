use prost::Message;
use crate::onnx_generated::onnx::{
    ModelProto, GraphProto, NodeProto, TensorProto, ValueInfoProto, TensorShapeProto,
    TypeProto, OperatorSetIdProto,
};
use crate::onnx_generated::onnx::tensor_proto::DataType;
use crate::onnx_generated::onnx::tensor_shape_proto::Dimension;
use crate::onnx_generated::onnx::tensor_shape_proto::dimension::Value as DimValue;
use crate::onnx_generated::onnx::type_proto::Tensor as TypeTensor;
use crate::onnx_generated::onnx::type_proto::Value as TypeValue;
use super::genome::Genome;

/// Convert a strictly feed-forward Genome into ONNX bytes
pub fn export_genome(genome: &Genome) -> Vec<u8> {
    // Debug: report uninitialized genome layers
    println!("export_genome: genome.layers() = {}", genome.layers().len());

    // 1) Model header
    let mut model = ModelProto::default();
    model.ir_version = Some(7);

    // 2) Graph container
    let mut graph = GraphProto::default();
    graph.name = Some("neat_model".to_string());

    // 3) Define input ValueInfo
    let in_dim = genome.input_size();
    let shape = TensorShapeProto { dim: vec![
        Dimension { denotation: None, value: Some(DimValue::DimParam("batch".to_string())) },
        Dimension { denotation: None, value: Some(DimValue::DimValue(in_dim as i64)) },
    ]};
    let tensor_type = TypeTensor { elem_type: Some(DataType::Float as i32), shape: Some(shape.clone()) };
    let mut ty = TypeProto::default();
    ty.value = Some(TypeValue::TensorType(tensor_type));
    let mut input_info = ValueInfoProto::default();
    input_info.name = Some("X".to_string());
    input_info.r#type = Some(ty);
    graph.input.push(input_info);

    // 4) Build feed-forward layers
    let mut prev = "X".to_string();
    for (i, layer) in genome.layers().iter().enumerate() {
        let out_dim = layer.output_size();
        let in_dim = layer.input_size();

        // Weight initializer
        let mut w = TensorProto::default();
        w.name = Some(format!("W{}", i));
        w.data_type = Some(DataType::Float as i32);
        w.dims = vec![in_dim as i64, out_dim as i64];
        w.raw_data = Some(layer.weight_bytes().clone());
        graph.initializer.push(w.clone());

        // Bias initializer
        let mut b = TensorProto::default();
        b.name = Some(format!("B{}", i));
        b.data_type = Some(DataType::Float as i32);
        b.dims = vec![out_dim as i64];
        b.raw_data = Some(layer.bias_bytes().clone());
        graph.initializer.push(b.clone());

        // MatMul node
        let mut mat = NodeProto::default();
        mat.input = vec![prev.clone(), format!("W{}", i)];
        mat.output = vec![format!("mat{}", i)];
        mat.op_type = Some("MatMul".to_string());
        graph.node.push(mat);

        // Add node
        let mut add = NodeProto::default();
        add.input = vec![format!("mat{}", i), format!("B{}", i)];
        add.output = vec![format!("pre{}", i)];
        add.op_type = Some("Add".to_string());
        graph.node.push(add);

        // Relu node
        let mut relu = NodeProto::default();
        relu.input = vec![format!("pre{}", i)];
        relu.output = vec![format!("act{}", i)];
        relu.op_type = Some("Relu".to_string());
        graph.node.push(relu);

        prev = format!("act{}", i);
    }

    // 5) Define output ValueInfo
    let out_dim = genome.output_size();
    let shape = TensorShapeProto { dim: vec![
        Dimension { denotation: None, value: Some(DimValue::DimParam("batch".to_string())) },
        Dimension { denotation: None, value: Some(DimValue::DimValue(out_dim as i64)) },
    ]};
    let tensor_type = TypeTensor { elem_type: Some(DataType::Float as i32), shape: Some(shape.clone()) };
    let mut ty_out = TypeProto::default();
    ty_out.value = Some(TypeValue::TensorType(tensor_type));
    let mut output_info = ValueInfoProto::default();
    output_info.name = Some(prev.clone());
    output_info.r#type = Some(ty_out);
    graph.output.push(output_info);

    // Debug: graph contents
    println!("graph: nodes={}, initializers={}", graph.node.len(), graph.initializer.len());

    // 6) Attach opset and encode
    let mut opset = OperatorSetIdProto::default();
    opset.domain = Some("".to_string());
    opset.version = Some(13);
    model.opset_import.push(opset);

    model.graph = Some(graph);
    model.encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neat::population::Population;
    use crate::onnx_generated::onnx::ModelProto;
    use prost::Message;

    #[test]
    fn test_export_genome_valid() {
        let pop = Population::new(&Default::default());
        let bytes = export_genome(&pop.genomes[0]);
        let model = ModelProto::decode(&*bytes).expect("Failed to decode ONNX bytes");
        assert!(model.graph.is_some(), "Graph is missing");
        assert_eq!(model.opset_import.len(), 1, "Expected exactly one opset_import");
        assert_eq!(model.opset_import[0].version.unwrap(), 13, "Unexpected opset version");
        let graph = model.graph.unwrap();
        assert_eq!(graph.name.unwrap(), "neat_model".to_string());
    }
}
