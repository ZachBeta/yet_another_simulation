# Hand-Rolling a Minimal ONNX Exporter in Rust

This tutorial shows how to implement a simple ONNX exporter for a strictly feed-forward NEAT `Genome` by hand-crafting only the ONNX message types you need.

Audience: Mid-level SWE who wants to learn the ONNX wire format without wrestling full protobuf codegen.

---

## 1. Motivation

The official ONNX schema is ~40K lines of `.proto` and requires complex build setup. For a controlled proof-of-concept we can hand-define the few messages we actually use:

- `ModelProto`
- `GraphProto`
- `NodeProto`
- `TensorProto`
- `ValueInfoProto`
- `TensorShapeProto` + `Dimension`
- `DataType` enum

With `prost` and `prost‐derive`, this is only ~100 lines of Rust.

## 2. Update Dependencies

In `Cargo.toml`:

```toml
[dependencies]
prost = "0.10"
prost-types = "0.10"

# remove `build-dependencies` and old `protobuf` / ONNX crates
```

Run:

```bash
cargo clean
cargo build
```

## 3. Define Minimal ONNX Types

Create `src/neat/onnx_minimal.rs`:

```rust
// src/neat/onnx_minimal.rs
use prost::Message;

/// ONNX data types (we only use FLOAT here)
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum DataType {
    Float = 1,
    // ... other values omitted
}

/// Shape dimension (value or symbolic param)
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

/// Tensor shape
#[derive(Clone, PartialEq, prost::Message)]
pub struct TensorShapeProto {
    #[prost(message, repeated, tag = "1")]
    pub dim: Vec<TensorShapeProto_Dimension>,
}

/// ValueInfo: name + type (only tensor_type)
#[derive(Clone, PartialEq, prost::Message)]
pub struct ValueInfoProto {
    #[prost(string, tag = "1")] pub name: String,
    #[prost(message, optional, tag = "2")] pub r#type: Option<TypeProto>,
}

/// TypeProto: only tensor_type used
#[derive(Clone, PartialEq, prost::Message)]
pub struct TypeProto {
    #[prost(message, optional, tag = "1")] pub tensor_type: Option<TypeProto_Tensor>,
}
#[derive(Clone, PartialEq, prost::Message)]
pub struct TypeProto_Tensor {
    #[prost(enumeration = "DataType", tag = "1")] pub elem_type: i32,
    #[prost(message, optional, tag = "2")] pub shape: Option<TensorShapeProto>,
}

/// Tensor initializer (weights/bias)
#[derive(Clone, PartialEq, prost::Message)]
pub struct TensorProto {
    #[prost(string, tag = "1")] pub name: String,
    #[prost(enumeration = "DataType", tag = "2")] pub data_type: i32,
    #[prost(int64, repeated, tag = "3")] pub dims: Vec<i64>,
    #[prost(bytes, tag = "5")] pub raw_data: Vec<u8>,
}

/// Compute node (MatMul, Add, Relu)
#[derive(Clone, PartialEq, prost::Message)]
pub struct NodeProto {
    #[prost(string, repeated, tag = "1")] pub input: Vec<String>,
    #[prost(string, repeated, tag = "2")] pub output: Vec<String>,
    #[prost(string, tag = "3")] pub op_type: String,
}

/// Graph: inputs, initializers, nodes, outputs
#[derive(Clone, PartialEq, prost::Message)]
pub struct GraphProto {
    #[prost(string, tag = "1")] pub name: String,
    #[prost(message, repeated, tag = "2")] pub input: Vec<ValueInfoProto>,
    #[prost(message, repeated, tag = "4")] pub initializer: Vec<TensorProto>,
    #[prost(message, repeated, tag = "5")] pub node: Vec<NodeProto>,
    #[prost(message, repeated, tag = "7")] pub output: Vec<ValueInfoProto>,
}

/// Model wrapper
#[derive(Clone, PartialEq, prost::Message)]
pub struct ModelProto {
    #[prost(int64, tag = "1")] pub ir_version: i64,
    #[prost(message, optional, tag = "2")] pub graph: Option<GraphProto>,
}
```

## 4. Refactor Exporter

In `src/neat/onnx_exporter.rs`:

```diff
-use crate::neat::onnx_exporter::{…};
-use onnx::proto::{…};
+use crate::neat::onnx_minimal::{ModelProto, GraphProto, NodeProto, TensorProto, ValueInfoProto};
+use crate::neat::onnx_minimal::DataType;

 pub fn export_genome(genome: &Genome) -> Vec<u8> {
     let mut model = ModelProto::default();
     model.ir_version = 7;
     let mut graph = GraphProto::default();
     graph.name = "neat_model".into();
     // … same logic, but set `graph.input.push(...)`, etc. …

-    model.set_graph(graph);
-    model.write_to_bytes().unwrap()
+    model.graph = Some(graph);
+    model.encode_to_vec()
 }
```

## 5. Cleanup & Test

- Remove `build.rs` and `proto/onnx/` folder
- Remove `onnx`, `protobuf`, `onnx-protobuf` deps from `Cargo.toml`
- Run:
  ```bash
  cargo clean
  cargo build
  cargo test
  ```

---

You now have a pure-Rust ONNX exporter (~100 lines of schemas). This is lightweight, educational, and fully under your control.
