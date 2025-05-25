# neat_train Benchmark Tutorial

This tutorial walks a mid-level SWE through adding CPU vs GPU (Apple M1/MPS) benchmarking to the `neat_train` binary in `sim_core`.

## 1. Prerequisites

- Rust toolchain (1.65+)
- `cargo` and `rustup` installed
- Python 3.9+ with FastAPI service running on M1 (see `python_onnx_service/app.py`)
- `reqwest`, `clap`, and `rusqlite` in `Cargo.toml`

## 2. Add a `device` Flag with Clap

1. In `sim_core/src/main.rs`, add to `Cargo.toml`:

   ```toml
   [dependencies]
   clap = { version = "4.0", features = ["derive"] }
   ```

2. In `main.rs`:

   ```rust
   use clap::Parser;
   
   #[derive(Parser)]
   #[clap(author, version)]
   struct Opts {
     /// device: cpu or mps
     #[clap(long, default_value = "cpu")]
     device: String,
     /// number of runs
     #[clap(long, default_value = "10")]
     runs: usize,
   }
   
   fn main() -> anyhow::Result<()> {
     let opts = Opts::parse();
     println!("Running on: {}", opts.device);
     // ...
   }
   ```

## 3. Implement CPU Inference Path

Inside your loop in `main()`, match on `opts.device`:

```rust
let result = match opts.device.as_str() {
  "cpu" => run_cpu_inference(&model_path)?,
  _ => panic!("Unknown device"),
};
```

Provide a helper:

```rust
fn run_cpu_inference(path: &str) -> Result<InferenceResult> {
  let session = onnxruntime::environment::Environment::builder()?
    .with_name("neat")
    .build()?;
  let mut session = session
    .new_session_builder()?
    .with_model_from_file(path)?;
  // load input, run inference
  // return metrics
}
```

## 4. Hook up GPU/MPS via Python Service

For `"mps"` branch, call your FastAPI service:

```rust
fn run_mps_inference(path: &str) -> Result<InferenceResult> {
  let bytes = std::fs::read(path)?;
  let client = reqwest::blocking::Client::new();
  let resp: InferenceResult = client
    .post("http://127.0.0.1:8000/infer?device=mps")
    .body(bytes)
    .send()?  
    .json()?;
  Ok(resp)
}
```

## 5. Time and Log Runs

Wrap each call:

```rust
for i in 0..opts.runs {
  let start = std::time::Instant::now();
  let res = match opts.device.as_str() {
    "cpu" => run_cpu_inference(&path)?,
    "mps" => run_mps_inference(&path)?,
    _ => unreachable!(),
  };
  let ms = start.elapsed().as_secs_f64() * 1000.0;
  println!(r#"{{"device":"{}","run":{},"latency_ms":{}}}"#, opts.device, i, ms);
}
```

Optionally write these JSON logs into a SQLite table via `rusqlite`:

```rust
let conn = rusqlite::Connection::open("benchmark.db")?;
conn.execute(
  "CREATE TABLE IF NOT EXISTS bench (device TEXT, run INT, latency REAL)",
  []
)?;
conn.execute(
  "INSERT INTO bench (device, run, latency) VALUES (?1, ?2, ?3)",
  &[&opts.device, &(i as i32), &ms]
)?;
```

## 6. Run and Analyze

1. Start your Python service:
   ```bash
   cd python_onnx_service
   uvicorn app:app --reload
   ```
2. Run benchmarks:
   ```bash
   cargo run --bin neat_train -- --device cpu --runs 20
   cargo run --bin neat_train -- --device mps --runs 20
   ```
3. Inspect `benchmark.db` with `sqlite3` or build queries to compare distributions.

## 7. Next Steps

- Add peak-memory measurement (use `sysinfo` or return from Python).
- Export profiling traces (`.json`) and view in Chrome tracing.
- Automate comparisons in a Rust/Streamlit dashboard.
