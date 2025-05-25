# Performance Benchmarking and Optimization Guide

This guide provides comprehensive instructions for benchmarking and optimizing the performance of the simulation, with a focus on ONNX inference in Python services and efficient concurrency patterns.

## Table of Contents

1. [Benchmarking Objectives](#1-benchmarking-objectives)
2. [Performance Optimization Strategies](#2-performance-optimization-strategies)
   - [Concurrency & Threading](#21-concurrency--threading)
   - [Round-Robin Tournament Optimization](#22-round-robin-tournament-optimization)
3. [Benchmarking Modes](#3-benchmarking-modes)
4. [Microbenchmarks](#4-microbenchmarks)
   - [Rust Criterion Benchmarks](#41-rust-criterion-benchmarks)
   - [HTTP Load Testing](#42-http-load-testing)
5. [In-Process Instrumentation](#5-in-process-instrumentation)
6. [Macro Benchmarking](#6-macro-benchmarking)
7. [System Metrics Collection](#7-system-metrics-collection)
8. [Analysis and Decision Making](#8-analysis-and-decision-making)
9. [Best Practices](#9-best-practices)

## 2. Performance Optimization Strategies

### 2.1 Concurrency & Threading

To minimize end-to-end training and tournament cycle time, we can exploit multi-core parallelism across both Rust and Python components. By default, use all but one CPU core (`n-1`) to leave headroom for the OS.

#### Default Thread Count

**Rust (Rayon global pool):**
```rust
use rayon::ThreadPoolBuilder;
use num_cpus;

fn configure_global_thread_pool() {
    let threads = num_cpus::get().saturating_sub(1);
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
}
```

**Environment variable:**
```bash
export RAYON_NUM_THREADS=$(( $(nproc) - 1 ))
```

#### Parallelize Tournament Evaluations

Evaluate genomes in parallel instead of sequentially:

```rust
use rayon::prelude::*;

fn evaluate_population(genomes: &[Genome], config: &Config) -> Vec<f64> {
    genomes
        .par_iter()
        .map(|genome| evaluate_genome(genome, config))
        .collect()
}
```

This yields near-linear speedups up to `n-1` threads.

#### Batch-Level Concurrency to Python Service

Dispatch multiple batched inference calls in parallel using async/await:

```rust
use tokio::runtime::Runtime;
use reqwest::Client;

async fn parallel_batch_inference(
    batches: Vec<Vec<f32>>,
    url: &str,
    concurrency: usize,
) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut tasks = Vec::with_capacity(concurrency);
    
    for batch in batches.into_iter() {
        let client = client.clone();
        let url = url.to_string();
        
        tasks.push(tokio::spawn(async move {
            let res = client.post(&url)
                .json(&json!({ "inputs": [batch] }))
                .send()
                .await?;
            res.json::<Vec<Vec<f32>>>().await
        }));
    }
    
    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        results.push(task.await??);
    }
    
    Ok(results)
}
```

### 2.2 Round-Robin Tournament Optimization

Optimize round-robin tournament performance with proper instrumentation and parallelism:

#### Instrumentation & Metrics

Track match times and other performance metrics:

```rust
use std::time::Instant;
use hdrhistogram::Histogram;

fn run_tournament(genomes: &[Genome], k: usize) {
    let pop_size = genomes.len();
    let mut match_times = Histogram::<u64>::new(3).unwrap();
    let gen_start = Instant::now();
    
    // Generate opponent indices (simplified)
    let opponents: Vec<Vec<usize>> = (0..pop_size)
        .map(|i| (0..k).map(|j| (i + j + 1) % pop_size).collect())
        .collect();
    
    // Process matches in parallel
    let results: Vec<_> = (0..pop_size)
        .into_par_iter()
        .map(|i| {
            let mut wins = 0;
            for &j in &opponents[i] {
                let t0 = Instant::now();
                let result = run_match(&genomes[i], &genomes[j]);
                let _ = match_times.record(t0.elapsed().as_micros() as u64);
                if result == MatchResult::Win {
                    wins += 1;
                }
            }
            (i, wins)
        })
        .collect();
    
    // Log performance metrics
    let gen_ms = gen_start.elapsed().as_millis();
    println!(
        "Tournament stats - Time: {}ms, Match count: {}, p50: {}µs, p95: {}µs, p99: {}µs",
        gen_ms,
        pop_size * k,
        match_times.value_at_percentile(50.0),
        match_times.value_at_percentile(95.0),
        match_times.value_at_percentile(99.0)
    );
    
    // Process results...
}
```

#### Profiling Tools

1. **Flamegraph**
   Generate flamegraphs to identify hotspots:
   ```bash
   # Install flamegraph
   cargo install flamegraph
   
   # Generate flamegraph
   cargo flamegraph --bench your_benchmark
   ```

2. **Criterion Benchmarks**
   ```rust
   use criterion::{criterion_group, criterion_main, Criterion, Bencher};
   
   fn bench_match(b: &mut Bencher) {
       let g1 = Genome::random();
       let g2 = Genome::random();
       b.iter(|| run_match(&g1, &g2));
   }
   
   criterion_group!(benches, bench_match);
   criterion_main!(benches);
   ```

3. **HTTP Tracing**
   Enable detailed tracing in your Python ONNX service:
   ```python
   # In your FastAPI app
   from fastapi import FastAPI
   import logging
   
   logging.basicConfig(level=logging.INFO)
   logger = logging.getLogger(__name__)
   
   app = FastAPI()
   
   @app.middleware("http")
   async def log_requests(request, call_next):
       start_time = time.time()
       response = await call_next(request)
       process_time = (time.time() - start_time) * 1000
       logger.info(f"Request: {request.method} {request.url} - {response.status_code} - {process_time:.2f}ms")
       return response
   ```

## 1. Benchmarking Objectives

- Measure and compare inference latency across different implementations
- Evaluate the impact of batching on throughput
- Identify performance bottlenecks in the inference pipeline
- Ensure the system meets performance requirements for production use

## 2. Benchmarking Modes

### 2.1 CPU-Only Mode
- Direct Rust implementation using `feed_forward`
- Serves as the baseline for performance comparison

### 2.2 Remote Single-Call Mode
- HTTP POST with one input vector per request
- Measures overhead of network and serialization

### 2.3 Remote Batched Mode
- HTTP POST with multiple input vectors per request
- Evaluates the benefits of request batching

## 3. Microbenchmarks

### 3.1 Rust Criterion Benchmarks

Create a benchmark file at `benches/bench_feed_forward.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use sim_core::neat::genome::Genome;

fn bench_cpu(c: &mut Criterion) {
    // Initialize genome with random weights
    let genome = Genome::random();
    let input = vec![0.0_f32; genome.input_size()];
    
    c.bench_function("feed_forward", |b| {
        b.iter(|| {
            genome.feed_forward(&input)
        })
    });
}

criterion_group!(benches, bench_cpu);
criterion_main!(benches);
```

Run the benchmark:

```bash
# Run with debug symbols for accurate profiling
RUSTFLAGS="-C debuginfo=1" cargo bench --bench bench_feed_forward
```

### 3.2 HTTP Load Testing

#### Using `wrk`

Create a Lua script at `scripts/infer.lua`:

```lua
-- Initialize request template
init = function(args)
    local req = { inputs = { {} } }
    -- Initialize with random inputs matching your model's expected format
    for i = 1, 100 do  -- Adjust size to match your input dimensions
        req.inputs[1][i] = math.random() * 2 - 1  # Random values in [-1, 1]
    end
    wrk.method = "POST"
    wrk.headers["Content-Type"] = "application/json"
    wrk.body = require("cjson").encode(req)
end

-- No need to modify the request function
request = function()
    return wrk.format()
end
```

Run the load test:

```bash
# -t: number of threads
# -c: number of connections
# -d: test duration
# -s: path to Lua script
wrk -t4 -c100 -d30s -s scripts/infer.lua http://localhost:8000/infer
```

#### Using `k6` (Alternative)

For more advanced scenarios, consider using `k6`:

```javascript
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
};

const url = 'http://localhost:8000/infer';
const payload = JSON.stringify({
  inputs: [Array(100).fill(0).map(() => Math.random() * 2 - 1)]
});

const params = {
  headers: {
    'Content-Type': 'application/json',
  },
};

export default function () {
  const res = http.post(url, payload, params);
  check(res, {
    'is status 200': (r) => r.status === 200,
  });
}
```

## 4. In-Process Instrumentation

Add performance monitoring to your Rust code:

```rust
use hdrhistogram::Histogram;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// Global metrics
static INFER_TIME_NS: AtomicU64 = AtomicU64::new(0);
static INFER_COUNT: AtomicU64 = AtomicU64::new(0);

fn record_inference_time<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    
    INFER_TIME_NS.fetch_add(
        elapsed.as_nanos() as u64,
        Ordering::Relaxed
    );
    INFER_COUNT.fetch_add(1, Ordering::Relaxed);
    
    result
}

// Example usage
fn process_inference(input: &[f32]) -> Vec<f32> {
    record_inference_time(|| {
        // Your inference code here
        vec![0.0; 10] // Dummy output
    })
}

// Log metrics periodically
fn log_metrics() {
    let total_ns = INFER_TIME_NS.load(Ordering::Relaxed);
    let count = INFER_COUNT.load(Ordering::Relaxed);
    
    if count > 0 {
        let avg_ns = total_ns / count;
        println!("Inference metrics - Count: {}, Avg: {} µs", 
                count, avg_ns as f64 / 1000.0);
    }
}
```

## 5. Macro Benchmarking

Run end-to-end training benchmarks:

```bash
# CPU-only baseline
neat_train train --runs 5 --mode cpu

# Remote inference with batch size 1
neat_train train --runs 5 --mode remote --batch 1

# Remote inference with batch size 8
neat_train train --runs 5 --mode remote --batch 8
```

Compare the following metrics:
- Generations per second
- Average simulation tick time
- Total inference time per generation
- Memory usage

## 6. System Metrics Collection

### 6.1 Using Prometheus and Grafana

1. Set up Prometheus to scrape metrics from your service
2. Configure Grafana dashboards to visualize performance data
3. Track key metrics:
   - Request rate
   - Latency percentiles
   - Error rates
   - Resource utilization (CPU, memory, network)

### 6.2 Using `htop` and `perf`

For detailed CPU analysis:

```bash
# Monitor CPU and thread usage
htop

# Profile CPU usage with perf
perf record -F 99 -p $(pgrep your_service) -g -- sleep 30
perf report -n --stdio
```

## 7. Analysis and Decision Making

1. **Performance Targets**
   - Define acceptable latency thresholds (e.g., < 1ms P99 for inference)
   - Set throughput requirements (e.g., > 10,000 RPS per instance)

2. **Bottleneck Analysis**
   - CPU-bound: Optimize model or increase compute resources
   - I/O-bound: Improve serialization or network stack
   - Memory-bound: Optimize data structures or batch sizes

3. **Decision Matrix**

   | Factor | Weight | CPU-Only | Remote (Batch=1) | Remote (Batch=8) |
   |--------|--------|----------|-----------------|------------------|
   | Latency | 40% | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
   | Throughput | 30% | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
   | Resource Usage | 20% | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
   | Complexity | 10% | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |

## 8. Best Practices

1. **Consistent Testing Environment**
   - Use the same hardware for comparable results
   - Minimize background processes
   - Warm up the system before measurements

2. **Statistical Significance**
   - Run multiple test iterations
   - Report confidence intervals
   - Identify and remove outliers

3. **Documentation**
   - Record hardware specifications
   - Note software versions
   - Document test parameters and configurations

4. **Continuous Monitoring**
   - Integrate benchmarks into CI/CD
   - Set up performance regression alerts
   - Track metrics over time

## 9. Troubleshooting

### High Latency in Remote Mode
- Check network latency between services
- Verify serialization/deserialization overhead
- Profile the HTTP server implementation

### Low Throughput
- Increase batch sizes
- Optimize model architecture
- Scale horizontally with load balancing

### Memory Issues
- Monitor for memory leaks
- Adjust batch sizes based on available memory
- Profile memory usage with tools like `valgrind` or `heaptrack`

## 10. Example Results

### Microbenchmark Results

| Implementation | P50 (ms) | P95 (ms) | P99 (ms) | RPS |
|----------------|----------|----------|----------|-----|
| CPU-Only | 0.05 | 0.08 | 0.12 | 50,000 |
| Remote (Batch=1) | 1.2 | 2.5 | 5.0 | 800 |
| Remote (Batch=8) | 2.0 | 4.0 | 8.0 | 4,000 |

### Training Throughput

| Configuration | Gens/sec | Speedup |
|--------------|----------|---------|
| CPU-Only | 35.2 | 1.0x |
| Remote (Batch=1) | 12.5 | 0.36x |
| Remote (Batch=8) | 28.7 | 0.82x |

## Conclusion

This guide provides a comprehensive approach to benchmarking ONNX inference performance. By following these steps, you can make informed decisions about deployment configurations and identify areas for optimization.

For production deployments, consider:
1. Using the CPU-Only mode for lowest latency
2. Implementing request batching for higher throughput
3. Monitoring performance metrics in real-time
4. Setting up alerts for performance regressions
