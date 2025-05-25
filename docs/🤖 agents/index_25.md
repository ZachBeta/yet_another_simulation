# Agent Event Logging System

This document describes the implementation and usage of the agent event logging system, which captures detailed per-tick agent activities for training and debugging purposes.

## Table of Contents

1. [Overview](#1-overview)
2. [Event Types](#2-event-types)
3. [JSONL Format](#3-jsonl-format)
4. [Implementation Details](#4-implementation-details)
   - [4.1 Event Definition](#41-event-definition)
   - [4.2 Event Collection](#42-event-collection)
   - [4.3 WebAssembly Integration](#43-webassembly-integration)
5. [Web Interface](#5-web-interface)
6. [Usage Examples](#6-usage-examples)
7. [Best Practices](#7-best-practices)
8. [Future Extensions](#8-future-extensions)

## 1. Overview

The agent event logging system captures detailed information about agent behavior during simulation, including:

- Agent state snapshots
- Actions taken
- Game events (hits, loot collection, etc.)
- Game state transitions

These logs are exported in JSONL (JSON Lines) format for easy processing and analysis.

## 2. Event Types

The following event types are supported:

### State Event
Captures the agent's state before making a decision.

```json
{
  "State": {
    "tick": 42,
    "agent_id": 0,
    "inputs": [0.1, 0.5, 0.8, ...]
  }
}
```

### Action Event
Records the action chosen by the agent.

```json
{
  "Action": {
    "tick": 42,
    "agent_id": 0,
    "action": "Thrust(1.0,0.0)"
  }
}
```

### Hit Event (Planned)
To be implemented for tracking combat interactions.

### Loot Event (Planned)
To be implemented for tracking resource collection.

### GameOver Event (Planned)
To be implemented for tracking game completion.

## 3. JSONL Format

The logging system uses JSONL (JSON Lines) format, where each line is a valid JSON object representing a single event. This format is:

- **Line-oriented**: Each line is a complete JSON object
- **Streamable**: Can be processed line by line
- **Appendable**: New events can be appended to existing log files
- **Human-readable**: Easy to inspect with standard tools

## 4. Implementation Details

### 4.1 Event Definition

Events are defined in `sim_core/src/log.rs`:

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
    // Additional event types will be added here
}
```

### 4.2 Event Collection

Events are collected in the `Simulation` struct:

```rust
pub struct Simulation {
    // ... existing fields ...
    events: Vec<AgentEvent>,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            // ... initialize other fields ...
            events: Vec::new(),
        }
    }
    
    pub fn log_event(&mut self, event: AgentEvent) {
        self.events.push(event);
    }
}
```

### 4.3 WebAssembly Integration

The logging system is exposed to JavaScript through WebAssembly:

```rust
#[wasm_bindgen]
impl Simulation {
    /// Export all logged events as a JSONL string
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

## 5. Web Interface

### HTML Button

Add a download button to your `index.html`:

```html
<button id="downloadLog" class="control-button">
  <i class="fas fa-download"></i> Download Event Log
</button>
```

### JavaScript Integration

Wire up the button in `script.js`:

```javascript
// Get the download button
const downloadBtn = document.getElementById('downloadLog');

// Set up click handler
downloadBtn.onclick = () => {
    // Get events as JSONL
    const jsonl = sim.export_events_jsonl();
    
    // Create a download link
    const blob = new Blob([jsonl], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `agent_events_${new Date().toISOString().replace(/[:.]/g, '-')}.jsonl`;
    
    // Trigger download
    document.body.appendChild(a);
    a.click();
    
    // Clean up
    setTimeout(() => {
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }, 0);
};
```

## 6. Usage Examples

### Logging Agent State

```rust
// In your agent's think() method
let inputs = self.get_inputs();
let event = AgentEvent::State {
    tick: self.tick,
    agent_id: self.id,
    inputs: inputs.clone(),
};
sim.log_event(event);

// Agent decision making logic
let action = self.brain.think(&inputs);
```

### Logging Agent Actions

```rust
// After the agent has selected an action
let event = AgentEvent::Action {
    tick: self.tick,
    agent_id: self.id,
    action: format!("{:?}", action),
};
sim.log_event(event);
```

## 7. Best Practices

1. **Selective Logging**
   - Only log events that are needed for your current analysis
   - Be mindful of performance impact when logging at high frequency

2. **Event Design**
   - Keep event payloads small and focused
   - Use enums for event types to ensure type safety
   - Include timestamps or tick numbers for correlation

3. **Performance Considerations**
   - Pre-allocate event buffers when possible
   - Consider sampling for high-frequency events
   - Use efficient serialization formats

4. **Log Management**
   - Implement log rotation for long-running simulations
   - Include version information in log formats
   - Document the schema of each event type

## 8. Future Extensions

### Additional Event Types

1. **Combat Events**
   - Damage dealt/received
   - Ability usage
   - Status effects

2. **Resource Management**
   - Ammo/energy usage
   - Health/shield changes
   - Resource collection

3. **Spatial Awareness**
   - Position updates
   - Proximity to objectives
   - Line of sight changes

### Advanced Features

1. **Filtering**
   - Per-agent event filtering
   - Event type whitelist/blacklist
   - Sampling rates

2. **Remote Logging**
   - WebSocket streaming
   - Batch uploads
   - Real-time monitoring

3. **Analytics Integration**
   - Real-time visualization
   - Performance metrics
   - Behavior analysis
