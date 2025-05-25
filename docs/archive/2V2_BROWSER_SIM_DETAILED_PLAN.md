# 2v2 Browser Simulation Integration Plan

This document details the steps to integrate 2v2 champion selection and battle playback into the browser simulation using the existing Rust/WASM codebase.

## 1. runs.json

- **Location**: `sim_core/out/runs.json`
- **Purpose**: List available training runs and key metrics.
- **Schema**:
  ```json
  [
    {
      "run_id": "2v2-30s",
      "best_elo": 1234.5
    },
    {
      "run_id": "2v2-60s",
      "best_elo": 1278.2
    }
  ]
  ```
- **Maintenance**: Manually update initially; later script-generate.

## 2. HTML Markup Updates

1. Ensure `index.html` has:
   - `<select id="runSelect"></select>` (existing).
   - `<div id="runMeta"></div>` (existing).
2. Optionally add labels:
   ```html
   <label>
     Select Run:
     <select id="runSelect" style="width:200px;"></select>
   </label>
   ```

## 3. script.js Modifications

### 3.1 Import & Globals

Add global reference after other `getElementById` calls:
```js
const runSelect = document.getElementById('runSelect');
const runMeta   = document.getElementById('runMeta');
```

### 3.2 loadRuns()

```js
async function loadRuns() {
  const resp = await fetch('sim_core/out/runs.json');
  const runs = await resp.json();
  runs.forEach(({ run_id, best_elo }) => {
    const label = `${run_id} (Best Elo ${best_elo.toFixed(1)})`;
    runSelect.add(new Option(label, run_id));
  });
  runSelect.onchange = () => {
    const id = runSelect.value;
    const run = runs.find(r => r.run_id === id);
    runMeta.innerHTML = `
      <strong>Run:</strong> ${id}<br/>
      <strong>Best Elo:</strong> ${run.best_elo.toFixed(1)}
    `;
    loadEloRatings(id);
  };
  // Trigger initial change
  runSelect.selectedIndex = 0;
  runSelect.onchange();
}
```

### 3.3 Bootstrap Sequence

Replace the final load/loop with:
```js
loadRuns()
  .then(() => requestAnimationFrame(loop));
```

## 4. Directory Structure

```
sim_core/
└── out/
    ├── runs.json           # new
    └── 2v2-30s/
        ├── champion_latest.json
        └── elo_ratings.json
```

## 5. Testing & Validation

1. Run `npm start` to serve the frontend.
2. Open `http://localhost:8000/index.html`.
3. Ensure `runSelect` populates, `runMeta` updates, and champion dropdown works.
4. Click “Start” to verify 2v2 battle renders correctly.

## 6. Next Steps

1. Automate `runs.json` generation in Rust post-training.
2. Persist metadata to a lightweight API or DB when scaling.
3. Add UI to filter runs by date, team size, etc.
