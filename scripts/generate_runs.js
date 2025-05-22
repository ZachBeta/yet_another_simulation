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
      let bestElo = null;
      // try per-run Elo file
      const eloFile = path.join(runDir, 'elo_ratings.json');
      try {
        const data = await fs.readFile(eloFile, 'utf8');
        const list = JSON.parse(data);
        if (Array.isArray(list) && list.length) {
          const best = list.reduce((max, e) => e.elo > max.elo ? e : max, list[0]);
          bestElo = best.elo;
        }
      } catch (_) {
        // fallback to champion fitness
        const champFile = path.join(runDir, 'champion_latest.json');
        try {
          const champData = await fs.readFile(champFile, 'utf8');
          const champJson = JSON.parse(champData);
          if (champJson.genome && typeof champJson.genome.fitness === 'number') {
            bestElo = champJson.genome.fitness;
          }
        } catch (_) {
          // no data available
        }
      }
      if (bestElo !== null) runs.push({ run_id: runId, best_elo: bestElo });
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
