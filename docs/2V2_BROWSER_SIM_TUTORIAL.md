# 2v2 Browser Simulation Tutorial

A step-by-step guide for integrating 2v2 champion selection and battle playback into the existing Rust/WASM frontend. This assumes you’re a mid-level SWE familiar with JavaScript, TypeScript, and basic Rust/WASM workflows.

## Prerequisites

- Node.js & npm installed (for local dev server)
- A working Rust toolchain and `wasm-pack` to compile the simulation
- Project cloned at `yet_another_simulation/`
- Existing 2v2 output in `sim_core/out/<run_id>/`
- Basic familiarity with `script.js`, `index.html`, and the Rust/WASM exports in `wasm/pkg/`

## Overview of the Flow

1. **Discover runs**: read `sim_core/out/runs.json` to populate a “Select Run” dropdown.
2. **Show run metadata**: display the run ID and its best Elo score.
3. **List champions**: fetch `elo_ratings.json` under the chosen run and fill the champion dropdown.
4. **Load champion**: fetch its JSON metadata + genome, then call the WASM entrypoint to spin up a 2v2 NN vs. naive simulation.
5. **Render loop**: replay the ticks in the browser canvas.

---

## Step 1: Create `runs.json`

In `sim_core/out/`, add a file named `runs.json`. This file drives the run selector.

```json
[
  { "run_id": "2v2-30s", "best_elo": 1234.5 },
  { "run_id": "2v2-60s", "best_elo": 1278.2 }
]
```

You can initially maintain it by hand. Later you can automate its generation in Rust or Node.

## Step 2: Update HTML Markup

Open `index.html` and ensure you have:

```html
<label>
  Select Run:
  <select id="runSelect" style="width:200px"></select>
</label>
<div id="runMeta" style="margin-left:8px; border:1px solid #ccc; padding:8px; max-width:300px;"></div>
```

These elements will host the run list and metadata.

## Step 3: Enhance `script.js`

1. **Grab DOM nodes** near the top of the file:
   ```js
   const runSelect = document.getElementById('runSelect');
   const runMeta   = document.getElementById('runMeta');
   ```

2. **Add `loadRuns()`** above `loadEloRatings`:
   ```js
   async function loadRuns() {
     const resp = await fetch('sim_core/out/runs.json');
     const runs = await resp.json();
     runs.forEach(({ run_id, best_elo }) => {
       const label = `${run_id} (Elo ${best_elo.toFixed(1)})`;
       runSelect.add(new Option(label, run_id));
     });
     runSelect.onchange = () => {
       const run = runs.find(r => r.run_id === runSelect.value);
       runMeta.innerHTML = `
         <strong>Run:</strong> ${run.run_id}<br/>
         <strong>Best Elo:</strong> ${run.best_elo.toFixed(1)}
       `;
       loadEloRatings(run.run_id);
     };
     runSelect.selectedIndex = 0;
     runSelect.onchange();
   }
   ```

3. **Boot sequence** at the bottom:
   ```js
   // Replace existing champion-load boot
   loadRuns()
     .then(() => requestAnimationFrame(loop));
   ```

This ensures runs and champions load before drawing begins.

## Step 4: Verify Your Work

1. Start the local server:
   ```bash
   npm install
   npm start
   # serves index.html, script.js, style.css on http://localhost:8000
   ```
2. Open your browser at `http://localhost:8000`.
3. The **Select Run** dropdown should list your runs with Elo scores.
4. Picking a run updates the run metadata and fills the champion dropdown.
5. Click **Start** to watch your 2v2 NN vs. naive battle animate on the canvas.

## Step 5: Next Steps

- **Automate** `runs.json` output in your Rust pipeline post‐training.
- Add **filters** (date, performance thresholds) to the UI.
- When scaling, swap flat files for a lightweight HTTP API or database.

---

That’s it—enjoy interactive 2v2 battles right in your browser! Feel free to tweak the UI or frame data as your simulation evolves.
