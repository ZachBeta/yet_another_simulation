fn main() {
    // Compile the official ONNX schema into Rust using prost-build
    prost_build::Config::new()
        .out_dir("src/onnx_generated")
        .compile_protos(&["proto/onnx/onnx.proto"], &["proto"])
        .expect("ONNX proto compilation failed");
}
