# Agent Event Logging

This document outlines how to capture per-tick agent events in `sim_core` and export them as JSONL for training and debugging.

## 1. Event Types

Define in `sim_core/src/log.rs`:

```rust
use serde::Serialize;

#[derive(Serialize)]
pub enum AgentEvent {
  /// Snapshot of inputs before agent decision
  State {
    tick: u32,
    agent_id: usize,
    inputs: Vec<f32>,
  },
  /// Action chosen by the agent
  Action {
    tick: u32,
    agent_id: usize,
    action: String,
  },
  // TODO: Add Hit, Loot, GameOver events
}
```

## 2. JSONL Schema

- Each line is one JSON object for an event.  
- Example lines:

```json
{"State": {"tick": 42, "agent_id": 0, "inputs": [0.1, 0.5, ...]}}
{"Action": {"tick": 42, "agent_id": 0, "action": "Thrust(1.0,0.0)"}}
```

## 3. In-Memory Store

Add an `events: Vec<AgentEvent>` field to `Simulation`:

```rust
pub struct Simulation {
  // ... existing fields ...
  events: Vec<AgentEvent>,
}
```

- In `NaiveAgent::think()`, push a `State` event before computing.
- In `Simulation::step()`, after inserting the command, push an `Action` event.

## 4. WASM Binding

Expose an exporter in `sim_core/src/lib.rs`:

```rust
#[wasm_bindgen]
impl Simulation {
  pub fn export_events_jsonl(&mut self) -> String {
    let out = self.events
      .iter()
      .map(|e| serde_json::to_string(e).unwrap())
      .collect::<Vec<_>>()
      .join("\n");
    self.events.clear();
    out
  }
}
```

- Requires `serde = { features = ["derive"] }` and `wasm-bindgen` in `Cargo.toml`.

## 5. JavaScript Integration

Add a Download button in `index.html`:

```html
<button id="downloadLog">Download Log</button>
```

Wire it up in `script.js`:

```js
const downloadBtn = document.getElementById('downloadLog');
downloadBtn.onclick = () => {
  const jsonl = sim.export_events_jsonl();
  const blob = new Blob([jsonl], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `events_${Date.now()}.jsonl`;
  a.click();
  URL.revokeObjectURL(url);
};
```

## 6. Next Steps

- Decide additional events (Hit, Loot, GameOver) to record.
- Integrate scanner-based inputs once available.
- Create a Rust CLI tool to ingest JSONL into SQLite/postgres for training.

*Document last updated: 2025-05-07*
