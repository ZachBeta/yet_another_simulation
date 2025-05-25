# Phase 2: NN vs Naive Duel Tutorial

This tutorial shows how to wire up a simple head-to-head simulation in which your new `NNAgent` plays against the existing `NaiveAgent` in a checkerboard formation (TL & BR vs TR & BL).

## Prerequisites
- Completed Phase 1: nearest-K sensor encoding.
- Familiarity with `Simulation::new(...)` and front-end rendering loop.

## 1. Refactor spawn logic

Factor out the quadrant spawn + brain registration into a helper:

```rust
impl Simulation {
    fn spawn_quadrants(
        &mut self,
        counts: [u32;4],                     // [orange,yellow,green,blue]
        factories: &[fn() -> Box<dyn Brain>], // Brain factory per side
        assignment: &[usize;4],               // map TL,TR,BL,BR → side index
    ) {
        // existing `new` logic (setting up agents_data)
        // then register_agent(factories[assignment[q]]()) for each ship
    }
}
```

## 2. Add the duel constructor

Expose a new constructor in `sim_core/src/lib.rs`:

```rust
#[wasm_bindgen]
pub fn new_nn_vs_naive(
    width: u32, height: u32,
    orange: u32, yellow: u32,
    green: u32, blue: u32,
) -> Simulation {
    let mut sim = Simulation::empty(width, height);
    sim.spawn_quadrants(
      [orange, yellow, green, blue],
      &[ 
        || Box::new(NNAgent),        // side 0
        || Box::new(NaiveBrain(...)) // side 1
      ],
      &[0, 1, 1, 0], // TL&BR=NN, TR&BL=Naive
    );
    sim
}
```

> *Tip:* Keep the old `Simulation::new(...)` unchanged so existing demos/tests continue to work.

## 3. Front-end integration

In your `script.js`, replace the reset logic:

```js
// Before:
sim = await Simulation.new(w, h, oCount, yCount, gCount, bCount);

// After:
sim = await Simulation.new_nn_vs_naive(
  w, h, oCount, yCount, gCount, bCount
);
```

No changes to HTML controls are required—you still read the four color count fields. 

## 4. Running the duel

1. Click **Reset** to spawn two teams in checkerboard quadrants.
2. Click **Start** to begin stepping the simulation and drawing agents in orange (vs green).
3. Observe behavior: NN agents (TL/BR) will cohesion-steer, naive agents (TR/BL) follow their built-in rules.

At any point, click **Pause** or **Reset** to restart the match.

## 5. Next steps
- Add end-of-match metrics (win/loss, kills).
- Expose other brain matchups (e.g. random vs NN).
- Build a tournament mode over multiple pairings.

Congratulations! You now have a head-to-head duel flow for evaluating your NN agent against the baseline.
