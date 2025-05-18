# Round-Robin Tournament Tutorial for Mid-Level SWE

This tutorial shows how to implement an intra-population round-robin format in your NEAT simulation. Each genome plays _k_ random peers per generation, producing more stable rankings.

---

## 1. Prerequisites

- Rust & Cargo
- Existing NEAT runner (`sim_core/src/neat/runner.rs`)
- CLI harness (`tournament` command)

---

## 2. Overview

Round-robin means each individual plays _k_ matches against distinct peers in the same generation. We’ll:

1. Add a `--round-robin k` flag to the CLI.
2. In `runner.rs`, sample _k_ opponents per genome.
3. Record win/loss/draw counts.
4. Compute a simple score or Elo per genome.

---

## 3. CLI Flag in `main.rs`

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

## 4. Sampling Opponents in `runner.rs`

At start of generation:

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

- `run_match` returns `Win`, `Loss`, or `Draw`.
- `record_result` should increment counters on both sides.

---

## 5. Tracking Results

Extend your stats struct:

```rust
struct Stats { wins: usize, losses: usize, draws: usize }
let mut stats = vec![Stats::default(); pop_size];

fn record_result(a: usize, b: usize, res: MatchResult) {
    match res {
        MatchResult::Win  => { stats[a].wins += 1; stats[b].losses += 1; }
        MatchResult::Loss => { stats[a].losses += 1; stats[b].wins += 1; }
        MatchResult::Draw => { stats[a].draws += 1; stats[b].draws += 1; }
    }
}
```

---

## 6. Scoring & Selection

Compute a raw score per genome:

```rust
let score = stats[i].wins as f32
          + 0.5 * stats[i].draws as f32;
```

Optionally, feed these scores into an Elo or TrueSkill library for refined ranking before selection.

---

## 7. Integrate into Evolution Loop

Replace your single-champion tournament call with the loop above. After scoring:

```rust
// select top N by score or Elo
pop.select_by_score(&scores, survivor_count);
pop.reproduce(&mut rng);
```

---

## 8. Testing & Validation

1. Run with `cargo run -- --round-robin 5` on a small toy population.  
2. Print stats to verify each genome played 5 games.  
3. Check that no self-matches occur and opponents are unique.

---

## 9. Next Steps

- Visualize score distributions per generation.  
- Experiment with different _k_ values to balance compute vs. ranking stability.  
- Integrate Elo rating for dynamic update of match pairings.

*Authored: 2025-05-18*
