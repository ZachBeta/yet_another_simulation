#!/usr/bin/env node
/**
 * scripts/run_global_tournament.js
 *
 * Aggregate all run champions across brackets and durations,
 * then run a single cross-run Elo tournament via neat_train.
 */
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const crateDir = path.resolve(__dirname, '..', 'sim_core');
const rootDir = path.resolve(__dirname, '..', '..');
const bin = path.join(crateDir, 'target', 'release', 'neat_train');

// 1) Discover champion files
const baseDirs = [
  path.join(crateDir, 'out'),
  path.join(crateDir, 'out_archive'),
  path.join(rootDir, 'out_archive_root')
];
const champs = [];
for (const bd of baseDirs) {
  if (!fs.existsSync(bd)) continue;
  fs.readdirSync(bd).forEach(runId => {
    const d = path.join(bd, runId);
    if (!fs.statSync(d).isDirectory()) return;
    const files = fs.readdirSync(d)
      .filter(f => f.endsWith('.json') && !['elo_ratings.json', 'metadata.json', 'runs.json'].includes(f))
      .sort();
    const champ = files.pop();
    if (champ) champs.push(path.join(d, champ));
  });
}
console.log(`Found ${champs.length} champions for global tournament`);

if (champs.length === 0) {
  console.error('No champion JSONs found.');
  process.exit(1);
}

// 2) Run global tournament with explicit pop-file flags
const args = ['tournament', '--tournament-include-naive'];
champs.forEach(c => args.push('--pop-file', c));
console.log('[CMD]', bin, args.join(' '));
const res = spawnSync(bin, args, { cwd: crateDir, stdio: 'inherit' });
if (res.error || res.status !== 0) process.exit(1);
