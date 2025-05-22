#!/usr/bin/env node

/**
 * scripts/run_tournaments.js
 *
 * Automates tournaments for all existing runs and updates runs.json.
 */
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Paths
const crateDir = path.resolve(__dirname, '..', 'sim_core');
const rootDir = path.resolve(__dirname, '..');

// 1) Build release binary
console.log('Building release binary for tournament...');
let res = spawnSync('cargo', ['build', '--release'], { cwd: crateDir, stdio: 'inherit' });
if (res.error || res.status !== 0) {
  console.error('Release build failed');
  process.exit(1);
}
const binary = path.join(crateDir, 'target', 'release', 'neat_train');

// 2) Discover run directories across outputs
console.log('Discovering run dirs across outputs...');
const baseDirs = [
  path.join(crateDir, 'out'),
  path.join(crateDir, 'out_archive'),
  path.join(rootDir, 'out_archive_root'),
];
const runSet = new Set();
for (const bd of baseDirs) {
  if (fs.existsSync(bd)) {
    fs.readdirSync(bd, { withFileTypes: true })
      .filter(d => d.isDirectory())
      .forEach(d => runSet.add(d.name));
  }
}
const runIds = Array.from(runSet).sort();
console.log(`[+] Found ${runIds.length} runs: ${runIds.join(', ')}`);

// 3) Run tournament for each run
for (const runId of runIds) {
  console.log(`\n=== Running tournament: ${runId} ===`);
  // resolve run folder
  let popDir;
  for (const bd of baseDirs) {
    const cand = path.join(bd, runId);
    if (fs.existsSync(cand)) { popDir = cand; break; }
  }
  if (!popDir) {
    console.warn(`Skipping ${runId}: directory not found`);
    continue;
  }
  const args = ['tournament', '--pop-path', popDir, '--tournament-include-naive'];
  const res = spawnSync(binary, args, { cwd: crateDir, stdio: 'inherit' });
  if (res.error || res.status !== 0) {
    console.error(`Tournament failed for ${runId}`);
    process.exit(1);
  }
}

// 4) Regenerate runs.json with updated Elo ratings
console.log('\nUpdating run catalog (runs.json) with new Elo values...');
res = spawnSync('node', ['scripts/generate_runs.js'], { cwd: rootDir, stdio: 'inherit' });
if (res.error || res.status !== 0) {
  console.error('Failed to update runs.json');
  process.exit(1);
}

console.log('\nAll tournaments and run catalog updates completed.');
