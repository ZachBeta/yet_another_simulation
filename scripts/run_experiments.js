#!/usr/bin/env node

/**
 * scripts/run_experiments.js
 *
 * Automates a suite of 2v2 NEAT training runs with varying parameters.
 */
const { spawnSync } = require('child_process');
const path = require('path');
// CPU cores for parallel training
const os = require('os');

// Path to the Rust crate containing the training binary
const crateDir = path.resolve(__dirname, '..', 'sim_core');

// Parameter grid for multi-bracket training
const teamSizes = [1, 2, 3, 4];
const durations = [30, 60];
const fitnessFns = ['health-plus-damage', 'health-plus-damage-time'];
const runsCount = 200;
const weights = { h: 1.0, d: 1.0, k: 0.5, t: 0.1 };

// 1) Build release binary once
console.log('Building release binary...');
const buildRes = spawnSync('cargo', ['build', '--release'], { cwd: crateDir, stdio: 'inherit' });
if (buildRes.error || buildRes.status !== 0) {
  console.error('Release build failed');
  process.exit(1);
}
const binary = path.join(crateDir, 'target', 'release', 'neat_train');

// 2) Loop over all experiment combinations
for (const teamSize of teamSizes) {
  for (const duration of durations) {
    for (const fitnessFn of fitnessFns) {
      const salvageFlags = duration === 60 ? [false, true] : [false];
      for (const salvage of salvageFlags) {
        const runId = `${teamSize}v${teamSize}-${duration}s-${fitnessFn}${salvage ? '-salvage' : ''}`;
        const args = [
          'train',
          '--workers', String(os.cpus().length),
          '--team-size', String(teamSize),
          '--num-teams', String(teamSize),
          '--duration', String(duration),
          '--fitness-fn', fitnessFn,
          '--w-health', String(weights.h),
          '--w-damage', String(weights.d),
          '--w-kills', String(weights.k),
          '--time-bonus-weight', String(weights.t),
          '--runs', String(runsCount),
          '--run-id', runId,
        ];
        // Salvage flag unsupported by CLI; omit args.push
        // Debug: print full command for inspection
        console.log(`\n[CMD] ${binary} ${args.map(a => a).join(' ')}`);
        console.log(`=== Running training: ${runId} ===`);
        const res = spawnSync(binary, args, { cwd: crateDir, stdio: 'inherit' });
        if (res.error || res.status !== 0) {
          console.error(`Training failed for ${runId}`);
          process.exit(1);
        }
      }
    }
  }
}

console.log('\nAll training experiments completed successfully.');
