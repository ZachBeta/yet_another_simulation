# Multi-Bracket Training Comparison Plan

Compare NEAT champions across team sizes 1v1, 2v2, 3v3, and 4v4 under varying durations, fitness functions, and salvage incentives.

## 1. Parameter Grid

- **Team Sizes**: 1, 2, 3, 4 (use `--team-size N --num-teams N`)
- **Durations**: 30s, 60s
- **Fitness Functions**:
  - `health-plus-damage`
  - `health-plus-damage-time`
- **Salvage**: only on 60s runs (to test salvage benefit)
- **Runs**: `--runs 200` per config

### Experiment Matrix

| Bracket | Duration | Fitness Fn              | Salvage | Runs |
|---------|----------|-------------------------|---------|------|
| 1v1     | 30s      | health-plus-damage      | No      | 200  |
| 1v1     | 30s      | health-plus-damage-time | No      | 200  |
| 1v1     | 60s      | health-plus-damage      | No      | 200  |
| 1v1     | 60s      | health-plus-damage-time | No      | 200  |
| 1v1     | 60s      | health-plus-damage      | Yes     | 200  |
| 1v1     | 60s      | health-plus-damage-time | Yes     | 200  |
| 2v2     | 30s      | health-plus-damage      | No      | 200  |
| 2v2     | 30s      | health-plus-damage-time | No      | 200  |
| 2v2     | 60s      | health-plus-damage      | No      | 200  |
| 2v2     | 60s      | health-plus-damage-time | No      | 200  |
| 2v2     | 60s      | health-plus-damage      | Yes     | 200  |
| 2v2     | 60s      | health-plus-damage-time | Yes     | 200  |
| 3v3     | (same pattern)                                  |         |      |
| 4v4     | (same pattern)                                  |         |      |

## 2. Run ID Scheme

Format: `<bracket>-<duration>s-<fn>` + optional `-salvage`

Examples:
```
1v1-30s-health-plus-damage
3v3-60s-health-plus-damage-time-salvage
```

## 3. Automation

1. **Refactor** `scripts/run_experiments.js`:
   - Loop over arrays:
     ```js
     const teamSizes = [1,2,3,4];
     const durations = [30,60];
     const fitnessFns = ['health-plus-damage','health-plus-damage-time'];
     const salvageFlags = duration===60 ? [false,true] : [false];
     ```
   - Build release binary once, then invoke:
     ```bash
     neat_train train \
       --team-size N --num-teams N \
       --duration D \
       --fitness-fn FN \
       --w-health 1.0 --w-damage 1.0 --w-kills 0.5 --time-bonus-weight 0.1 \
       --runs 200 \
       [--enable-salvage] \
       --run-id <run_id>
     ```
2. **Tournament Automation**: run `node scripts/run_tournaments.js` to generate Elo ratings and update `runs.json`.

## 4. Analysis & UI

- Refresh browser to see new runs in dropdown.
- Compare Elo distributions and head-to-head replays per bracket.
- Identify scaling effects: does 4v4 require longer durations or different fitness weights?

---

Once you approve this plan, I can update the experiment script accordingly and kick off the full sweep.
