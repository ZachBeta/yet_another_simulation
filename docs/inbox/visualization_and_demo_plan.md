# Visualization & Demo Roadmap

This document outlines a three-week plan to enhance the browser replay viewer (“Load Replay”) and introduce a live NN-vs-NN demo, plus future extras.

---

## Track 1 – Load Replay Enhancements (Week 1)

1. **Robust Fetch & Parse**
   - Validate `resp.ok` and `Content-Type` before parsing
   - Wrap `JSON.parse` in try/catch, surface clear UI error messages
2. **UI/UX Upgrades**
   - Replace free-text path with file-picker or dropdown listing `out/*.jsonl`
   - Add playback controls: Play/Pause, Step Frame, Speed Slider
   - Show loading spinner and error alerts
3. **Stats & Visualization**
   - Display per-team health as small inline chart
   - Log tick-by-tick summaries in a collapsible pane
4. **TypeScript & Testing**
   - Clean up TS types, remove unchecked casts
   - Unit tests for `drawReplay` and `updateReplayStats`

---

## Track 2 – Live Heads-Up Matches (Week 2)

### Rust/WASM Changes
- Expose `WasmSimulation::new_nn_vs_nn(jsonA: &str, jsonB: &str)`
  - Load two champion genomes
  - Spawn one NN agent per side
  - Return a simulation instance ready to step

### JavaScript/UI
- Two file inputs or dropdowns to select champion JSONs
- **Run Match** button to instantiate and start live sim
- Live canvas render + `updateStats`
- Post-match summary: winner badge, ticks to finish, health curves

---

## Track 3 – Unified Replay Recording (Week 3)

- Record every NN-vs-NN live match to a JSONL replay file
- Add **Save Replay** button in UI
- Leverage existing “Load Replay” viewer for post-mortem playback
- Store replays under `out/nn_vs_nn_<champA>_vs_<champB>.jsonl`

---

## Future Extras

- **Elo-Rating Leaderboard**: compute from recorded match outcomes
- **Side-by-Side Replay Comparison**: dual playback panes
- **Custom Arena Config**: let user tweak teams, counts, and config in UI
- **Performance Dashboard**: visualize average TPS, HTTP vs CPU timing

---

*Prepared on 2025-05-18*
