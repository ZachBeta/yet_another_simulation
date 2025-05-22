#!/usr/bin/env node
/**
 * Compare fitness variants by training and tournament.
 */
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const simCoreDir = path.resolve(__dirname, '..', 'sim_core');
const outDir = path.join(simCoreDir, 'out');

const variants = [
  { name: 'health-plus-damage', cliArgs: ['--fitness-fn', 'health-plus-damage'] },
  { name: 'health-damage-salvage', cliArgs: ['--fitness-fn', 'health-damage-salvage', '--w-salvage', '0.1'] },
  { name: 'health-damage-explore', cliArgs: ['--fitness-fn', 'health-damage-explore', '--w-explore', '0.05'] },
  { name: 'health-damage-time-salvage-explore', cliArgs: ['--fitness-fn', 'health-damage-time-salvage-explore', '--w-salvage', '0.1', '--w-explore', '0.05', '--time-bonus-weight', '0.1'] }
];

for (const { name, cliArgs } of variants) {
  console.log(`\n=== Training variant: ${name} ===`);
  const trainArgs = [
    'train',
    '--run-id', name,
    '--runs', '50',
    '--snapshot-interval', '5',
    '--map-var', '0',
    '--w-health', '1.0',
    '--w-damage', '1.0',
    '--w-kills', '0.5',
    ...cliArgs
  ];
  const res = spawnSync(path.join(simCoreDir, 'target', 'release', 'neat_train'), trainArgs, { cwd: simCoreDir, stdio: 'inherit' });
  if (res.status !== 0) {
    console.error(`Training failed for variant ${name}`);
    process.exit(res.status);
  }
  const src = path.join(outDir, name, 'champion_latest.json');
  const dest = path.join(outDir, `champion_${name}.json`);
  fs.copyFileSync(src, dest);
  console.log(`Saved ${dest}`);
}

console.log('\n=== Running global tournament ===');
const tourArgs = ['tournament', '--tournament-include-naive'];
variants.forEach(v => tourArgs.push('--pop-file', path.join(outDir, `champion_${v.name}.json`)));
const res2 = spawnSync(path.join(simCoreDir, 'target', 'release', 'neat_train'), tourArgs, { cwd: simCoreDir, stdio: 'inherit' });
if (res2.status !== 0) process.exit(res2.status);
