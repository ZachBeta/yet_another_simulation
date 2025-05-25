# Tutorial: Scalable Model Selection UX

**Audience:** Mid-Level Software Engineers

In projects with frequent training runs, browsing raw directories becomes a chore. This tutorial shows how to implement an **OptGroup‐based selector** in your web UI, backed by a simple metadata index, so you can group runs by date and view key statistics at a glance.

---
## 1. Overview

We’ll build:
1. **Rust pipeline support** to emit per‐run metadata and maintain a central index.  
2. **HTML** with a grouped `<select>` (using `<optgroup>`) and a metadata panel.  
3. **JavaScript** to load runs, populate groups, and display details on selection.

---
## 2. Prerequisites

- A Rust CLI (`run_pipeline`) that produces run artifacts under `out/{run_id}`.  
- A web folder with `index.html` and `script.js` that serve static assets.  
- [serde_json](https://docs.rs/serde_json) in your `Cargo.toml`.

---
## 3. Step 1: Emit Run Metadata in Rust

### 3.1 Define `RunMetadata`
In `src/main.rs`:
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct RunMetadata {
    run_id: String,
    timestamp: String,       // ISO 8601
    team_size: usize,
    num_teams: usize,
    fitness_fn: String,
    best_fitness: f32,
    best_generation: usize,
}
```

### 3.2 Write `metadata.json` and Update `runs_index.json`
At the end of your `run_pipeline` function:
```rust
// build metadata
let meta = RunMetadata {
    run_id: run_id.clone(),
    timestamp: chrono::Utc::now().to_rfc3339(),
    team_size: opts.train.team_size,
    num_teams: opts.train.num_teams,
    fitness_fn: opts.train.fitness_fn.to_string(),
    best_fitness: hof[0].fitness,
    best_generation: hof[0].generation,
};
// write to out/{run_id}/metadata.json
let run_dir = format!("out/{}", run_id);
fs::write(
    format!("{}/metadata.json", run_dir),
    serde_json::to_string_pretty(&meta)?
)?;
// update top‐level index
let index_path = "out/runs_index.json";
let mut index: Vec<RunMetadata> = if Path::new(index_path).exists() {
    serde_json::from_str(&fs::read_to_string(index_path)?)?
} else {
    Vec::new()
};
if !index.iter().any(|r| r.run_id == meta.run_id) {
    index.push(meta);
    fs::write(
        index_path,
        serde_json::to_string_pretty(&index)?
    )?;
}
```

This ensures each run folder has its metadata and `out/runs_index.json` tracks all runs.

---
## 4. Step 2: HTML – OptGroup Dropdown + Metadata Panel

Edit `index.html` under your `docs/` or root:

```html
<!-- Run Selector -->
<label>
  Select Run:
  <select id="runSelect"></select>
</label>
<!-- Metadata Panel -->
<div id="runMeta" class="run-meta-panel"></div>
```

You can style `.run-meta-panel` via CSS for clarity.

---
## 5. Step 3: JavaScript – Load and Render Groups

In `script.js`:

```js
let runsIndex = [];
const runSelect = document.getElementById('runSelect');
const runMeta = document.getElementById('runMeta');

// 1) Load runs_index.json and group by date
async function loadRuns() {
  const resp = await fetch('out/runs_index.json');
  runsIndex = await resp.json();

  const byDate = runsIndex.reduce((m,r) => {
    const day = r.timestamp.split('T')[0];
    (m[day] ||= []).push(r);
    return m;
  }, {});

  // populate optgroups
  Object.keys(byDate).sort().reverse().forEach(day => {
    const og = document.createElement('optgroup');
    og.label = day;
    byDate[day].forEach(r => {
      const label = `${r.timestamp.slice(11,19)} [${r.fitness_fn}] → ${r.best_fitness.toFixed(1)}`;
      og.append(new Option(label, r.run_id));
    });
    runSelect.append(og);
  });
}

// 2) On change: show metadata and load champions
runSelect.onchange = () => {
  const id = runSelect.value;
  const meta = runsIndex.find(r => r.run_id === id);
  runMeta.innerHTML = `
    <strong>Params:</strong> teams=${meta.team_size}×${meta.num_teams}, fn=${meta.fitness_fn}<br>
    <strong>Best:</strong> ${meta.best_fitness.toFixed(2)} @ gen ${meta.best_generation}
  `;
  loadEloRatings(id);
};

// 3) Initialize on page load
(async () => {
  await loadRuns();
  runSelect.selectedIndex = 0;
  runSelect.onchange();
  requestAnimationFrame(loop);
})();
```

---
## 6. Next Steps & Polishing

- **Add a Date Filter:** A simple `<input type="date">` above the selector to narrow dates.  
- **Top-N Toggle:** Limit default options to Top-10 runs by `best_fitness`.  
- **Styling:** Use tables or cards for `#runMeta` to show more fields.  

With this setup, your UI scales to hundreds of runs while keeping selection clear and context‐rich. Happy coding!
