# Round-Robin Performance & Benchmarking Tutorial

Optimize your round-robin tournament for speed and stability. This guide covers instrumentation, profiling, parallelism, and benchmarks when each genome plays _k_ peers.

---

## 1. Instrumentation & Metrics

In `runner.rs`, wrap your generation loop:

```rust
use std::time::Instant;
// use hdrhistogram::Histogram;  // add to Cargo.toml

let mut match_times = Histogram::<u64>::new(3).unwrap();
let gen_start = Instant::now();

for i in 0..pop_size {
    for &j in &opponents[i] {
        let t0 = Instant::now();
        run_match(&genomes[i], &genomes[j]);
        match_times.record(t0.elapsed().as_micros() as u64).unwrap();
    }
}
let gen_ms = gen_start.elapsed().as_millis();
println!("Gen time: {} ms; matches: {}; p50: {} µs; p95: {} µs; p99: {} µs", 
    gen_ms, pop_size * k,
    match_times.value_at_percentile(50.0),
    match_times.value_at_percentile(95.0),
    match_times.value_at_percentile(99.0),
);
```

Also accumulate your inference timers (`INFER_TIME_NS`, `HTTP_TIME_NS`) per generation and log ratios.

---

## 2. Profiling Tools

- **Flamegraph**: Use `cargo flamegraph` or `pprof` crate to pinpoint hotspots.  
- **Criterion**: Write benchmarks:

```rust
#[bench]
fn bench_match(b: &mut Bencher) {
    let g1 = Genome::random(); let g2 = Genome::random();
    b.iter(|| run_match(&g1, &g2));
}
```

- **HTTP Tracing**: Enable Uvicorn logs or `tracing` in Python ONNX service for tail latency.

---

## 3. Parallel Execution

Leverage Rayon to distribute matches across threads:

```rust
use rayon::prelude::*;

(0..pop_size).into_par_iter().for_each(|i| {
    for &j in &opponents[i] {
        run_match(&genomes[i], &genomes[j]);
    }
});
```

Adjust thread count via `RAYON_NUM_THREADS`.

---

## 4. Batch Inference

If using remote ONNX:

1. Gather up to B agents’ input vectors.  
2. Single POST: `POST /infer` with `inputs: Vec<Vec<f32>>`.  
3. Distribute responses back to brains.

This trades minor latency for fewer HTTP calls.

---

## 5. Early Termination & Caching

- **Early exit**: if one side’s health <= 0, break match loop.  
- **Skip serialization**: disable JSONL writes during training; snapshot only sample matches.

---

## 6. Benchmark Workflow

1. **Baseline**: 1v1 vs naive, record gen time.  
2. **RR (k)**: `--round-robin k`, log new gen time.  
3. **Parallel**: add Rayon, compare.  
4. **Batch**: switch to batched ONNX, measure.  
5. **Chart**: plot gen_ms vs config.

---

## 7. Trade-Offs Summary

| Change             | Gen Time   | Effort | Notes                                  |
|--------------------|------------|--------|----------------------------------------|
| Single-thread k>1  | ×k         | Low    | Stable ranking but slow                |
| Rayon parallel     | ~/threads  | Med    | Speeds up at cost of thread overhead   |
| Batched HTTP calls | – I/O calls| Med    | Fewer requests, slight response lag    |
| Early exit         | – ticks    | Low    | Cuts wasted compute on lop-sided games |

By measuring at each stage, you’ll know exactly where to optimize—keeping your training pipeline fast and scalable.  

*Authored: 2025-05-18*
