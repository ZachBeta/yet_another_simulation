# NEAT Implementation Guide for Mid-Level SWE

This guide walks you through implementing full NEAT in `sim_core`, covering mutation, crossover, speciation, and evolution loop.

## Prerequisites
- Rust project with `Genome`, `Population`, `Config`, and `run_match` in place.
- `rand`, `serde`, and `wasm-bindgen` crates available.

---
## 1. Gene-Level Mutation (`Genome::mutate`)

### 1.1 Add Connection Mutation
1. Pick two existing nodes `a`, `b` not already directly connected.
2. Generate new `ConnGene { in_node: a, out_node: b, weight: rand(), enabled: true, innovation: next_id() }`.
3. Push into `self.conns`.

### 1.2 Add Node Mutation
1. Select an enabled connection `c`.
2. Disable it and assign `innovation`.
3. Insert new `NodeGene { id: new_node_id(), Hidden }`.
4. Add two connections:
   ```rust
   // c: ConnGene
   let w1 = 1.0;
   let w2 = c.weight;
   self.conns.push(ConnGene { in_node: c.in_node, out_node: new_id, weight: w1, enabled: true, innovation: next1 });
   self.conns.push(ConnGene { in_node: new_id, out_node: c.out_node, weight: w2, enabled: true, innovation: next2 });
   ```

---
## 2. Crossover (`Genome::crossover`)
1. Align parent genes by `innovation` number.
2. Inherit matching genes randomly; keep disjoint/excess from fitter parent.
3. Clone into child:
   ```rust
   let child = Genome {
     nodes: merge_nodes(p1, p2),
     conns: inherited_conns,
     fitness: 0.0,
   };
   ```

---
## 3. Speciation & Reproduction (`Population::reproduce`)
1. **Compatibility**: define distance:
   ```rust
   δ = c1 * E/N + c2 * D/N + c3 * W;
   ```
2. Group genomes into species by `δ < threshold`.
3. Within each species, select parents (e.g. tournament).
4. Generate offspring via crossover + mutate.
5. Fill next generation to `pop_size`, carry over elites if desired.

---
## 4. Evolution Loop (in `main.rs`)
Wrap evaluate/reproduce:
```rust
let mut pop = Population::new(&evo_cfg);
for gen in 0..max_gen {
    pop.evaluate(&sim_cfg, &evo_cfg);
    log_stats(gen, &pop);
    pop.reproduce(&evo_cfg);
}
```

---
## 5. Recording & Visualization
- Use `run_match_record` to dump JSONL per tick.
- Serve with WASM+Canvas via `browser_preview` for UI.

---
## 6. Testing
- Unit tests: small custom network for `feed_forward`.
- Property tests: ensure mutate/crossover preserve invariants.

---

By following these steps, you'll have a working NEAT pipeline that evolves both weights and topology. Enjoy!
