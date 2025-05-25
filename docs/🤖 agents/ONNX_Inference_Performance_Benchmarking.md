# ONNX Inference Performance Benchmarking

*Authored: 2025-05-18*

This guide shows how to benchmark your Python-service ONNX inference versus the native Rust CPU path, both at micro and macro scales.

## 1. Objectives

- Measure per-call latency and tail behavior.
- Compare Rust `feed_forward` vs HTTP single-call vs HTTP batched inference.
- Evaluate overall impact on NEAT training throughput (gens/sec).

## 2. Modes to Compare

1. **CPU-only**: `genome.feed_forward(inputs)` in Rust.
2. **Remote single-call**: POST one input vector at a time to `/infer`.
3. **Remote batched**: POST batches of B inputs per request.

## 3. Microbenchmarks

### 3.1 Rust Criterion Bench

Create `benches/bench_feed_forward.rs`:
```rust
use criterion::{criterion_group, criterion_main, Criterion};
use sim_core::neat::genome::Genome;

fn bench_cpu(c: &mut Criterion) {
    let genome = Genome::random();
    let input = vec![0.0_f32; genome.input_size()];
    c.bench_function("feed_forward", |b| b.iter(|| genome.feed_forward(&input)));
}

criterion_group!(benches, bench_cpu);
criterion_main!(benches);
```

Run:
```bash
cargo bench --bench bench_feed_forward
```

### 3.2 HTTP Load Test

Write a small `wrk` script (e.g. `scripts/infer.lua`) to wrap JSON payload:
```lua
init = function(args)
    req = { inputs = { {} } }
end
request = function()
    req.inputs[1] = {} -- fill with random f32s
    return wrk.format("POST", "/infer", { ["Content-Type"] = "application/json" }, cjson.encode(req))
end
```

Run:
```bash
wrk -t4 -c100 -d30s -s scripts/infer.lua http://localhost:8000/infer
```

Record P50/P95/P99 latency and requests/sec.

## 4. In-Process Instrumentation

Leverage existing counters in `NeatBrain`:
- `INFER_TIME_NS`, `INFER_COUNT`
- `HTTP_TIME_NS`, `REMOTE_INFER_NS`

Add an HDR histogram to log tail percentiles per generation:
```rust
use hdrhistogram::Histogram;
let mut hist = Histogram::<u64>::new(3).unwrap();
INFER_TIME_NS.fetch_iter(|dt| hist.record(dt).unwrap());
println!("Inference latencies: p50={}, p95={}, p99={} µs",
    hist.value_at_percentile(50.0) / 1_000,
    hist.value_at_percentile(95.0) / 1_000,
    hist.value_at_percentile(99.0) / 1_000,
);
```

## 5. Macro Benchmark

Run short training sessions (1–5 gens) in three modes:
```bash
neat_train train --runs 5 --mode cpu
neat_train train --runs 5 --mode remote --batch 1
neat_train train --runs 5 --mode remote --batch 8
```
Compare gens/sec, sim tick µs, total inference ms.

## 6. System Metrics

Collect:
- CPU & thread utilization (e.g. `htop`).
- Memory and network I/O (e.g. Prometheus, cAdvisor).

## 7. Analysis & Decision

- If single-call adds >10–20 µs per inference, batching likely wins.
- Check tail percentiles: high P99 may indicate jitter in HTTP path.
- Balance complexity vs throughput: aim for >2× speedup to justify changes.

## 8. Next Steps

1. If batching helps: implement buffered requests in `NeatBrain`.
2. Otherwise, stick with Rust CPU inference and optimize your genomes.
3. Document final metrics and update performance guides.

---

Keep this sheet updated as you iterate on ONNX inference strategies.
