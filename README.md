# Yet Another Simulation

A WebAssembly-powered neural network battle simulation. Watch AI agents trained with NEAT evolution battle each other in real-time.

## Quick Start (2 minutes)

### Prerequisites
```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo install wasm-pack

# Install Node.js (any recent version)
# macOS: brew install node
# Or download from: https://nodejs.org/
```

### Run the Simulation
```bash
git clone <your-repo>
cd yet_another_simulation

# Install dependencies and start
npm install
npm start

# Open http://localhost:8000
# Select a trained model from the dropdown and watch the battle!
```

**That's it!** The repository includes 58 pre-trained AI models ready to use.

## What You'll See

- **Colored dots**: Each represents an AI agent with neural network decision-making
- **Model dropdown**: Different AI models trained with various parameters
- **Real-time combat**: Agents use weapons, steering behaviors, and strategy
- **Team battles**: 1v1, 2v2, 3v3, or 4v4 configurations
- **Elo ratings**: Models ranked by tournament performance

## Current Scripts (All Working)

### For Users
```bash
npm start                        # Run the simulation
npm test                         # Run UI tests
node scripts/generate_runs.js    # Refresh model catalog
```

### For Training New Models  
```bash
node scripts/run_experiments.js      # Train multiple model variants (~hours)
node scripts/run_tournaments.js      # Run tournaments on trained models
node scripts/run_global_tournament.js # Cross-parameter model comparison
node scripts/compare_fitness_variants.js # Compare different fitness functions
```

## File Structure (What Matters)

```
├── index.html           # Simulation interface
├── script.js            # Frontend logic  
├── wasm/pkg/           # Compiled WASM module (pre-built)
├── sim_core/out/       # 58 trained models (ready to use)
│   ├── runs.json       # Model catalog for frontend
│   └── */              # Individual model directories
└── scripts/            # Working automation scripts
```

## Understanding the Models

Each model in the dropdown shows:
- **Team size**: 1v1, 2v2, 3v3, 4v4
- **Duration**: 30s, 60s, 120s battle length used for training
- **Fitness function**: What the AI optimized for
  - `health-plus-damage`: Survival + combat effectiveness
  - `health-damage-salvage`: Above + resource collection
  - `health-damage-explore`: Above + map exploration
- **Elo rating**: Performance against other models

Try different models to see how training parameters affect behavior!

## Training Your Own Models

The existing training system works but takes time:

```bash
# Full training suite (creates many model variants)
node scripts/run_experiments.js

# This will:
# - Train 1v1, 2v2, 3v3, 4v4 team configurations  
# - Use 30s and 60s battle durations
# - Try different fitness functions
# - Take several hours total
# - Output to sim_core/out/<model-name>/
```

After training, update the model catalog:
```bash
node scripts/run_tournaments.js     # Generate Elo ratings
node scripts/generate_runs.js       # Update frontend catalog
```

## Troubleshooting

**Models not loading**: Run `node scripts/generate_runs.js`
**Port 8000 busy**: Use `npm start -- --port=8001`  
**Build issues**: The WASM module is pre-built, but if needed: `cd sim_core && wasm-pack build --target web --out-dir ../wasm/pkg`

## Technical Details

- **Rust WASM core**: High-performance simulation engine
- **NEAT evolution**: Topology-evolving neural networks
- **Real-time visualization**: Canvas-based rendering
- **Model persistence**: JSON format for easy sharing
- **Tournament system**: Elo rating-based model comparison

The simulation demonstrates emergent AI behavior where simple rules create complex strategy.

## Screenshots

![Battle Simulation](./Screenshot%202025-05-04%20at%2012.35.25.png)
