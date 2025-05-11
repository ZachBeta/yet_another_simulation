// src/neat/onnx_minimal.rs
// Minimal ONNX types for export

/// ONNX data types (we only use FLOAT here)
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum DataType {
    Float = 1,
    // Other types (INT64, etc.) can be added if needed
}

/// Shape dimension (value or symbolic parameter)
#[derive(Clone, PartialEq, prost::Message)]
pub struct TensorShapeProto_Dimension {
    #[prost(oneof = "tensor_shape_proto_dimension::Dim", tags = "1, 2")]
    pub dim: Option<tensor_shape_proto_dimension::Dim>,
}

pub mod tensor_shape_proto_dimension {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Dim {
        #[prost(int64, tag = "1")] DimValue(i64),
        #[prost(string, tag = "2")] DimParam(String),
    }
}

// Tensor shape
#[derive(Clone, PartialEq, prost::Message)]
pub struct TensorShapeProto {
    #[prost(message, repeated, tag = "1")]
    pub dim: Vec<TensorShapeProto_Dimension>,
}

// TypeProto and nested Tensor type
#[derive(Clone, PartialEq, prost::Message)]
pub struct TypeProto {
    #[prost(message, optional, tag = "1")]
    pub tensor_type: Option<TypeProto_Tensor>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct TypeProto_Tensor {
    #[prost(enumeration = "DataType", tag = "1")]
    pub elem_type: i32,
    #[prost(message, optional, tag = "2")]
    pub shape: Option<TensorShapeProto>,
}

// Value information for graph I/O
#[derive(Clone, PartialEq, prost::Message)]
pub struct ValueInfoProto {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(message, optional, tag = "2")]
    pub r#type: Option<TypeProto>,
}

// Tensor initializer (weights, biases)
#[derive(Clone, PartialEq, prost::Message)]
pub struct TensorProto {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(enumeration = "DataType", tag = "2")]
    pub data_type: i32,
    #[prost(int64, repeated, tag = "3")]
    pub dims: Vec<i64>,
    #[prost(bytes, tag = "5")]
    pub raw_data: Vec<u8>,
}

// Compute node
#[derive(Clone, PartialEq, prost::Message)]
pub struct NodeProto {
    #[prost(string, repeated, tag = "1")]
    pub input: Vec<String>,
    #[prost(string, repeated, tag = "2")]
    pub output: Vec<String>,
    #[prost(string, tag = "3")]
    pub op_type: String,
}

// Graph container
#[derive(Clone, PartialEq, prost::Message)]
pub struct GraphProto {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(message, repeated, tag = "2")]
    pub input: Vec<ValueInfoProto>,
    #[prost(message, repeated, tag = "4")]
    pub initializer: Vec<TensorProto>,
    #[prost(message, repeated, tag = "5")]
    pub node: Vec<NodeProto>,
    #[prost(message, repeated, tag = "7")]
    pub output: Vec<ValueInfoProto>,
}

// Operator set ID
#[derive(Clone, PartialEq, prost::Message)]
pub struct OperatorSetIdProto {
    #[prost(string, tag = "1")]
    pub domain: String,
    #[prost(int64, tag = "2")]
    pub version: i64,
}

// Model wrapper
#[derive(Clone, PartialEq, prost::Message)]
pub struct ModelProto {
    #[prost(int64, tag = "1")]
    pub ir_version: i64,
    #[prost(message, optional, tag = "7")]
    pub graph: Option<GraphProto>,
    #[prost(message, repeated, tag = "8")]
    pub opset_import: Vec<OperatorSetIdProto>,
}
