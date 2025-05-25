# Development Guide

This guide provides comprehensive instructions for setting up the development environment, understanding the project structure, and contributing to the simulation project.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Structure](#project-structure)
3. [Environment Setup](#environment-setup)
4. [Development Workflow](#development-workflow)
5. [Running the Simulation](#running-the-simulation)
6. [Training New Models](#training-new-models)
7. [Browser-Based Battle Simulator](#browser-based-battle-simulator)
8. [Testing](#testing)
9. [Contributing](#contributing)

## Prerequisites

- **Rust** (1.70+):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
  ```

- **Node.js** (14+) and npm:
  - Download from [nodejs.org](https://nodejs.org/) or use a version manager like `nvm`

- **Web Browser**: Chrome or Firefox (latest versions)

- **Git**: For version control

## Project Structure

```
/ (root)
├── sim_core/          # Core Rust crate
│   ├── src/           # Source code
│   ├── out/           # Training outputs (runs.json, champion folders)
│   └── out_archive/   # Archived training runs
├── scripts/           # Automation scripts
│   ├── generate_runs.js
│   ├── run_experiments.js
│   └── run_tournaments.js
├── docs/              # Documentation
├── web/               # Frontend code
│   ├── index.html
│   ├── script.js
│   └── styles.css
├── package.json       # JavaScript dependencies
└── Cargo.toml         # Rust dependencies
```

## Environment Setup

1. Clone the repository:
   ```bash
   git clone git@github.com:ZachBeta/yet_another_simulation.git
   cd yet_another_simulation
   ```

2. Install JavaScript dependencies:
   ```bash
   npm install
   ```

3. Build the WebAssembly modules:
   ```bash
   cd sim_core
   wasm-pack build --target web
   cd ..
   ```

## Model Naming and Output Organization

Consistent model naming and output organization is crucial for managing multiple training runs. This section outlines the conventions and automation for generating run IDs and organizing outputs.

### Run ID Generation

Run IDs are auto-generated with the following format:
```
<timestamp>-fn-<fitness-function>-h<health-weight>-d<damage-weight>-k<kills-weight>
```

Example: `20250518_213000-fn-health-plus-damage-h1.0-d1.0-k0.5`

### CLI Override

You can override the auto-generated ID with a custom name:
```bash
cargo run -- train --run-id my_custom_name
```

### Output Directory Structure

Each run creates a directory in the `sim_core/out/` folder with the following structure:

```
sim_core/out/
├── <run_id>/
│   ├── champion_gen_001.json      # Champion from generation 1
│   ├── champion_gen_002.json      # Champion from generation 2
│   ├── champion_latest.json       # Latest champion
│   ├── champ_replay_001.jsonl     # Replay data for generation 1
│   ├── metrics.csv                # Training metrics
│   └── config.toml               # Training configuration
```

### Implementation Details

1. **CLI Options**
   The `TrainOpts` struct includes an optional `run_id` field:
   ```rust
   #[clap(long)]
   run_id: Option<String>,
   ```

2. **Auto-Generating Run IDs**
   ```rust
   let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
   let fn_name = opts.fitness_fn.to_possible_value().unwrap().get_name();
   let id = opts.run_id.clone().unwrap_or_else(|| format!(
       "{}-fn-{}-h{:.1}-d{:.1}-k{:.1}",
       ts, fn_name, opts.w_health, opts.w_damage, opts.w_kills
   ));
   ```

3. **Creating Output Directories**
   ```rust
   let out_dir = format!("out/{}", id);
   std::fs::create_dir_all(&out_dir).unwrap();
   ```

### Best Practices

1. **Naming Conventions**
   - Use descriptive names when manually specifying run IDs
   - Include key parameters in the name (e.g., `2v2-60s-salvage`)
   - Avoid spaces and special characters in run IDs

2. **File Management**
   - Keep all related files for a run in its directory
   - Use consistent naming for output files
   - Archive old runs by moving them to `sim_core/out_archive/`

3. **Logging**
   Include the run ID in log messages for better traceability:
   ```rust
   eprintln!("[{}][{:.1}s] Saved generation {}", id, elapsed, gen);
   ```

## Development Workflow

### 1. Generate Run Catalog

The `runs.json` file serves as the source of truth for the browser simulator's dropdown menus.

```bash
# Scan sim_core/out and archives to generate runs.json
node scripts/generate_runs.js
```

### 2. Run Training Experiments

The project supports 1v1–4v4 training sweeps via `run_experiments.js`:

```bash
node scripts/run_experiments.js
```

This script will:
- Build the release `neat_train` binary
- Loop over team sizes (1–4), durations, fitness functions, and salvage flags
- Output to `sim_core/out/<run_id>/` with JSON and snapshots

> **Tip**: Edit `scripts/run_experiments.js` to adjust runs or add new fitness variants.

### 3. Run Tournaments & Update Elo Ratings

After training completes, run tournaments to evaluate models:

```bash
node scripts/run_tournaments.js
```

This will:
- Build the `neat_train` binary if needed
- Execute tournaments for each run (including the naive agent)
- Regenerate `runs.json` with updated `best_elo` ratings

## Running the Simulation

1. Start the development server:
   ```bash
   npm start
   # or
   python3 -m http.server 8000
   ```

2. Open http://localhost:8000 in your browser

3. Use the dropdown to select any two champions to battle

## Training New Models

### Basic Training

To train a new model with default parameters:

```bash
cd sim_core
cargo run --release --bin neat_train -- --runs 1
```

### Advanced Training Options

Customize training with various parameters:

```bash
cargo run --release --bin neat_train -- \
  --runs 5 \
  --population 100 \
  --generations 100 \
  --team-size 2 \
  --duration 60 \
  --fitness damage_taken \
  --salvage
```

## Browser-Based Battle Simulator

The browser simulator allows you to:
- View battles between any two trained models
- Adjust simulation speed
- Toggle visualization options
- View agent statistics

### Keyboard Shortcuts

- **Space**: Pause/Resume
- **R**: Reset simulation
- **1-9**: Adjust simulation speed
- **D**: Toggle debug info

## Testing

Run Rust tests:
```bash
cd sim_core
cargo test
```

Run JavaScript tests:
```bash
npm test
```

## Contributing

1. Create a new branch for your feature or bugfix
2. Write tests for your changes
3. Ensure all tests pass
4. Submit a pull request with a clear description of changes

### Code Style

- **Rust**: Follow the Rust style guide (run `cargo fmt` before committing)
- **JavaScript**: Follow standard JavaScript style (run `npx standard --fix` before committing)
- **Documentation**: Keep documentation up to date with code changes

## Troubleshooting

### Common Issues

- **WASM Build Failures**: Try `wasm-pack clean` and rebuild
- **Missing Dependencies**: Run `npm install` and ensure all Rust toolchains are up to date
- **Performance Issues**: Use release builds (`--release` flag) for training
