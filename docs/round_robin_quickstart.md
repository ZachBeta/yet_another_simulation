# Round-Robin Quickstart for Mid-Level SWE

- Existing NEAT runner (`sim_core/src/neat/runner.rs`)
- CLI harness (`tournament` command)

---

## Overview

Round-robin means each individual plays _k_ matches against distinct peers in the same generation. We’ll:

1. Add a `--round-robin k` flag to the CLI.
2. In `runner.rs`, sample _k_ opponents per genome.
3. Record win/loss/draw counts.
4. Compute a simple score or Elo per genome.

---

## CLI Flag in `main.rs`

In your CLI parser (e.g. `sim_core/src/bin/main.rs`):

```rust
use clap::Arg;

let matches = App::new("NEAT Tournament")
    .arg(Arg::new("round-robin")
         .long("round-robin")
         .takes_value(true)
         .help("Number of peers each genome plays per generation"))
    // ... other args
    .get_matches();

let k = matches.value_of("round-robin").unwrap_or("0").parse::<usize>()?;
```

Pass `k` into your tournament function.

---

## Sampling Opponents in `runner.rs`

At the start of each generation:

```rust
// genomes: Vec<Genome>
let pop_size = genomes.len();
let mut rng = rand::thread_rng();

for i in 0..pop_size {
    let mut opponents = Vec::new();
    while opponents.len() < k {
        let j = rng.gen_range(0..pop_size);
        if j != i && !opponents.contains(&j) {
            opponents.push(j);
        }
    }
    for &opp_idx in &opponents {
        let result = run_match(&genomes[i], &genomes[opp_idx]);
        record_result(i, opp_idx, result);
    }
}
```
