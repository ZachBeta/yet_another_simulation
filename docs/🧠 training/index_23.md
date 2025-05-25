# Model Selection UX: OptGroup Dropdown + Metadata Panel

This tutorial walks you through implementing a scalable, user-friendly model selection interface for your NEAT training frontend. When you have dozens of runs, grouping by date and showing key metadata makes picking the right champion painless.

## Prerequisites
- A Rust-based `run_pipeline` that writes per-run metadata (`metadata.json`) and maintains a top-level `runs_index.json`.
- A web UI (`index.html` + `script.js`) that can fetch JSON files from your `out/` directory.

## Step 1: Define and Emit Run Metadata in Rust

1. Add a `RunMetadata` struct in `main.rs`:

   ```rust
   use serde::{Serialize, Deserialize};

   #[derive(Serialize, Deserialize)]
   struct RunMetadata {
     run_id: String,
     timestamp: String,     // ISO 8601
     team_size: usize,
     num_teams: usize,
     fitness_fn: String,
     best_fitness: f32,
     best_generation: usize,
   }
   ```

2. At the end of `run_pipeline`, compute your hall-of-fame best and write:

   ```rust
   // inside run_pipeline
   let meta = RunMetadata { /* fill fields */ };
   let run_dir = format!("out/{}", run_id);
   serde_json::to_writer(/* file: run_dir/metadata.json */, &meta)?;
   
   // update top-level index
   let mut index: Vec<RunMetadata> = if exists("out/runs_index.json") {
     read and parse
   } else { Vec::new() };
   if !index.iter().any(|r| r.run_id == meta.run_id) {
     index.push(meta);
     overwrite "out/runs_index.json" with index
   }
   ```

## Step 2: Add OptGroup Dropdown and Metadata Panel in HTML

1. Edit `index.html`: replace free-text with a grouped selector and detail panel.

   ```html
   <!-- Model Selection Controls -->
   <label>
     Select Run:
     <select id="runSelect"></select>
   </label>
   <div id="runMeta" style="margin-top:1em;"></div>
   ```

## Step 3: Load and Render Groups + Metadata in JavaScript

In `script.js`:

```js
let runsIndex = [];
async function loadRuns() {
  const resp = await fetch('out/runs_index.json');
  runsIndex = await resp.json();

  // Group by date
  const byDate = runsIndex.reduce((map, r) => {
    const day = r.timestamp.split('T')[0];
    (map[day] ||= []).push(r);
    return map;
  }, {});

  const runSelect = document.getElementById('runSelect');
  Object.keys(byDate).sort().reverse().forEach(day => {
    const optgroup = document.createElement('optgroup');
    optgroup.label = day;
    byDate[day].forEach(r => {
      const label = `${r.timestamp.slice(11,19)} [${r.fitness_fn}] → ${r.best_fitness.toFixed(1)}`;
      optgroup.append(new Option(label, r.run_id));
    });
    runSelect.append(optgroup);
  });
}

// On run change, show metadata and reload champions
runSelect.onchange = () => {
  const id = runSelect.value;
  const meta = runsIndex.find(r => r.run_id === id);
  document.getElementById('runMeta').innerHTML = `
    <strong>Params:</strong> teams=${meta.team_size}×${meta.num_teams}, fn=${meta.fitness_fn}<br>
    <strong>Best:</strong> ${meta.best_fitness.toFixed(2)} @ gen ${meta.best_generation}
  `;
  loadEloRatings(id);
};

// Initialize
loadRuns().then(() => {
  runSelect.selectedIndex = 0;
  runSelect.onchange();
  requestAnimationFrame(loop);
});
```

## Next Steps and Polishing
- **Filters:** Add a simple text-filter above the dropdown to narrow by fitness_fn or run_id.  
- **Top-N Toggle:** Show only top K runs by default, with a checkbox to reveal all.  
- **Styling:** Use CSS cards or tables in `#runMeta` for richer visual presentation.

Now you have a grouped, metadata-rich model selector that scales gracefully as your run collection grows.
