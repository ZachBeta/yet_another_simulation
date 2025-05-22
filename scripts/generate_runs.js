#!/usr/bin/env node

/**
 * generate_runs.js
 *
 * Scans output directories for 2v2 simulation runs and writes sim_core/out/runs.json
 */
const fs = require('fs').promises;
const path = require('path');

// Base directories to scan for runs
const BASE_DIRS = [
  path.join(__dirname, '..', 'sim_core', 'out'),
  path.join(__dirname, '..', 'sim_core', 'out_archive'),
  path.join(__dirname, '..', 'out_archive_root'),
];

async function findRuns() {
  const runs = [];
  for (const base of BASE_DIRS) {
    let entries;
    try {
      entries = await fs.readdir(base, { withFileTypes: true });
    } catch (e) {
      continue;
    }
    for (const dirent of entries) {
      if (!dirent.isDirectory()) continue;
      const runId = dirent.name;
      const runDir = path.join(base, runId);
      const eloFile = path.join(runDir, 'elo_ratings.json');
      try {
        const data = await fs.readFile(eloFile, 'utf8');
        const list = JSON.parse(data);
        if (Array.isArray(list) && list.length) {
          // find max elo
          const best = list.reduce((max, e) => e.elo > max.elo ? e : max, list[0]);
          runs.push({ run_id: runId, best_elo: best.elo });
        }
      } catch (e) {
        // skip runs without elo_ratings.json
        continue;
      }
    }
  }
  return runs;
}

async function writeRunsJson() {
  const runs = await findRuns();
  const outPath = path.join(__dirname, '..', 'sim_core', 'out', 'runs.json');
  await fs.writeFile(outPath, JSON.stringify(runs, null, 2));
  console.log(`Wrote ${runs.length} runs to ${outPath}`);
}

writeRunsJson().catch(err => {
  console.error('Error generating runs.json:', err);
  process.exit(1);
});
