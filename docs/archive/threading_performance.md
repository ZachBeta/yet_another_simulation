# Concurrency & Threading Performance Tuning

To minimize end-to-end training and tournament cycle time, we can exploit multi-core parallelism across both Rust and Python components. By default, use all but one CPU core (`n-1`) to leave headroom for the OS.

---

## Default Thread Count

**Rust (Rayon global pool):**
```rust
use rayon::ThreadPoolBuilder;
let threads = num_cpus::get().saturating_sub(1);
ThreadPoolBuilder::new()
    .num_threads(threads)
    .build_global()
    .unwrap();
```  
**Environment variable:**
```bash
export RAYON_NUM_THREADS=$(( $(nproc) - 1 ))
```

---

## 1) Parallelize Tournament Evaluations (Rust)

Evaluate genomes in parallel instead of sequentially:
```rust
use rayon::prelude::*;
let results: Vec<_> = genomes
    .par_iter()
    .map(|genome| evaluate_genome(genome, &config))
    .collect();
```
This yields near-linear speedups up to `n-1` threads.

---

## 2) Batch-Level Concurrency to Python Service

Dispatch multiple batched inference calls in parallel:

```rust
use tokio::runtime::Runtime;
use reqwest::Client;

let rt = Runtime::new().unwrap();
let client = Client::new();
let futures: Vec<_> = (0..threads)
    .map(|_| {
        client.post(&url)
            .json(&payload)
            .send()
    })
    .collect();
rt.block_on(async {
    futures::future::join_all(futures).await;
});
```

Also run your Python FastAPI with multiple workers:
```bash
uvicorn app:app --workers <n-1> --host 0.0.0.0 --port 8000
```

---

## 3) Scale the Python Service Itself

- **Run in executor:**
  ```python
  @app.post("/infer_batch")
  async def infer_batch(req: BatchRequest):
      return await asyncio.get_event_loop().run_in_executor(
          None,
          lambda: session.run(...)
      )
  ```
- **Uvicorn workers:** fork multiple processes with `--workers`.

---

## 4) Hybrid: Tournament + Inference Pool

Combine (1) and (2): each parallel genome-eval thread reuses its own HTTP client and issues batched calls concurrently. This maximizes CPU and I/O saturation.

---

## 5) Rust Threadpool for GA Operations

Pipeline selection, crossover, and mutation while batches are inferring:

```rust
use rayon::prelude::*;
// e.g., parallel mutate:
parents.par_iter()
    .map(|p| mutate(p))
    .collect::<Vec<_>>();
```

By overlapping GA CPU work with inference, you reduce idle time and improve overall throughput.
