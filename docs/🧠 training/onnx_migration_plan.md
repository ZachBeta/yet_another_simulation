# ONNX Exporter Migration Plan

This document outlines the step-by-step roadmap for replacing the hand-rolled minimal schema with the official ONNX types in our Rust project.

1. Remove the old minimal schema
   - Delete `src/neat/onnx_minimal.rs`
   - Remove `pub mod onnx_minimal;` from `src/neat/mod.rs`

2. Expose the generated module
   - Add `pub mod onnx_generated;` in the crate root (`src/lib.rs`)

3. Swap imports in `onnx_exporter.rs`
   - Import from `crate::onnx_generated::onnx::{…}` and its sub-modules instead of `onnx_minimal`

4. Refactor `export_genome` to use real protobuf types
   - Call `XxxProto::default()` for each message (`ModelProto`, `GraphProto`, `NodeProto`, `TensorProto`, `ValueInfoProto`, `OperatorSetIdProto`)
   - Wrap every optional field in `Some(...)` (strings, numbers)
   - Assign repeated fields directly to `Vec<…>`
   - Use the generated one-of enums (`type_proto::Value::Tensor(...)`, etc.)
   - Set `model.ir_version = Some(7)`, `graph.name = Some("neat_model".to_string())`, populate `input`, `initializer`, `node`, `output`, then push an opset import with `domain = Some("")` and `version = Some(13)`

5. Update tests in `onnx_exporter.rs`
   - Decode via `ModelProto::decode(&bytes).unwrap()`
   - Unwrap options in assertions (e.g. `model.opset_import[0].version.unwrap()`, `graph.name.unwrap()`)

6. Adjust genome-level ONNX tests
   - Remove references to `onnx_minimal` in `genome.rs` tests
   - Add/modify a `to_onnx()` test that decodes into `ModelProto` and checks `graph.name.unwrap()`

7. Run final verification
   - Execute `cargo test` to confirm compilation succeeds and all tests pass

---

Once these steps are complete, your ONNX exporter will produce fully compliant models compatible with ONNX Runtime.
