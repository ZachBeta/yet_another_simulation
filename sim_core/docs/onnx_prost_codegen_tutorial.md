# Integrating the Official ONNX Schema with Prost in Rust

This tutorial walks you through vendoring the official `onnx.proto`, generating Rust types via `prost-build`, and updating your exporter to produce fully compliant ONNX models that load in ONNX Runtime.

Audience: Mid-level SWE familiar with Rust, Protobuf, and basic ML model formats.

---

## 1. Vendor the ONNX `.proto`

1. Create a `proto/onnx` directory under your `sim_core` crate:
   ```bash
   mkdir -p sim_core/proto/onnx
   ```
2. Download the official schema:
   ```bash
   curl -L https://raw.githubusercontent.com/onnx/onnx/main/onnx/onnx.proto \
     -o sim_core/proto/onnx/onnx.proto
   ```
3. (Optional) Pin to a specific commit or tag by saving alongside `.proto` and `.proto.sha`.

## 2. Add `build.rs` for `prost-build`

In `sim_core/build.rs`:
```rust
fn main() {
    prost_build::Config::new()
        .out_dir("src/onnx_generated")
        .compile_protos(&["proto/onnx/onnx.proto"], &["proto"])
        .expect("ONNX proto compilation failed");
}
```

- **`out_dir`**: where generated `.rs` files will live (e.g. `src/onnx_generated`).
- **`compile_protos`**: point to your vendored `.proto` and include path.

## 3. Update `Cargo.toml`

```toml
[build-dependencies]
prost-build = "0.10"

[dependencies]
prost = "0.10"
prost-types = "0.10"
# remove any hand-rolled onnx_minimal
```

Ensure the crate picks up `build.rs` automatically (Cargo will run it).

## 4. Import the Generated Types

Replace your minimal ONNX definitions:

```diff
-use crate::neat::onnx_minimal::{ModelProto, GraphProto, ...};
+use onnx_generated::onnx::ModelProto;
+use onnx_generated::onnx::GraphProto;
+// etc. for OperatorSetIdProto, NodeProto, TensorProto, ValueInfoProto...
```

- Generated types live under `mod onnx_generated { pub mod onnx { ... } }`.
- Adjust any module paths accordingly.

## 5. Update Your Exporter

In `src/neat/onnx_exporter.rs`:

1. Remove `onnx_minimal` imports.
2. Import generated `onnx_generated::onnx::*` types.
3. Serialize with `model.encode_to_vec()`, same as before.

Your export logic remains unchanged, but now writes a spec-compliant ONNX model.

## 6. Build & Validate

1. Run:
   ```bash
   cargo clean && cargo build
   ```
2. Generate a model:
   ```bash
   cargo run --example export_model
   ```
3. In the Python service directory:
   ```bash
   python3 - << 'PY'
import onnx
m = onnx.load("model.onnx")
print(m.ir_version, [o.version for o in m.opset_import])
PY
   ```
4. Start ONNX Runtime:
   ```bash
   uvicorn app:app --reload --host 127.0.0.1 --port 8000
   ```
   No more parse errors!

## 7. Clean Up

- Remove `onnx_minimal.rs` and related code.
- Update docs to reference generated types.

---

By following these steps, your Rust exporter now uses the exact same Protobuf layout as ONNX Runtime, eliminating boundary mismatches and ensuring a valid, interoperable model. If you run into any issues, double-check field numbers in `onnx.proto` and the path passed to `prost-build`.
