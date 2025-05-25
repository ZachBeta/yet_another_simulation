# Midlevel SWE Tutorial: NEAT Multi-Bracket Training & Browser Simulation

This guide walks a midlevel software engineer through setting up, running, and extending our NEAT training pipeline (1v1–4v4) and browser-based battle simulator.

## Prerequisites

- **Rust** (1.70+): `rustup toolchain install stable && rustup default stable`
- **Node.js** (14+): download from https://nodejs.org or via `nvm`
- **npm/yarn**: for JS scripts
- **Chrome/Firefox**: modern browser
- **Git**: clone the repo and sync changes

## Repository Layout

```
/ (root)
├─ sim_core/          # Rust crate: training, tournament, WASM bindings
│  ├─ src/
│  ├─ out/            # training outputs: runs.json, champion folders
│  └─ out_archive/    # archived runs
├─ scripts/           # automation scripts
│  ├─ generate_runs.js
│  ├─ run_experiments.js
│  └─ run_tournaments.js
├─ docs/
│  ├─ 2V2_BROWSER_SIM_TUTORIAL.md    # 2v2-specific tutorial
│  ├─ MULTI_BRACKET_TRAINING_PLAN.md # experiment plan
│  └─ MIDLEVEL_SWE_TUTORIAL.md       # <--- you are here
├─ web/              # frontend: index.html, script.js, CSS
├─ package.json      # JS dependencies
└─ Cargo.toml        # Rust dependencies
```

## 1. Environment Setup

1. Clone and cd into repo:
   ```bash
   git clone git@github.com:ZachBeta/yet_another_simulation.git
   cd yet_another_simulation
   ```
2. Install Rust and Node.js (see prereqs).
3. Install JS deps:
   ```bash
   npm install
   ```
4. Build WASM for browser:
   ```bash
   cd sim_core && wasm-pack build --target web && cd ..
   ```

## 2. Generate Run Catalog

Runs.json is the source-of-truth for dropdowns in the browser sim.

```bash
# Scan sim_core/out and archives
node scripts/generate_runs.js
```

Inspect `sim_core/out/runs.json` to verify your runs.

## 3. Launch Training Experiments

We support 1v1–4v4 sweeps via `run_experiments.js`:

```bash
node scripts/run_experiments.js
```

This script will:
- Build the release `neat_train` binary
- Loop over team sizes (1–4), durations (30s,60s), fitness fns, and salvage flags
- Emit `sim_core/out/<run_id>/` with JSON and snapshots

> Tip: Edit `scripts/run_experiments.js` to adjust `runs` or add new fitness variants.

## 4. Run Tournaments & Update Elo

Once training finishes, run:

```bash
node scripts/run_tournaments.js
```

This will:
- Build `neat_train`
- Execute tournaments for each run (including naive agent)
- Regenerate `runs.json` with updated `best_elo`

## 5. Serve & View Browser Sim

Start a simple HTTP server at root (e.g., via Python):

```bash
npm run serve      # if configured in package.json
# or
python3 -m http.server 8000
```

Then open http://localhost:8000 in your browser. The dropdown will list all runs; select any two champions to battle.

## 6. Customization & Extension

- **Grid tweaks**: Modify `docs/MULTI_BRACKET_TRAINING_PLAN.md` and update `scripts/run_experiments.js` loops.
- **Fitness functions**: Add new enums in `sim_core/src/main.rs` under `FitnessFnArg`, recompile.
- **Simulation parameters**: Adjust `--duration`, `--team-size`, or introduce new flags in `clap` definitions.
- **Frontend**: Extend `script.js` to display metadata, Elo charts, or head-to-head replays.

## 7. Troubleshooting

- **Arg parse errors**: run `sim_core/target/release/neat_train train --help` to check flag names.
- **WASM load errors**: confirm `sim_core/out/.../champion_latest.json` includes expected fields.
- **CORS/404**: serve from project root and check resource paths in `index.html`.

## 8. Next Steps

- Integrate CI (GitHub Actions) to run a subset of experiments on PRs.
- Add Dockerfile for environment reproducibility.
- Hook up a simple web API (FastAPI/Express) to serve runs dynamically instead of static JSON.

---

Happy coding! Feel free to tweak this tutorial or pipeline to fit your workflow.
