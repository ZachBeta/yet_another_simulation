# 2v2 Full Pipeline: Train → Tournament → Battle

This document outlines the complete end-to-end workflow for 2v2 NEAT training, evaluation, and final replay.

## 1. Training Stage

1. Launch training:
   ```bash
   cargo run --release -- train \
     --team-size 2 --num-teams 2 \
     --duration 3600    # or --runs 200
   ```
2. Monitor logs:
   - Generation labels show `(2v2)`
   - Fitness progression, plateau when `best` and `avg` saturate.
3. Plateau detection:
   - Stop when no improvement for `stagnation_window` gens.
   - Inject recovery genomes or terminate.
4. Output:
   - `out/<run_id>/champion_latest.json`: final HOF champion.
   - `out/<run_id>/champion_*.json`: snapshots.

## 2. Tournament Stage

1. Run tournament on saved champions:
   ```bash
   cargo run --release -- tournament \
     --pop-path out/<run_id> \
     --include-naive
   ```
2. This computes Elo or ranking among HOF champions + naive baseline.
3. Output: console table of champion rankings.

## 3. Battle / Replay Stage

1. Replay best vs second-best:
   - Already recorded as `champ_replay.jsonl` during training.
2. Visualize / inspect:
   ```bash
   cargo run --release -- battle \
     --replay out/<run_id>/champ_replay.jsonl
   ```
   (or use `live-server` / front-end viewer)
3. Analyze team behavior, coordinated tactics.

## 4. Next Steps

- **Automation**: wrap these commands in a shell script or CI pipeline.
- **Metrics**: add diversity logs during training (species count, compatibility).
- **Visualization**: integrate browser preview of battle replays.

*Generated on 2025-05-19*
